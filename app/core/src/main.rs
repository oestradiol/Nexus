mod loader;
mod on_shutdown;

use std::path::PathBuf;
use std::sync::Arc;
use tokio::fs;
use tokio::io::AsyncReadExt;

use loader::PluginInstance;
use nexus_utils::api::TokioRuntimeHandle;
use tracing::info;
use on_shutdown::with_graceful_shutdown;

// #[cfg(not(target_env = "msvc"))]
// #[global_allocator]
// static GLOBAL: mimalloc::MiMalloc = mimalloc::MiMalloc;

#[tokio::main]
async fn main() {
    let discord_worker = {
        let path = PathBuf::from("./Logs");
        let discord_hook = Some("https://discord.com/api/webhooks/1302403792595320932/uRKBpnrO0QHW2av73HubxxOgIsOAUPNyrcRFvwwrvBYpxRZMvy-Ycj5I2jZ1AFbr2-OI".to_string());
        nexus_utils::init_logging(&path, "INFO".to_string(), discord_hook).await
    };

    // TODO
    // Create runtime handle for plugins
    let runtime_handle = Arc::new(TokioRuntimeHandle::new(tokio::runtime::Handle::current()));
    // Scan and load all plugins from ./plugins/ directory
    let plugin_dir = PathBuf::from("./plugins");
    let mut entries = match fs::read_dir(&plugin_dir).await {
        Ok(e) => e,
        Err(_) => {
            info!("No plugins directory found at {:?}", plugin_dir);
            return with_graceful_shutdown(discord_worker).await;
        }
    };

    while let Some(entry) = entries.next_entry().await.ok().flatten() {
        let path = entry.path();
        
        // Check if file is an ELF shared library (ET_DYN) not executable (ET_EXEC)
        const ET_DYN: u16 = 3;
        let is_so = match fs::File::open(&path).await {
            Ok(mut f) => {
                let mut header = [0u8; 18];
                match f.read_exact(&mut header).await {
                    Ok(_) => {
                        // Check ELF magic and e_type at offset 16 (32-bit) or read proper ELF64 header
                        let is_elf = &header[0..4] == &[0x7f, b'E', b'L', b'F'];
                        // e_type is at offset 16 for both 32/64-bit ELF
                        let e_type = u16::from_le_bytes([header[16], header[17]]);
                        is_elf && e_type == ET_DYN
                    }
                    Err(_) => false,
                }
            }
            Err(_) => false,
        };
        
        if !is_so {
            continue;
        }
        // Get the plugin name from the file name
        let Some(plugin_name) = path.file_name().and_then(|n| n.to_str()).map(String::from) else {
            continue;
        };

        let path_for_spawn = path.clone();
        let runtime_clone = Arc::clone(&runtime_handle);
        tokio::spawn(async move {
            match PluginInstance::new(&path_for_spawn, runtime_clone) {
                Ok(instance) => {
                    instance.plugin.main().await;
                }
                Err(e) => {
                    tracing::error!("Failed to load plugin {}: {}", plugin_name, e);
                }
            }
        });
    }

    with_graceful_shutdown(discord_worker).await;
}
