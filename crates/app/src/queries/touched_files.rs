use miette::IntoDiagnostic;
use moon_common::is_ci;
use moon_common::path::{standardize_separators, WorkspaceRelativePathBuf};
use moon_vcs::{BoxedVcs, TouchedStatus};
use rustc_hash::FxHashSet;
use serde::{Deserialize, Serialize};
use starbase_styles::color;
use starbase_utils::json;
use std::env;
use std::io::{stdin, IsTerminal, Read};
use tracing::{debug, trace, warn};

#[derive(Clone, Default, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct QueryTouchedFilesOptions {
    pub base: Option<String>,
    pub default_branch: bool,
    pub head: Option<String>,
    pub json: bool,
    pub local: bool,
    pub status: Vec<TouchedStatus>,
}

#[derive(Default, Deserialize, Serialize)]
pub struct QueryTouchedFilesResult {
    pub files: FxHashSet<WorkspaceRelativePathBuf>,
    pub options: QueryTouchedFilesOptions,
    pub shallow: bool,
}

// If we're in a shallow checkout, many diff commands will fail
macro_rules! check_shallow {
    ($vcs:ident) => {
        if $vcs.is_shallow_checkout().await? {
            warn!("Detected a shallow checkout, unable to run Git commands to determine touched files.");

            if is_ci() {
                warn!("A full Git history is required for affected checks, falling back to an empty files list.");
            } else {
                warn!("A full Git history is required for affected checks, disabling for now.");
            }

            let mut result = QueryTouchedFilesResult::default();
            result.shallow = true;

            return Ok(result);
        }
    };
}

/// Query a list of files that have been modified between branches.
pub async fn query_touched_files(
    vcs: &BoxedVcs,
    options: &QueryTouchedFilesOptions,
) -> miette::Result<QueryTouchedFilesResult> {
    debug!("Querying for touched files");

    let default_branch = vcs.get_default_branch().await?;
    let current_branch = vcs.get_local_branch().await?;
    let base_revision = env::var("MOON_BASE").ok().or(options.base.clone());
    let head_revision = env::var("MOON_HEAD").ok().or(options.head.clone());

    // Check locally touched files
    let touched_files_map = if options.local {
        trace!("Against local");

        vcs.get_touched_files().await?
    }
    // Otherwise compare against remote
    else if base_revision.is_none()
        && options.default_branch
        && vcs.is_default_branch(&current_branch)
    {
        // Since base is not set, ensure we're not in a
        // shallow checkout
        check_shallow!(vcs);

        trace!(
            "Against previous revision, as we're on the default branch \"{}\"",
            current_branch
        );

        vcs.get_touched_files_against_previous_revision(&default_branch)
            .await?
    } else {
        // Don't check for shallow since base is set,
        // and we can assume the user knows what they're doing
        if base_revision.is_none() {
            check_shallow!(vcs);
        }

        let base = base_revision.as_deref().unwrap_or(&default_branch);
        let head = head_revision.as_deref().unwrap_or("HEAD");

        trace!(
            "Against remote using base \"{}\" with head \"{}\"",
            base,
            head,
        );

        vcs.get_touched_files_between_revisions(base, head).await?
    };

    let mut touched_files = FxHashSet::default();

    if options.status.is_empty() {
        debug!(
            "Filtering based on touched status {}",
            color::symbol(TouchedStatus::All.to_string())
        );

        touched_files.extend(touched_files_map.all());
    } else {
        debug!(
            "Filtering based on touched status {}",
            options
                .status
                .iter()
                .map(|f| color::symbol(f.to_string()))
                .collect::<Vec<_>>()
                .join(", ")
        );

        for status in &options.status {
            touched_files.extend(match status {
                TouchedStatus::Added => touched_files_map.added.iter().collect(),
                TouchedStatus::All => touched_files_map.all(),
                TouchedStatus::Deleted => touched_files_map.deleted.iter().collect(),
                TouchedStatus::Modified => touched_files_map.modified.iter().collect(),
                TouchedStatus::Staged => touched_files_map.staged.iter().collect(),
                TouchedStatus::Unstaged => touched_files_map.unstaged.iter().collect(),
                TouchedStatus::Untracked => touched_files_map.untracked.iter().collect(),
            });
        }
    }

    let touched_files: FxHashSet<WorkspaceRelativePathBuf> = touched_files
        .iter()
        .map(|f| WorkspaceRelativePathBuf::from(standardize_separators(f)))
        .collect();

    if !touched_files.is_empty() {
        debug!(
            files = ?touched_files.iter().map(|f| f.as_str()).collect::<Vec<_>>(),
            "Found touched files",
        );
    }

    Ok(QueryTouchedFilesResult {
        files: touched_files,
        options: options.to_owned(),
        shallow: false,
    })
}

pub async fn load_touched_files(
    vcs: &BoxedVcs,
) -> miette::Result<FxHashSet<WorkspaceRelativePathBuf>> {
    let mut buffer = String::new();

    // Only read piped data when stdin is not a TTY,
    // otherwise the process will hang indefinitely waiting for EOF.
    if !stdin().is_terminal() {
        stdin().read_to_string(&mut buffer).into_diagnostic()?;
    }

    // If piped via stdin, parse and use it
    if !buffer.is_empty() {
        // As JSON
        if buffer.starts_with('{') {
            let result: QueryTouchedFilesResult = json::parse(&buffer)?;

            return Ok(result.files);
        }
        // As lines
        else {
            let files =
                FxHashSet::from_iter(buffer.split('\n').map(WorkspaceRelativePathBuf::from));

            return Ok(files);
        }
    }

    let result = query_touched_files(
        vcs,
        &QueryTouchedFilesOptions {
            local: !is_ci(),
            ..QueryTouchedFilesOptions::default()
        },
    )
    .await?;

    Ok(result.files)
}