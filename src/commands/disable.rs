use std::io::{self, Write as _};

use colored::Colorize as _;

use crate::commands::prelude::*;

#[derive(Debug, Clone, clap::Args)]
pub struct Disable;

impl Run for Disable {
    fn run(&self, ctx: &mut Context) -> anyhow::Result<()> {
        ctx.state_mut().pause_indefinitely()?;

        writeln!(
            io::stdout(),
            "{}",
            "Theme switcher has been disabled".green()
        )
        .ok();

        Ok(())
    }
}
