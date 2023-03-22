use moon_dep_graph::DepGraphError;
use moon_enforcer::EnforcerError;
use moon_error::MoonError;
use moon_project::ProjectError;
use moon_runner::RunnerError;
use moon_target::TargetError;
use moon_tool::ToolError;
use moon_workspace::WorkspaceError;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum PipelineError {
    #[error("{0}")]
    Aborted(String),

    #[error("An unknown action was encountered in the pipeline. Unable to proceed!")]
    UnknownActionNode,

    #[error(transparent)]
    DepGraph(#[from] DepGraphError),

    #[error(transparent)]
    Enforcer(#[from] EnforcerError),

    #[error(transparent)]
    Moon(#[from] MoonError),

    #[error(transparent)]
    Project(#[from] ProjectError),

    #[error(transparent)]
    Runner(#[from] RunnerError),

    #[error(transparent)]
    Target(#[from] TargetError),

    #[error(transparent)]
    Tool(#[from] ToolError),

    #[error(transparent)]
    Workspace(#[from] WorkspaceError),
}
