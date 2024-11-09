use crate::config::Config;
use std::sync::Arc;
use std::time::{SystemTime, UNIX_EPOCH};
use tokio::io::{AsyncBufReadExt, BufReader};
use tokio::process::Command;
use tokio::sync::Mutex;
use tokio::task;

pub struct AuditMonitor {}

impl AuditMonitor {
    pub fn new(config: Arc<Config>, notifier: Arc<Notifier>) -> Self {
        AuditMonitor { config, notifier }
    }

    pub async fn start(&self) {
        let config = Arc::clone(&self.config);
        let notifier = Arc::clone(&self.notifier);
        task::spawn(async move {
            let last_login_details = Arc::new(Mutex::new((None, 0u64)));
            monitor_ssh_logins(&config, &notifier, last_login_details).await;
        });
    }
}

async fn monitor_ssh_logins(
    config: &Arc<Config>,
    notifier: &Arc<Notifier>,
    last_login_details: Arc<Mutex<(Option<SshLoginDetails>, u64)>>,
) {
    let log_path = &config.ssh_alerts.log_path;

    let mut child = Command::new("tail")
        .arg("-F")
        .arg(log_path)
        .stdout(std::process::Stdio::piped())
        .spawn()
        .expect("Failed to execute tail command");

    let stdout = child.stdout.take().expect("Failed to capture stdout");
    let reader = BufReader::new(stdout);

    let mut lines = reader.lines();

    while let Some(line) = lines.next_line().await.unwrap_or(None) {
        if line.contains("Accepted password for")
            || line.contains("Accepted publickey for")
        {
            if let Some(details) = parse_ssh_login_details(&line) {
                let should_send = {
                    let mut last_login = last_login_details.lock().await;
                    let current_time = SystemTime::now()
                        .duration_since(UNIX_EPOCH)
                        .unwrap()
                        .as_secs();
                    if *last_login != (Some(details.clone()), current_time) {
                        if current_time - last_login.1 >= 2 {
                            *last_login = (Some(details.clone()), current_time);
                            true
                        } else {
                            false
                        }
                    } else {
                        false
                    }
                }; // MutexGuard dropped here

                if should_send {
                    notifier.send_ssh_login_embed(details.clone()).await;
                }
            }
        }
    }
}

fn parse_ssh_login_details(log_line: &str) -> Option<SshLoginDetails> {
    let parts: Vec<&str> = log_line.split_whitespace().collect();
    if parts.len() >= 11 {
        let time = format!("{} {} {}", parts[0], parts[1], parts[2]);
        let user = parts[8].to_string();
        let ip = parts[10].to_string();
        Some(SshLoginDetails { user, ip, time })
    } else {
        None
    }
}
