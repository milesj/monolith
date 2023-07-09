use crate::event::{Event, EventFlow};
use moon_workspace::Workspace;

#[async_trait::async_trait]
pub trait Subscriber: Send + Sync {
    async fn on_emit<'e>(
        &mut self,
        event: &Event<'e>,
        workspace: &Workspace,
    ) -> miette::Result<EventFlow>;
}
