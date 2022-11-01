use crate::vcs::{TouchedFiles, Vcs, VcsResult};
use async_trait::async_trait;
use moon_utils::fs;
use moon_utils::process::{output_to_string, output_to_trimmed_string, Command};
use regex::Regex;
use std::collections::{BTreeMap, HashMap, HashSet};
use std::fs::metadata;
use std::path::{Path, PathBuf};
use std::sync::Arc;
use std::time::SystemTime;
use tokio::sync::RwLock;

// TODO: This code hasn't been tested yet and may not be accurate!

pub struct Svn {
    cache: Arc<RwLock<HashMap<String, String>>>,
    default_branch: String,
    root: PathBuf,
}

impl Svn {
    pub fn load(default_branch: &str, working_dir: &Path) -> Self {
        let root = match fs::find_upwards(".svn", working_dir) {
            Some(dir) => dir.parent().unwrap().to_path_buf(),
            None => working_dir.to_path_buf(),
        };

        Svn {
            cache: Arc::new(RwLock::new(HashMap::new())),
            default_branch: String::from(default_branch),
            root,
        }
    }

    fn extract_line_from_info(&self, label: &str, info: &str) -> String {
        for line in info.split('\n') {
            if line.starts_with(label) {
                return String::from(line).replace(label, "").trim().to_owned();
            }
        }

        String::new()
    }

    async fn get_revision_number(&self, revision: &str) -> VcsResult<String> {
        let output = self
            .run_command(&mut self.create_command(vec!["info", "-r", revision]), true)
            .await?;

        Ok(self.extract_line_from_info("Revision:", &output))
    }

    fn process_touched_files(output: String) -> TouchedFiles {
        if output.is_empty() {
            return TouchedFiles::default();
        }

        let mut added = HashSet::new();
        let mut deleted = HashSet::new();
        let mut modified = HashSet::new();
        let mut untracked = HashSet::new();
        let mut staged = HashSet::new();
        let unstaged = HashSet::new();
        let mut all = HashSet::new();

        for line in output.split('\n') {
            if line.is_empty() {
                continue;
            }

            let mut chars = line.chars();
            let x = chars.next().unwrap_or_default();
            let y = chars.next().unwrap_or_default();
            let file = String::from(&line[8..]);

            match x {
                'A' | 'C' => {
                    added.insert(file.clone());
                }
                'D' => {
                    deleted.insert(file.clone());
                }
                'M' | 'R' => {
                    modified.insert(file.clone());
                    staged.insert(file.clone());
                }
                '?' => {
                    untracked.insert(file.clone());
                }
                _ => {}
            }

            if y == 'M' {
                modified.insert(file.clone());
            }

            all.insert(file.clone());

            // svn files are always staged by default
            staged.insert(file.clone());
        }

        TouchedFiles {
            added,
            all,
            deleted,
            modified,
            staged,
            unstaged, // svn has no concept for this
            untracked,
        }
    }

    async fn run_command(&self, command: &mut Command, trim: bool) -> VcsResult<String> {
        let (cache_key, _) = command.get_command_line();

        // Read first before locking with a write
        {
            let cache = self.cache.read().await;

            if cache.contains_key(&cache_key) {
                return Ok(cache.get(&cache_key).unwrap().clone());
            }
        }

        // Otherwise lock and calculate a new value to write
        let mut cache = self.cache.write().await;
        let output = command.exec_capture_output().await?;

        let value = if trim {
            output_to_trimmed_string(&output.stdout)
        } else {
            output_to_string(&output.stdout)
        };

        cache.insert(cache_key.to_owned(), value.clone());

        Ok(value)
    }
}

// https://edoras.sdsu.edu/doc/svn-book-html-chunk/svn.ref.svn.c.info.html
#[async_trait]
impl Vcs for Svn {
    fn create_command(&self, args: Vec<&str>) -> Command {
        let mut cmd = Command::new("svn");
        cmd.args(args).cwd(&self.root);
        cmd
    }

    async fn get_local_branch(&self) -> VcsResult<String> {
        let output = self
            .run_command(&mut self.create_command(vec!["info"]), false)
            .await?;
        let url = self.extract_line_from_info("URL:", &output);
        let pattern = Regex::new("branches/([^/]+)").unwrap();

        if pattern.is_match(&url) {
            let caps = pattern.captures(&url).unwrap();

            return Ok(String::from(
                caps.get(1)
                    .map_or(self.default_branch.as_str(), |m| m.as_str()),
            ));
        }

        Ok(self.get_default_branch().to_owned())
    }

    async fn get_local_branch_revision(&self) -> VcsResult<String> {
        Ok(self.get_revision_number("BASE").await?)
    }

    fn get_default_branch(&self) -> &str {
        &self.default_branch
    }

    async fn get_default_branch_revision(&self) -> VcsResult<String> {
        Ok(self.get_revision_number("HEAD").await?)
    }

    async fn get_file_hashes(&self, files: &[String]) -> VcsResult<BTreeMap<String, String>> {
        let mut map = BTreeMap::new();

        // svn doesnt support file hashing, so instead of generating some
        // random hash ourselves, just use the modified time.
        for file in files {
            if let Ok(metadata) = metadata(file) {
                if let Ok(modified) = metadata.modified() {
                    map.insert(
                        file.to_owned(),
                        modified
                            .duration_since(SystemTime::UNIX_EPOCH)
                            .unwrap_or_default()
                            .as_millis()
                            .to_string(),
                    );
                }
            }
        }

        Ok(map)
    }

    async fn get_file_tree_hashes(&self, dir: &str) -> VcsResult<BTreeMap<String, String>> {
        let mut map = BTreeMap::new();

        let output = self
            .run_command(
                &mut self.create_command(vec!["ls", "--recursive", "--depth", "infinity", dir]),
                false,
            )
            .await?;

        // svn doesnt support file hashing, so instead of generating some
        // random hash ourselves, just pass an emptry string.
        for file in output.split('\n') {
            map.insert(file.to_owned(), String::new());
        }

        Ok(map)
    }

    // https://svnbook.red-bean.com/en/1.8/svn.ref.svn.c.status.html
    async fn get_touched_files(&self) -> VcsResult<TouchedFiles> {
        let output = self
            .run_command(&mut self.create_command(vec!["status", "wc"]), false)
            .await?;

        Ok(Svn::process_touched_files(output))
    }

    #[track_caller]
    async fn get_touched_files_against_previous_revision(
        &self,
        revision: &str,
    ) -> VcsResult<TouchedFiles> {
        let number: usize = self.get_revision_number(revision).await?.parse().unwrap();

        // TODO: this is definitely not right...
        Ok(self
            .get_touched_files_between_revisions(&(number - 1).to_string(), &(number).to_string())
            .await?)
    }

    // https://svnbook.red-bean.com/en/1.8/svn.ref.svn.c.diff.html
    async fn get_touched_files_between_revisions(
        &self,
        base_revision: &str,
        revision: &str,
    ) -> VcsResult<TouchedFiles> {
        let output = self
            .run_command(
                &mut self.create_command(vec![
                    "diff",
                    "-r",
                    &format!("{}:{}", base_revision, revision),
                    "--summarize",
                ]),
                false,
            )
            .await?;

        Ok(Svn::process_touched_files(output))
    }

    fn is_default_branch(&self, branch: &str) -> bool {
        self.default_branch == branch
    }

    fn is_enabled(&self) -> bool {
        self.root.join(".svn").exists()
    }
}
