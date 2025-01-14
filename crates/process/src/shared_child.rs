use crate::signal::*;
use std::io;
use std::process::{ExitStatus, Output};
use std::sync::Arc;
use tokio::process::{Child, ChildStderr, ChildStdin, ChildStdout};
use tokio::sync::Mutex;

#[derive(Clone)]
pub struct SharedChild {
    inner: Arc<Mutex<Option<Child>>>,
    pid: u32,
}

impl SharedChild {
    pub fn new(child: Child) -> Self {
        Self {
            pid: child.id().unwrap(),
            inner: Arc::new(Mutex::new(Some(child))),
        }
    }

    pub fn id(&self) -> u32 {
        self.pid
    }

    pub async fn take_stdin(&self) -> Option<ChildStdin> {
        self.inner
            .lock()
            .await
            .as_mut()
            .and_then(|child| child.stdin.take())
    }

    pub async fn take_stdout(&self) -> Option<ChildStdout> {
        self.inner
            .lock()
            .await
            .as_mut()
            .and_then(|child| child.stdout.take())
    }

    pub async fn take_stderr(&self) -> Option<ChildStderr> {
        self.inner
            .lock()
            .await
            .as_mut()
            .and_then(|child| child.stderr.take())
    }

    pub async fn kill(&self) -> io::Result<()> {
        let mut child = self.inner.lock().await;

        if let Some(mut child) = child.take() {
            child.kill().await?;
        }

        Ok(())
    }

    pub async fn kill_with_signal(&self, signal: SignalType) -> io::Result<()> {
        let mut child = self.inner.lock().await;

        if let Some(mut child) = child.take() {
            // https://github.com/rust-lang/rust/blob/master/library/std/src/sys/pal/unix/process/process_unix.rs#L947
            #[cfg(unix)]
            {
                kill(self.id(), signal)?;
            }

            // https://github.com/rust-lang/rust/blob/master/library/std/src/sys/pal/windows/process.rs#L658
            #[cfg(windows)]
            {
                child.start_kill().await?;
            }

            child.wait().await?;
        }

        Ok(())
    }

    pub async fn wait(&self) -> io::Result<ExitStatus> {
        let mut child = self.inner.lock().await;

        if let Some(child) = child.as_mut() {
            return child.wait().await;
        }

        unreachable!()
    }

    pub async fn wait_with_output(&self) -> io::Result<Output> {
        let mut child = self.inner.lock().await;

        if let Some(child) = child.take() {
            return child.wait_with_output().await;
        }

        unreachable!()
    }
}
