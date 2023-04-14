use crate::cache_item;
use crate::helpers::get_cache_mode;
use moon_archive::{untar_with_diff, TarArchiver, TreeDiffer};
use moon_error::MoonError;
use moon_logger::trace;
use moon_utils::{fs, glob};
use serde::{Deserialize, Serialize};
use starbase_styles::color;
use starbase_utils::json;
use std::path::{Path, PathBuf};

#[derive(Debug, Default, Deserialize, Eq, PartialEq, Serialize)]
#[serde(default, rename_all = "camelCase")]
pub struct RunTargetState {
    pub exit_code: i32,

    pub hash: String,

    pub last_run_time: u128,

    pub target: String,

    #[serde(skip)]
    pub path: PathBuf,
}

cache_item!(RunTargetState);

fn prepare_outputs_list(outputs: &[String], source: &str) -> Vec<String> {
    let mut list = vec![];

    for output in outputs {
        if let Some(value) = output.strip_prefix('/') {
            list.push(value.to_owned());
        } else if source.is_empty() {
            list.push(output.to_owned());
        } else {
            list.push(format!("{source}/{output}"));
        }
    }

    list
}

impl RunTargetState {
    pub fn archive_outputs(
        &self,
        archive_file: &Path,
        workspace_root: &Path,
        project_source: &str,
        outputs: &[String],
    ) -> Result<bool, MoonError> {
        if get_cache_mode().is_writable() && !archive_file.exists() {
            let mut tar = TarArchiver::new(workspace_root, archive_file);

            // Outputs are relative from project root (the input)
            if !outputs.is_empty() {
                let outputs = prepare_outputs_list(outputs, project_source);

                for output in &outputs {
                    if glob::is_glob(output) {
                        tar.add_source_glob(output, None);
                    } else {
                        tar.add_source(workspace_root.join(output), Some(output));
                    }
                }
            }

            // Also include stdout/stderr logs at the root of the tarball
            let (stdout_path, stderr_path) = self.get_output_logs();

            if stdout_path.exists() {
                tar.add_source(stdout_path, Some("stdout.log"));
            }

            if stderr_path.exists() {
                tar.add_source(stderr_path, Some("stderr.log"));
            }

            tar.pack().map_err(|e| MoonError::Generic(e.to_string()))?;

            return Ok(true);
        }

        Ok(false)
    }

    pub fn hydrate_outputs(
        &self,
        archive_file: &Path,
        workspace_root: &Path,
        project_source: &str,
        outputs: &[String],
    ) -> Result<bool, MoonError> {
        if get_cache_mode().is_readable() && archive_file.exists() {
            let outputs = prepare_outputs_list(outputs, project_source);
            let mut differ = TreeDiffer::load(workspace_root, &outputs)?;

            untar_with_diff(&mut differ, archive_file, workspace_root, None)
                .map_err(|e| MoonError::Generic(e.to_string()))?;

            let cache_logs = self.get_output_logs();
            let stdout_log = workspace_root.join("stdout.log");
            let stderr_log = workspace_root.join("stderr.log");

            if stdout_log.exists() {
                fs::rename(&stdout_log, cache_logs.0)?;
            }

            if stderr_log.exists() {
                fs::rename(&stderr_log, cache_logs.1)?;
            }

            return Ok(true);
        }

        Ok(false)
    }

    pub fn get_output_logs(&self) -> (PathBuf, PathBuf) {
        (
            self.get_dir().join("stdout.log"),
            self.get_dir().join("stderr.log"),
        )
    }

    /// Load the stdout.log and stderr.log files from the cache directory.
    pub fn load_output_logs(&self) -> Result<(String, String), MoonError> {
        let (stdout_path, stderr_path) = self.get_output_logs();

        let stdout = if stdout_path.exists() {
            fs::read(stdout_path)?
        } else {
            String::new()
        };

        let stderr = if stderr_path.exists() {
            fs::read(stderr_path)?
        } else {
            String::new()
        };

        Ok((stdout, stderr))
    }

    /// Write stdout and stderr log files to the cache directory.
    pub fn save_output_logs(&self, stdout: String, stderr: String) -> Result<(), MoonError> {
        let (stdout_path, stderr_path) = self.get_output_logs();

        fs::write(stdout_path, stdout)?;
        fs::write(stderr_path, stderr)?;

        Ok(())
    }
}
