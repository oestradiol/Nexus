// src/main.rs

mod loader;
mod on_shutdown;

use std::path::PathBuf;

use loader::PluginInstance;
use on_shutdown::with_graceful_shutdown;
use tracing::debug;

#[tokio::main]
async fn main() {
    let discord_worker = {
        let path = PathBuf::from("./Logs");
        nexus_utils::init_logging(&path, "INFO".to_string(), Some("https://discord.com/api/webhooks/1302403792595320932/uRKBpnrO0QHW2av73HubxxOgIsOAUPNyrcRFvwwrvBYpxRZMvy-Ycj5I2jZ1AFbr2-OI".to_string())).await
    };

    debug!("Starting the plugin...");

    tokio::spawn(async move {
        PluginInstance::new("./target/debug/libnexus_example.so")
            .unwrap()
            .plugin
            .start()
            .await;
    });

    with_graceful_shutdown(discord_worker).await;
}
