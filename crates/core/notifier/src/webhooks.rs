use ci_env::{get_ci_environment, CiEnvironment};
use moon_emitter::{Event, EventFlow, Subscriber};
use moon_error::MoonError;
use moon_logger::{color, error, trace};
use moon_utils::time::{chrono::prelude::*, now_timestamp};
use moon_workspace::Workspace;
use serde::{Deserialize, Serialize};
use tokio::task::JoinHandle;
use uuid::Uuid;

const LOG_TARGET: &str = "moon:notifier:webhooks";

#[derive(Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct WebhookPayload<T: Serialize> {
    pub created_at: NaiveDateTime,

    pub environment: Option<CiEnvironment>,

    // Only for testing!
    #[serde(skip_deserializing)]
    pub event: T,

    #[serde(rename = "type")]
    pub type_of: String,

    pub uuid: String,
}

pub async fn notify_webhook(
    url: String,
    body: String,
) -> Result<reqwest::Response, reqwest::Error> {
    reqwest::Client::new()
        .post(url)
        .body(body)
        .header("Accept", "application/json")
        .header("Content-Type", "application/json")
        .header("Connection", "keep-alive")
        .header("Keep-Alive", "timeout=30, max=120")
        .send()
        .await
}

pub struct WebhooksSubscriber {
    enabled: bool,
    environment: Option<CiEnvironment>,
    requests: Vec<JoinHandle<()>>,
    url: String,
    uuid: String,
}

impl WebhooksSubscriber {
    pub fn new(url: String) -> Self {
        WebhooksSubscriber {
            enabled: true,
            environment: get_ci_environment(),
            requests: vec![],
            uuid: if url.contains("127.0.0.1") {
                "XXXX-XXXX-XXXX-XXXX".into()
            } else {
                Uuid::new_v4().to_string()
            },
            url,
        }
    }
}

#[async_trait::async_trait]
impl Subscriber for WebhooksSubscriber {
    async fn on_emit<'a>(
        &mut self,
        event: &Event<'a>,
        _workspace: &Workspace,
    ) -> Result<EventFlow, MoonError> {
        if !self.enabled {
            return Ok(EventFlow::Continue);
        }

        let payload = WebhookPayload {
            created_at: now_timestamp(),
            environment: self.environment.clone(),
            event,
            type_of: event.get_type(),
            uuid: self.uuid.clone(),
        };

        trace!(
            target: LOG_TARGET,
            "Posting event {} to webhook endpoint",
            color::id(&payload.type_of),
        );

        let body = serde_json::to_string(&payload).unwrap();

        // For the first event, we want to ensure that the webhook URL is valid
        // by sending the request and checking for a failure. If failed,
        // we will disable subsequent requests from being called.
        if matches!(event, Event::PipelineStarted { .. }) {
            let response = notify_webhook(self.url.to_owned(), body).await;

            if response.is_err() || !response.unwrap().status().is_success() {
                self.enabled = false;

                error!(
                    target: LOG_TARGET,
                    "Failed to send webhook event to {}. Subsequent webhook requests will be disabled.",
                    color::url(&self.url),
                );
            }

            // For every other event, we will make the request and ignore the result.
            // We will also avoid awaiting the request to not slow down the overall runner.
        } else {
            let url = self.url.to_owned();

            self.requests.push(tokio::spawn(async {
                let _ = notify_webhook(url, body).await;
            }));
        }

        // For the last event, we want to ensure that all webhook requests have
        // actually sent, otherwise, when the program exists all of these requests
        // will be dropped!
        if event.is_end() {
            for future in self.requests.drain(0..) {
                let _ = future.await;
            }
        }

        Ok(EventFlow::Continue)
    }
}
