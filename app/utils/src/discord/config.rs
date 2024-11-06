pub struct Config {
    pub(crate) webhook_url: String,
}

impl Config {
    #[must_use]
    pub const fn new(webhook_url: String) -> Self {
        Self { webhook_url }
    }

    /// # Panics
    /// Panics if the `DISCORD_WEBHOOK_URL` env. var is not set.
    #[must_use]
    pub fn new_from_env() -> Self {
        Self::new(
            std::env::var("DISCORD_WEBHOOK_URL")
                .expect("DISCORD_WEBHOOK_URL env. var not set!"),
        )
    }
}
