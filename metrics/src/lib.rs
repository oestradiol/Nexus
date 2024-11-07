mod metrics;

use std::time::Duration;

use metrics::{MetricsDataFormatted, SystemMetrics};
use nexus_api::{r#impl, Meta};
use tokio::time::sleep;
use tracing::info;

r#impl! {
    pub static META: Meta = Meta {
        name: env!("CARGO_PKG_NAME"),
        authors: env!("CARGO_PKG_AUTHORS"),
        version: env!("CARGO_PKG_VERSION"),
    };

    async fn main(&self) {
        info!("Now collecting system metrics");

        // Initialize components
        let mut metrics_collector = SystemMetrics::new();

        // // Start SSH audit monitoring if enabled
        // if true { // TODO
        //     let audit_monitor = AuditMonitor::new(Arc::clone(&config), Arc::clone(&notifier));
        //     audit_monitor.start().await;
        // }

        // Main loop interval
        loop {
            // Collect system metrics
            let metrics = metrics_collector.collect().await;
            let metrics = MetricsDataFormatted::from(metrics);
            info!("## Metrics update\n{}", metrics.to_string());
            // Wait for the next update interval
            sleep(Duration::from_secs(30)).await;
        }
    }
}
