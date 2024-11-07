use std::{
    collections::{HashMap, HashSet},
    fmt::{Display, Write},
};
use sysinfo::{Networks, RefreshKind, System};
use tokio::process::Command;
use tracing::info;

pub struct SystemMetrics {
    system: System,
    networks: Networks,
}

pub struct NetworkMetrics {
    pub bytes_received: u64,
    pub bytes_transmitted: u64,
    pub receive_error_percentage: f64,
    pub transmit_error_percentage: f64,
}

/// All units are in bytes
pub struct MetricsData {
    pub cpu: f32,
    pub memory: u64,
    pub disk: HashMap<Box<str>, (f64, f64)>,
    pub network: HashMap<Box<str>, NetworkMetrics>,
}
pub struct MetricsDataFormatted {
    pub cpu: String,
    pub memory: String,
    pub disk: String,
    pub network: String,
}
impl From<MetricsData> for MetricsDataFormatted {
    fn from(metrics: MetricsData) -> Self {
        let cpu_usage = format!("{:.2}%", metrics.cpu);
        #[allow(clippy::cast_precision_loss)]
        let used_memory = format!("{:.2}", fmt_unit(metrics.memory as f64));
        let disk_usage = metrics.disk.into_iter().fold(
            String::new(),
            |mut acc, (name, (used, available))| {
                let _ = write!(
                    acc,
                    "\n- {}: {:.2}/{:.2} ({:.2}%)",
                    name,
                    fmt_unit(used),
                    fmt_unit(available),
                    used * 100.0 / (used + available)
                );
                acc
            },
        );
        let network_usage = metrics
            .network
            .iter()
            .fold(String::new(), |mut acc, (name, data)| {
                #[allow(clippy::cast_precision_loss)]
                let received = fmt_unit_net(data.bytes_received as f64);
                #[allow(clippy::cast_precision_loss)]
                let transmitted = fmt_unit_net(data.bytes_transmitted as f64);
                let _ = write!(
                    acc,
                    "\n- {}: {} received with {}% errors, {} transmitted with {}% errors",
                    name,
                    received,
                    data.receive_error_percentage,
                    transmitted,
                    data.transmit_error_percentage
                );
                acc
            });
        Self {
            cpu: cpu_usage,
            memory: used_memory,
            disk: disk_usage,
            network: network_usage,
        }
    }
}
impl Display for MetricsDataFormatted {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "**CPU Usage:** {}\n**Memory Usage:** {}\n**Disk Usage:** {}\n**Network Usage:** {}",
            self.cpu, self.memory, self.disk, self.network
        )
    }
}

impl SystemMetrics {
    pub fn new() -> Self {
        Self {
            system: System::new_with_specifics(
                RefreshKind::new().without_processes(),
            ),
            networks: Networks::new(),
        }
    }

    pub async fn collect(&mut self) -> MetricsData {
        self.system.refresh_all();

        let cpu_usage = self.system.global_cpu_usage();
        let used_memory_bytes = self.system.used_memory();
        let disk_usage = self.collect_disk_usage().await;
        let network_usage = {
            let mut set = HashSet::new(); // TODO: Read from config
            set.insert("enp5s0".to_string());
            self.collect_network_usage(&set)
        };

        MetricsData {
            cpu: cpu_usage,
            memory: used_memory_bytes,
            disk: disk_usage,
            network: network_usage,
        }
    }

    fn collect_network_usage(
        &mut self,
        interfaces: &HashSet<String>,
    ) -> HashMap<Box<str>, NetworkMetrics> {
        self.networks.refresh_list();
        self.networks
            .list()
            .iter()
            .filter_map(|(name, data)| {
                if !interfaces.is_empty() && !interfaces.contains(name) {
                    return None;
                }

                let metrics = NetworkMetrics {
                    bytes_received: data.received(),
                    bytes_transmitted: data.transmitted(),
                    #[allow(clippy::cast_precision_loss)]
                    receive_error_percentage: data.errors_on_received() as f64
                        * 100f64
                        / data.packets_received() as f64,
                    #[allow(clippy::cast_precision_loss)]
                    transmit_error_percentage: data.errors_on_transmitted()
                        as f64
                        * 100f64
                        / data.packets_transmitted() as f64,
                };
                Some((Box::from(name.clone()), metrics))
            })
            .collect()
    }

    async fn collect_disk_usage(&self) -> HashMap<Box<str>, (f64, f64)> {
        let mut disk_usage_map = HashMap::new();

        let output = Command::new("df")
            .arg("-B1")
            .arg("-P")
            .arg("/dev/sda1")
            .output()
            .await
            .expect("Failed to execute df command");
        if !output.status.success() {
            return disk_usage_map;
        }

        let output_str = String::from_utf8_lossy(&output.stdout);
        let mut lines = output_str.lines();

        lines.next(); // Skip header
        if let Some(line) = lines.next() {
            let fields: Vec<&str> = line.split_whitespace().collect();
            if fields.len() >= 4 {
                let used_bytes = fields[2].parse::<f64>().unwrap_or(0.0);
                let available_bytes = fields[3].parse::<f64>().unwrap_or(0.0);
                disk_usage_map.insert(
                    Box::from("/dev/sda1"),
                    (used_bytes, available_bytes),
                );
            }
        }
        disk_usage_map
    }
}

fn fmt_unit(bytes: f64) -> String {
    info!("Formatting {} bytes", bytes);
    let current = 1_024.0;
    let next = |mut c: f64| {
        c *= 1_024.0;
        c
    };
    if bytes < current {
        return format!("{bytes:.2}B");
    }
    if bytes < next(current) {
        return format!("{:.2}KB", bytes / current);
    }
    if bytes < next(current) {
        return format!("{:.2}MB", bytes / current);
    }
    if bytes < next(current) {
        return format!("{:.2}GB", bytes / current);
    }
    format!("{:.2}TB", bytes / next(current))
}

fn fmt_unit_net(bytes: f64) -> String {
    let bits = bytes * 8.0;
    let current = 1_024.0;
    let next = |mut c: f64| {
        c *= 1_024.0;
        c
    };
    if bits < current {
        return format!("{bits:.2}bps");
    }
    if bits < next(current) {
        return format!("{:.2}Kbps", bits / current);
    }
    if bits < next(current) {
        return format!("{:.2}Mbps", bits / current);
    }
    if bits < next(current) {
        return format!("{:.2}Gbps", bits / current);
    }
    format!("{:.2}Tbps", bytes / next(current))
}
