mod sys_info;

use std::time::Duration;

use nexus_api::{r#impl, tokio, tracing, Meta};
use sys_info::SysInfo;
use tokio::time::sleep;
use tracing::info;

pub const DELAY_SECS: f64 = 30.0;

r#impl! {
    pub static META: Meta = Meta {
        name: env!("CARGO_PKG_NAME"),
        authors: env!("CARGO_PKG_AUTHORS"),
        version: env!("CARGO_PKG_VERSION"),
    };

    async fn main(&self) {
        info!("Now collecting system metrics");

        // Initialize components
        let mut metrics_collector = SysInfo::new();

        // // Start SSH audit monitoring if enabled
        // if true { // TODO
        //     let audit_monitor = AuditMonitor::new(Arc::clone(&config), Arc::clone(&notifier));
        //     audit_monitor.start().await;
        // }

        // Main loop interval
        loop {
            // Collect system metrics
            let metrics = metrics_collector.collect();
            info!("## Metrics update\n{metrics}" );
            // Wait for the next update interval
            #[allow(clippy::cast_sign_loss)]
            #[allow(clippy::cast_possible_truncation)]
            sleep(Duration::from_secs(DELAY_SECS as u64)).await;
        }
    }
}
