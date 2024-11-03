pub struct Config {
    pub(crate) webhook_url: String,
}

impl Config {
    pub fn new(webhook_url: String) -> Self {
        Self { webhook_url }
    }

    pub fn new_from_env() -> Self {
        Self::new(
            std::env::var("DISCORD_WEBHOOK_URL")
                .expect("DISCORD_WEBHOOK_URL env. var not set!"),
        )
    }
}
