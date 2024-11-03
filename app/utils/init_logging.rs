use std::{
    path::{Path, PathBuf},
    sync::{Arc, OnceLock},
};

use crate::discord;

use super::discord::EventFilters;
use debug_print::debug_println;
use tracing::{level_filters::LevelFilter, Subscriber};
use tracing_layer_core::layer::WebhookLayer;
use tracing_subscriber::{layer::SubscriberExt, EnvFilter, Layer, Registry};

pub use super::discord::BackgroundWorker; // Re-export for the shutdown function

pub static LOGGER: OnceLock<Arc<dyn Subscriber + Send + Sync>> =
    OnceLock::new();

/// This method initializes the logging system for the application.
/// The logs are written to the console and to a file in the specified directory.
///
/// # Returns
/// Discord [BackgroundWorker] to be used on graceful shutdown.
///
/// # Panics
/// When logging fails to initialize.
pub async fn init_logging(
    dir: &Path,
    log_severity: String,
    discord_hook: Option<String>,
) -> Option<BackgroundWorker> {
    debug_println!("\nInitializing logging...");

    // Initializing color_eyre for better error handling
    color_eyre::install().unwrap_or_default();

    // Appenders - the guards are needed for the lifetime of the program
    let dir = log_directory(dir).await; // Ensures directory exists

    // Filtering verbose crates
    let filtered = vec![];
    let env_filter = filter(&filtered, &log_severity);

    let file_appender = tracing_appender::rolling::daily(
        dir,
        concat!(env!("CARGO_PKG_NAME"), ".log"),
    );
    let (non_blocking_file, guard0) =
        tracing_appender::non_blocking(file_appender);
    Box::leak(Box::new(guard0));

    let (non_blocking_stdout, guard1) =
        tracing_appender::non_blocking(std::io::stdout());
    Box::leak(Box::new(guard1));

    // Default formatter
    let formatter = tracing_subscriber::fmt::format()
        .with_thread_ids(true)
        .with_thread_names(true);

    // Layers
    let file_layer = tracing_subscriber::fmt::layer()
        .event_format(formatter.clone().compact())
        .with_writer(non_blocking_file);
    let stdout_layer = tracing_subscriber::fmt::layer()
        .event_format(formatter.pretty())
        .with_writer(non_blocking_stdout);
    let (discord_layer, discord_worker) = init_discord(discord_hook).await;

    let layers = stdout_layer.and_then(file_layer);
    let layers = match discord_layer {
        Some(d) => d.and_then(layers).boxed(),
        None => layers.boxed(),
    };

    // Creates the subscriber and initialises
    let registry = Registry::default().with(layers.with_filter(env_filter));
    tracing::subscriber::set_global_default(
        LOGGER.get_or_init(|| Arc::new(registry)).clone(),
    )
    .unwrap();

    debug_println!("Success!");
    debug_println!("-------------------------------------------------------");
    discord_worker
}

async fn init_discord(
    discord_hook: Option<String>,
) -> (
    Option<WebhookLayer<discord::Layer>>,
    Option<BackgroundWorker>,
) {
    match discord_hook {
        None => {
            debug_println!("Discord webhook not found.");
            (None, None)
        }
        Some(hook) => {
            debug_println!("Discord webhook provided.");

            let (discord_layer, worker) = discord::Layer::builder(
                discord::Config::new(hook),
                env!("CARGO_PKG_NAME").to_string(),
                EventFilters::new(None, None),
            )
            .build();

            worker.start().await;
            (Some(discord_layer), Some(worker))
        }
    }
}

/// Creates the log directory (if doesn't exist) and returns its path.
async fn log_directory(dir: &Path) -> PathBuf {
    let canonical = super::canonicalize_unexistent(dir)
        .unwrap_or_else(|| panic!("Failed to canonicalize path!"));

    tokio::fs::create_dir_all(&canonical)
    .await
    .unwrap_or_else(|e| panic!("Failed to create canonical directory: {e}. Path: {canonical:?}"));

    canonical
}

/// Useful for filtering verbose crates
fn filter(filter_entries: &[&str], log_severity: &str) -> EnvFilter {
    #[expect(clippy::unwrap_used)] // Safe because it's a constant
    let filter = EnvFilter::builder()
        .with_default_directive(
            log_severity.parse::<LevelFilter>().unwrap().into(),
        )
        .from_env()
        .unwrap_or_else(|e| {
            panic!("Invalid directives for tracing subscriber: {e}.")
        });

    filter_entries.iter().fold(filter, |acc, s| {
        acc.add_directive(format!("{s}=warn").parse().unwrap())
    })
}
