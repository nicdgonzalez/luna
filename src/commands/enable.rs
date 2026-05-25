use std::io::{self, Write as _};

use colored::Colorize as _;

use crate::commands::prelude::*;

#[derive(Debug, Clone, clap::Args)]
pub struct Enable;

impl Run for Enable {
    fn run(&self, ctx: &mut Context) -> anyhow::Result<()> {
        ctx.state_mut().set_pause(None)?;

        writeln!(
            io::stdout(),
            "{}",
            "Theme switcher has been enabled".green()
        )
        .ok();

        Ok(())
    }
}
