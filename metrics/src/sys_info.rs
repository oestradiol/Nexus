use std::{
    collections::{HashMap, HashSet},
    fmt::Display,
};
use sysinfo::{Disks, Networks, RefreshKind, System};

use crate::DELAY_SECS;

pub struct SysInfo {
    system: System,
    networks: Networks,
    disks: Disks,
}

pub struct NetworkMetrics {
    pub bytes_received: f64,    // bytes per second
    pub bytes_transmitted: f64, // bytes per second
    pub received_error_percentage: f64,
    pub transmit_error_percentage: f64,
}
pub struct MemoryMetrics {
    pub used: u64,  // bytes
    pub total: u64, // bytes
}
/// All units are in bytes
pub struct Metrics {
    pub cpu: f32,
    pub ram: MemoryMetrics, // bytes
    pub disks: Option<HashMap<Box<str>, MemoryMetrics>>,
    pub net_interfaces: Option<HashMap<Box<str>, NetworkMetrics>>,
}
impl Display for Metrics {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        // CPU
        write!(f, "**CPU:** {:.2}%", self.cpu)?;

        // RAM
        #[allow(clippy::cast_precision_loss)]
        let used = self.ram.used as f64;
        #[allow(clippy::cast_precision_loss)]
        let total = self.ram.total as f64;
        if total != 0.0 {
            write!(
                f,
                "\n**Memory:** {}/{} ({:.2}%)",
                fmt_unit(used),
                fmt_unit(total),
                100.0 * used / total
            )?;
        }

        // Disk
        if let Some(disks) = &self.disks {
            write!(f, "\n**Disks:**")?;
            for (name, MemoryMetrics { used, total }) in disks {
                #[allow(clippy::cast_precision_loss)]
                let used = *used as f64;
                #[allow(clippy::cast_precision_loss)]
                let total = *total as f64;
                write!(
                    f,
                    "\n- {name}: {}/{} ({:.2}%)",
                    fmt_unit(used),
                    fmt_unit(total),
                    100.0 * used / total
                )?;
            }
        }

        // Network
        if let Some(network) = &self.net_interfaces {
            write!(f, "\n**Network interfaces:**")?;
            for (
                name,
                NetworkMetrics {
                    bytes_received,
                    bytes_transmitted,
                    received_error_percentage,
                    transmit_error_percentage,
                },
            ) in network
            {
                let received = fmt_unit_net(*bytes_received);
                let transmitted = fmt_unit_net(*bytes_transmitted);
                write!(f, "\n- {name}: {received} received with {received_error_percentage:.2}% errors")?;
                write!(f, ", {transmitted} transmitted with {transmit_error_percentage:.2}% errors")?;
            }
        }

        Ok(())
    }
}

impl SysInfo {
    pub fn new() -> Self {
        Self {
            system: System::new_with_specifics(
                RefreshKind::new().without_processes(),
            ),
            networks: Networks::new(),
            disks: Disks::new(),
        }
    }

    pub fn collect(&mut self) -> Metrics {
        self.system.refresh_all();

        let cpu = self.system.global_cpu_usage();
        let ram = MemoryMetrics {
            used: self.system.used_memory(),
            total: self.system.total_memory(),
        };
        let disk = {
            let mut set = HashSet::new(); // TODO: Read from config
            set.insert("/dev/sdb1");
            self.collect_disk(&set)
        };
        let network = {
            let mut set = HashSet::new(); // TODO: Read from config
            set.insert("enp5s0");
            self.collect_network(&set)
        };

        Metrics {
            cpu,
            ram,
            disks: disk,
            net_interfaces: network,
        }
    }

    fn collect_network(
        &mut self,
        interfaces: &HashSet<&str>,
    ) -> Option<HashMap<Box<str>, NetworkMetrics>> {
        if interfaces.is_empty() {
            return None;
        }

        self.networks.refresh_list();
        let res = self
            .networks
            .list()
            .iter()
            .filter_map(|(name, data)| {
                if !interfaces.contains(&**name) {
                    return None;
                }

                #[allow(clippy::cast_precision_loss)]
                let received_error_percentage =
                    data.errors_on_received() as f64 * 100f64
                        / data.packets_received() as f64;
                if !received_error_percentage.is_finite() {
                    return None;
                }
                #[allow(clippy::cast_precision_loss)]
                let transmit_error_percentage =
                    data.errors_on_transmitted() as f64 * 100f64
                        / data.packets_transmitted() as f64;
                if !transmit_error_percentage.is_finite() {
                    return None;
                }

                let metrics = NetworkMetrics {
                    #[allow(clippy::cast_precision_loss)]
                    bytes_received: data.received() as f64 / DELAY_SECS,
                    #[allow(clippy::cast_precision_loss)]
                    bytes_transmitted: data.transmitted() as f64 / DELAY_SECS,
                    received_error_percentage,
                    transmit_error_percentage,
                };
                Some((Box::from(name.clone()), metrics))
            })
            .collect::<HashMap<_, _>>();

        if res.is_empty() {
            None
        } else {
            Some(res)
        }
    }

    fn collect_disk(
        &mut self,
        disks: &HashSet<&str>,
    ) -> Option<HashMap<Box<str>, MemoryMetrics>> {
        if disks.is_empty() {
            return None;
        }

        self.disks.refresh_list();
        let res = self
            .disks
            .list()
            .iter()
            .filter_map(|d| {
                let name = d.name().to_string_lossy();
                if !disks.contains(&*name) {
                    return None;
                }

                let total = d.total_space();
                let available = d.available_space();
                let metrics = MemoryMetrics {
                    used: total - available,
                    total,
                };
                Some((Box::from(name), metrics))
            })
            .collect::<HashMap<_, _>>();

        if res.is_empty() {
            None
        } else {
            Some(res)
        }
    }
}

fn fmt_unit(mut bytes: f64) -> String {
    let next = |c: &mut f64| {
        *c /= 1_024.0;
        *c
    };
    if bytes < 1_024.0 {
        return format!("{bytes:.2}B");
    }
    if next(&mut bytes) < 1_024.0 {
        return format!("{bytes:.2}KiB");
    }
    if next(&mut bytes) < 1_024.0 {
        return format!("{bytes:.2}MiB");
    }
    if next(&mut bytes) < 1_024.0 {
        return format!("{bytes:.2}GiB");
    }
    format!("{:.2}TiB", next(&mut bytes))
}

fn fmt_unit_net(bytes: f64) -> String {
    let mut bits = bytes * 8.0;
    let next = |c: &mut f64| {
        *c /= 1_024.0;
        *c
    };
    if bits < 1_024.0 {
        return format!("{bits:.2}bps");
    }
    if next(&mut bits) < 1_024.0 {
        return format!("{bits:.2}Kbps");
    }
    if next(&mut bits) < 1_024.0 {
        return format!("{bits:.2}Mbps");
    }
    if next(&mut bits) < 1_024.0 {
        return format!("{bits:.2}Gbps");
    }
    format!("{:.2}Tbps", next(&mut bits))
}
