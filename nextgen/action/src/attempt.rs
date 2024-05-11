use crate::action::ActionStatus;
use moon_time::chrono::NaiveDateTime;
use moon_time::now_timestamp;
use serde::{Deserialize, Serialize};
use std::mem;
use std::process::Output;
use std::sync::Arc;
use std::time::{Duration, Instant};

#[derive(Debug, Default, Deserialize, Serialize)]
#[serde(rename_all = "kebab-case")]
pub enum AttemptType {
    ArchiveCreation,
    HashGeneration,
    MutexAcquisition,
    NoOperation,
    OutputHydration,
    #[default]
    TaskExecution,
}

#[derive(Debug, Default, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct AttemptExecution {
    pub exit_code: Option<i32>,

    pub stderr: Option<Arc<String>>,

    pub stdout: Option<Arc<String>>,
}

impl AttemptExecution {
    pub fn set_stderr(&mut self, output: String) {
        if !output.is_empty() {
            self.stderr = Some(Arc::new(output));
        }
    }

    pub fn set_stdout(&mut self, output: String) {
        if !output.is_empty() {
            self.stdout = Some(Arc::new(output));
        }
    }
}

#[derive(Debug, Default, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Attempt {
    pub duration: Option<Duration>,

    pub execution: Option<AttemptExecution>,

    pub finished_at: Option<NaiveDateTime>,

    pub started_at: NaiveDateTime,

    #[serde(skip)]
    pub start_time: Option<Instant>,

    pub status: ActionStatus,

    #[serde(rename = "type")]
    pub type_of: AttemptType,
}

impl Attempt {
    pub fn new(type_of: AttemptType) -> Self {
        Attempt {
            duration: None,
            execution: None,
            finished_at: None,
            started_at: now_timestamp(),
            start_time: Some(Instant::now()),
            status: ActionStatus::Running,
            type_of,
        }
    }

    pub fn new_finished(type_of: AttemptType, status: ActionStatus) -> Self {
        let time = now_timestamp();

        Attempt {
            duration: None,
            execution: None,
            finished_at: Some(time),
            started_at: time,
            start_time: None,
            status,
            type_of,
        }
    }

    pub fn get_exit_code(&self) -> i32 {
        self.execution
            .as_ref()
            .and_then(|exec| exec.exit_code)
            .unwrap_or(-1)
    }

    pub fn get_last_execution(attempts: &[Attempt]) -> Option<&Attempt> {
        attempts
            .iter()
            .rfind(|attempt| matches!(attempt.type_of, AttemptType::TaskExecution))
    }

    pub fn get_last_failed_execution(attempts: &[Attempt]) -> Option<&Attempt> {
        attempts.iter().rfind(|attempt| {
            attempt.has_failed() && matches!(attempt.type_of, AttemptType::TaskExecution)
        })
    }

    pub fn finish(&mut self, status: ActionStatus) {
        self.finished_at = Some(now_timestamp());
        self.status = status;

        if let Some(start) = &self.start_time {
            self.duration = Some(start.elapsed());
        }
    }

    pub fn finish_from_output(&mut self, output: &mut Output) {
        let mut execution = AttemptExecution {
            exit_code: output.status.code(),
            ..Default::default()
        };

        execution.set_stderr(String::from_utf8(mem::take(&mut output.stderr)).unwrap_or_default());

        execution.set_stdout(String::from_utf8(mem::take(&mut output.stdout)).unwrap_or_default());

        self.execution = Some(execution);

        self.finish(if output.status.success() {
            ActionStatus::Passed
        } else {
            ActionStatus::Failed
        });
    }

    pub fn has_failed(&self) -> bool {
        matches!(
            &self.status,
            ActionStatus::Failed | ActionStatus::FailedAndAbort
        )
    }

    pub fn has_passed(&self) -> bool {
        matches!(
            &self.status,
            ActionStatus::Cached | ActionStatus::CachedFromRemote | ActionStatus::Passed
        )
    }

    pub fn has_output(&self) -> bool {
        self.execution.as_ref().is_some_and(|exec| {
            exec.stderr.as_ref().is_some_and(|err| !err.is_empty())
                || exec.stdout.as_ref().is_some_and(|out| !out.is_empty())
        })
    }

    pub fn is_cached(&self) -> bool {
        matches!(
            &self.status,
            ActionStatus::Cached | ActionStatus::CachedFromRemote
        )
    }
}
