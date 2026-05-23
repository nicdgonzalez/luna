use std::thread;
use std::time::{Duration, Instant};

use tracing::debug;

use crate::commands::prelude::*;

#[derive(Debug, Clone, clap::Args)]
pub struct Start {
    /// Time to wait between checks (in milliseconds)
    #[arg(long, default_value = "3000")]
    interval: u64,
}

impl Run for Start {
    fn run(&self) -> anyhow::Result<()> {
        let interval = Duration::from_millis(self.interval);

        loop {
            let next_tick = Instant::now() + interval;
            debug!("Hello, World!");
            sleep_until(next_tick);
        }
    }
}

/// Puts the current thread to sleep until at least the specified deadline has passed.
fn sleep_until(next_tick: Instant) {
    let tick = Instant::now();

    if tick < next_tick {
        thread::sleep(next_tick - tick);
    }
}
