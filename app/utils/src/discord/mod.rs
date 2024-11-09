mod config;
pub use config::Config;

mod payload;
use payload::Payload;

use nexus_api::tracing::Level;
pub use tracing_layer_core::filters::EventFilters;
pub use tracing_layer_core::layer::WebhookLayer;
use tracing_layer_core::layer::WebhookLayerBuilder;
pub use tracing_layer_core::BackgroundWorker;
use tracing_layer_core::{
    WebhookMessage, WebhookMessageFactory, WebhookMessageInputs,
};

const MAX_FIELD_VALUE_CHARS: usize = 1024 - 15;
const MAX_ERROR_MESSAGE_CHARS: usize = 2048 - 15;

pub struct Layer {
    config: Config,
}
impl Layer {
    #[must_use]
    pub fn builder(
        config: Config,
        app_name: String,
        target_filters: EventFilters,
    ) -> WebhookLayerBuilder<Self> {
        WebhookLayerBuilder::new(Self { config }, app_name, target_filters)
    }
}

impl WebhookMessageFactory for Layer {
    fn create(
        &self,
        WebhookMessageInputs {
            app_name,
            mut message,
            target,
            span,
            metadata,
            source_line,
            source_file,
            event_level,
        }: WebhookMessageInputs,
    ) -> Box<dyn WebhookMessage> {
        if message.chars().count() > MAX_ERROR_MESSAGE_CHARS {
            println!(
                "Truncating message to {MAX_ERROR_MESSAGE_CHARS} characters, original: {message}"
            );
            message = message.chars().take(MAX_ERROR_MESSAGE_CHARS).collect();
        }

        let emoji = emoji_from_level(event_level);
        let color = color_from_level(event_level);
        let mut embed = serde_json::json!({
            "title": format!("{} - {} {}", app_name, emoji, event_level),
            "description": format!("```rust\n{}\n```", message),
            "fields": [
                // {
                //     "name": "Target Span",
                //     "value": format!("`{}::{}`", target, span),
                //     "inline": true
                // },
                // {
                //     "name": "Source",
                //     "value": format!("`{}#L{}`", source_file, source_line),
                //     "inline": true
                // },
            ],
            "footer": {
                "text": format!("@ {}#L{}, {}::{}", source_file, source_line, target, span)
            },
            "color": color,
        });
        let mut skipped = 0;
        let mut fields = into_chunks(&metadata, MAX_FIELD_VALUE_CHARS)
            .into_iter()
            .enumerate()
            .filter_map(|(i, chunk)| {
                if chunk == "{}" {
                    skipped += 1;
                    return None;
                }
                Some(serde_json::json!({
                    "name": format!("Metadata ({})", i - skipped),
                    "value": format!("```json\n{}\n```", chunk),
                    "inline": false
                }))
            })
            .collect();
        embed["fields"].as_array_mut().unwrap().append(&mut fields);

        Box::new(Payload {
            content: match event_level {
                Level::ERROR => Some("@everyone".to_string()),
                Level::WARN => Some("@here".to_string()),
                _ => None,
            },
            embeds: Some(vec![embed]),
            webhook_url: self.config.webhook_url.clone(),
        })
    }
}

const fn color_from_level(level: Level) -> i32 {
    match level {
        Level::DEBUG | Level::TRACE => 1_752_220,
        Level::INFO => 5_763_719,
        Level::WARN => 15_105_570,
        Level::ERROR => 15_548_997,
    }
}

const fn emoji_from_level(level: Level) -> &'static str {
    match level {
        Level::TRACE => ":mag:",
        Level::DEBUG => ":bug:",
        Level::INFO => ":information_source:",
        Level::WARN => ":warning:",
        Level::ERROR => ":x:",
    }
}

fn into_chunks(s: &str, max_size: usize) -> Vec<&str> {
    let len = s.len();
    let chunk_n = (len / max_size) + usize::from(len % max_size != 0);
    let mut chunks = Vec::with_capacity(chunk_n);
    for i in 0..chunk_n - 1 {
        chunks.push(&s[i * max_size..(i + 1) * max_size]);
    }
    chunks.push(&s[(chunk_n - 1) * max_size..len]);
    chunks
}
