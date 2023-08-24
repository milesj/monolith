use moon_time::chrono::NaiveDateTime;
use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SigninInput {
    pub organization_key: String,
    pub repository: String,
    pub repository_key: String,
}

#[derive(Debug, Default, Deserialize, Serialize)]
#[serde(default, rename_all = "camelCase")]
pub struct SigninResponse {
    pub ci_insights: bool,
    pub organization_id: i32,
    pub remote_caching: bool,
    pub repository_id: i32,
    pub token: String,
}

// ARTIFACTS

#[derive(Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Artifact {
    pub id: i64,
    pub repository_id: i32,
    pub hash: String,
    pub size: i32,
    pub target: String,
    pub path: String,
    pub created_at: NaiveDateTime,
    pub deleted_at: Option<NaiveDateTime>,
}

#[derive(Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ArtifactWriteInput {
    pub target: String,
    pub size: usize,
}

#[derive(Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ArtifactCompleteInput {
    pub job_id: Option<i64>,
    pub success: bool,
}

#[derive(Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ArtifactResponse {
    pub artifact: Artifact,
    pub presigned_url: Option<String>,
}

#[derive(Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct EmptyData {}
