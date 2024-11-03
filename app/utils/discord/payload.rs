use serde::Serialize;
use serde_json::Value;
use tracing_layer_core::WebhookMessage;

#[derive(Debug, Serialize)]
pub(super) struct Payload {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub(super) content: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub(super) embeds: Option<Vec<Value>>,
    #[serde(skip_serializing)]
    pub(super) webhook_url: String,
}
impl WebhookMessage for Payload {
    fn webhook_url(&self) -> &str {
        self.webhook_url.as_str()
    }

    fn serialize(&self) -> String {
        serde_json::to_string(self)
            .expect("Failed to serialize Discord Payload!")
    }
}
