use serde::Deserialize;
use std::collections::{HashMap, HashSet};
use std::time::Duration;
use tokio::fs;

#[derive(Debug, Deserialize)]
pub struct Config {
    pub webhook_url: String,
    pub embed_title: String,
    pub embed_color: String,
    pub update_interval: String,
    pub optional_message: String,
    pub user_tags: Vec<String>,
    pub show_memory: bool,
    pub memory_in_mb: bool,
    pub show_cpu: bool,
    pub show_network_usage: bool,
    pub network_interfaces: HashSet<String>,
    pub optional_message_enabled: bool,
    pub user_tags_enabled: bool,
    pub update_previous_message: bool,
    pub message_id: Option<String>,
    pub show_disk_usage: bool,
    pub disk_drives: HashSet<String>,
    pub disk_names: HashMap<String, String>,
    pub ssh_alerts: SshAlertsConfig,
}

#[derive(Debug, Deserialize)]
pub struct SshAlertsConfig {
    pub enabled: bool,
    pub log_path: String,
    pub ssh_alert_webhook_url: String,
}

impl Config {
    pub async fn load(path: &str) -> Self {
        let config_content = fs::read_to_string(path)
            .await
            .expect("Failed to read config.toml");
        toml::from_str(&config_content).expect("Failed to parse config.toml")
    }

    pub fn get_embed_color(&self) -> Result<u32, std::num::ParseIntError> {
        u32::from_str_radix(&self.embed_color.trim_start_matches('#'), 16)
    }

    pub fn update_interval(&self) -> Duration {
        humantime::parse_duration(&self.update_interval)
            .expect("Failed to parse update interval from configuration")
    }
}
