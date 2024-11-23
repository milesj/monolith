use crate::fs_digest::Blob;
use crate::grpc_tls::*;
use crate::remote_client::RemoteClient;
use crate::remote_error::RemoteError;
use bazel_remote_apis::build::bazel::remote::execution::v2::{
    action_cache_client::ActionCacheClient, batch_update_blobs_request,
    capabilities_client::CapabilitiesClient, compressor,
    content_addressable_storage_client::ContentAddressableStorageClient, digest_function,
    ActionResult, BatchReadBlobsRequest, BatchUpdateBlobsRequest, Digest, GetActionResultRequest,
    GetCapabilitiesRequest, ServerCapabilities, UpdateActionResultRequest,
};
use moon_common::color;
use moon_config::RemoteConfig;
use std::{error::Error, path::Path};
use tonic::{
    transport::{Channel, Endpoint},
    Code,
};
use tracing::{trace, warn};

fn map_transport_error(error: tonic::transport::Error) -> RemoteError {
    RemoteError::ConnectFailed {
        error: Box::new(error),
    }
}

fn map_status_error(error: tonic::Status) -> RemoteError {
    match error.source() {
        Some(src) => RemoteError::CallFailedViaSource {
            error: src.to_string(),
        },
        None => RemoteError::CallFailed {
            error: Box::new(error),
        },
    }
}

#[derive(Default)]
pub struct GrpcRemoteClient {
    channel: Option<Channel>,
    instance_name: String,
}

#[async_trait::async_trait]
impl RemoteClient for GrpcRemoteClient {
    async fn connect_to_host(
        &mut self,
        config: &RemoteConfig,
        workspace_root: &Path,
    ) -> miette::Result<()> {
        let host = &config.host;

        trace!(
            instance = &config.cache.instance_name,
            "Connecting to gRPC host {} {}",
            color::url(host),
            if config.mtls.is_some() {
                "(with mTLS)"
            } else if config.tls.is_some() {
                "(with TLS)"
            } else {
                "(insecure)"
            }
        );

        let mut endpoint = Endpoint::from_shared(host.to_owned())
            .map_err(map_transport_error)?
            .user_agent("moon")
            .map_err(map_transport_error)?;

        if let Some(mtls) = &config.mtls {
            endpoint = endpoint
                .tls_config(create_mtls_config(mtls, workspace_root)?)
                .map_err(map_transport_error)?;
        } else if let Some(tls) = &config.tls {
            endpoint = endpoint
                .tls_config(create_tls_config(tls, workspace_root)?)
                .map_err(map_transport_error)?;
        }

        self.channel = Some(endpoint.connect().await.map_err(map_transport_error)?);
        self.instance_name = config.cache.instance_name.clone();

        Ok(())
    }

    // https://github.com/bazelbuild/remote-apis/blob/main/build/bazel/remote/execution/v2/remote_execution.proto#L452
    async fn load_capabilities(&self) -> miette::Result<ServerCapabilities> {
        let mut client = CapabilitiesClient::new(self.channel.clone().unwrap());

        trace!("Loading remote execution API capabilities from gRPC server");

        let response = client
            .get_capabilities(GetCapabilitiesRequest {
                instance_name: self.instance_name.clone(),
            })
            .await
            .map_err(map_status_error)?;

        Ok(response.into_inner())
    }

    // https://github.com/bazelbuild/remote-apis/blob/main/build/bazel/remote/execution/v2/remote_execution.proto#L170
    async fn get_action_result(&self, digest: &Digest) -> miette::Result<Option<ActionResult>> {
        let mut client = ActionCacheClient::new(self.channel.clone().unwrap());

        trace!(hash = &digest.hash, "Checking for a cached action result");

        match client
            .get_action_result(GetActionResultRequest {
                instance_name: self.instance_name.clone(),
                action_digest: Some(digest.to_owned()),
                inline_stderr: true,
                inline_stdout: true,
                digest_function: digest_function::Value::Sha256 as i32,
                ..Default::default()
            })
            .await
        {
            Ok(response) => {
                let result = response.into_inner();

                trace!(
                    hash = &digest.hash,
                    files = result.output_files.len(),
                    links = result.output_symlinks.len(),
                    dirs = result.output_directories.len(),
                    exit_code = result.exit_code,
                    "Cache hit on action result"
                );

                Ok(Some(result))
            }
            Err(status) => {
                if matches!(status.code(), Code::NotFound) {
                    trace!(hash = &digest.hash, "Cache miss on action result");

                    Ok(None)
                } else {
                    Err(map_status_error(status).into())
                }
            }
        }
    }

    // https://github.com/bazelbuild/remote-apis/blob/main/build/bazel/remote/execution/v2/remote_execution.proto#L193
    async fn update_action_result(
        &self,
        digest: &Digest,
        result: ActionResult,
    ) -> miette::Result<Option<ActionResult>> {
        let mut client = ActionCacheClient::new(self.channel.clone().unwrap());

        trace!(
            hash = &digest.hash,
            files = result.output_files.len(),
            links = result.output_symlinks.len(),
            dirs = result.output_directories.len(),
            exit_code = result.exit_code,
            "Caching action result"
        );

        match client
            .update_action_result(UpdateActionResultRequest {
                instance_name: self.instance_name.clone(),
                action_digest: Some(digest.to_owned()),
                action_result: Some(result),
                digest_function: digest_function::Value::Sha256 as i32,
                ..Default::default()
            })
            .await
        {
            Ok(response) => {
                trace!(hash = &digest.hash, "Cached action result");

                Ok(Some(response.into_inner()))
            }
            Err(status) => {
                let code = status.code();

                if matches!(code, Code::InvalidArgument | Code::FailedPrecondition) {
                    warn!(
                        code = ?code,
                        "Failed to cache action result: {}",
                        status.message()
                    );

                    Ok(None)
                } else if matches!(code, Code::ResourceExhausted) {
                    warn!(
                        code = ?code,
                        "Remote service is out of storage space: {}",
                        status.message()
                    );

                    Ok(None)
                } else {
                    Err(map_status_error(status).into())
                }
            }
        }
    }

    // https://github.com/bazelbuild/remote-apis/blob/main/build/bazel/remote/execution/v2/remote_execution.proto#L403
    async fn batch_read_blobs(
        &self,
        digest: &Digest,
        blob_digests: Vec<Digest>,
    ) -> miette::Result<Vec<Blob>> {
        let mut client = ContentAddressableStorageClient::new(self.channel.clone().unwrap());

        trace!(
            hash = &digest.hash,
            "Downloading {} output blobs",
            blob_digests.len()
        );

        let response = match client
            .batch_read_blobs(BatchReadBlobsRequest {
                acceptable_compressors: vec![compressor::Value::Identity as i32],
                instance_name: self.instance_name.clone(),
                digests: blob_digests,
                digest_function: digest_function::Value::Sha256 as i32,
            })
            .await
        {
            Ok(res) => res,
            Err(status) => {
                return if matches!(status.code(), Code::InvalidArgument) {
                    warn!(
                        hash = &digest.hash,
                        "Attempted to download more blobs than the allowed limit"
                    );

                    Ok(vec![])
                } else {
                    Err(map_status_error(status).into())
                };
            }
        };

        let mut blobs = vec![];
        let mut total_count = 0;

        for download in response.into_inner().responses {
            if let Some(status) = download.status {
                if status.code != 0 {
                    warn!(
                        details = ?status.details,
                        "Failed to download blob: {}",
                        status.message
                    );
                }
            }

            if let Some(digest) = download.digest {
                blobs.push(Blob {
                    digest,
                    bytes: download.data,
                });
            }

            total_count += 1;
        }

        trace!(
            hash = &digest.hash,
            "Downloaded {} of {} output blobs",
            blobs.len(),
            total_count
        );

        Ok(blobs)
    }

    // https://github.com/bazelbuild/remote-apis/blob/main/build/bazel/remote/execution/v2/remote_execution.proto#L379
    async fn batch_update_blobs(
        &self,
        digest: &Digest,
        blobs: Vec<Blob>,
    ) -> miette::Result<Vec<Option<Digest>>> {
        let mut client = ContentAddressableStorageClient::new(self.channel.clone().unwrap());

        trace!(
            hash = &digest.hash,
            "Uploading {} output blobs",
            blobs.len()
        );

        let response = match client
            .batch_update_blobs(BatchUpdateBlobsRequest {
                instance_name: self.instance_name.clone(),
                requests: blobs
                    .into_iter()
                    .map(|blob| batch_update_blobs_request::Request {
                        digest: Some(blob.digest),
                        data: blob.bytes,
                        compressor: compressor::Value::Identity as i32,
                    })
                    .collect(),
                digest_function: digest_function::Value::Sha256 as i32,
            })
            .await
        {
            Ok(res) => res,
            Err(status) => {
                let code = status.code();

                return if matches!(code, Code::InvalidArgument) {
                    warn!(
                        hash = &digest.hash,
                        "Attempted to upload more blobs than the allowed limit"
                    );

                    Ok(vec![])
                } else if matches!(code, Code::ResourceExhausted) {
                    warn!(
                        code = ?code,
                        "Remote service exhausted resource: {}",
                        status.message()
                    );

                    Ok(vec![])
                } else {
                    Err(map_status_error(status).into())
                };
            }
        };

        let mut digests = vec![];
        let mut uploaded_count = 0;

        for upload in response.into_inner().responses {
            if let Some(status) = upload.status {
                if status.code != 0 {
                    warn!(
                        details = ?status.details,
                        "Failed to upload blob: {}",
                        status.message
                    );
                }
            }

            if upload.digest.is_some() {
                uploaded_count += 1;
            }

            digests.push(upload.digest);
        }

        trace!(
            hash = &digest.hash,
            "Uploaded {} of {} output blobs",
            uploaded_count,
            digests.len()
        );

        Ok(digests)
    }
}