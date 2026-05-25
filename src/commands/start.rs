use std::thread;
use std::time::{Duration, Instant};

use chrono::Local;

use crate::commands::prelude::*;

#[derive(Debug, Clone, clap::Args)]
pub struct Start {
    /// Time to wait between checks (in milliseconds)
    #[arg(long, default_value = "3000")]
    interval: u64,
}

impl Run for Start {
    fn run(&self, ctx: &mut Context) -> anyhow::Result<()> {
        let interval = Duration::from_millis(self.interval);

        loop {
            let next_tick = Instant::now() + interval;
            let now = Local::now();

            let state = ctx.state_mut();
            state.reload(); // Ensure we have the latest changes.

            if state.is_paused(&now) {
                sleep_until(next_tick);
                continue;
            }

            // Get user's location
            // - Check configuration file
            // - Check cache
            // - Query external API
            let _config = ctx.config_mut();

            // Get sunrise/sunset times

            // Compare against current theme

            // Change theme if not already set

            sleep_until(next_tick);
        }
    }
}

/// Puts the current thread to sleep until at least the specified deadline has passed.
fn sleep_until(deadline: Instant) {
    let now = Instant::now();

    if now < deadline {
        thread::sleep(deadline - now);
    }
}
