use std::io::{self, Write as _};

use colored::Colorize as _;

use crate::commands::prelude::*;

#[derive(Debug, Clone, clap::Args)]
pub struct Enable;

impl Run for Enable {
    fn run(&self, ctx: &mut Context) -> anyhow::Result<()> {
        if ctx.state().data().pause().is_none() {
            writeln!(
                io::stdout(),
                "{}",
                "Theme switcher already enabled".yellow()
            )
            .ok();

            return Ok(());
        }

        ctx.state_mut().resume()?;

        writeln!(
            io::stdout(),
            "{}",
            "Theme switcher has been enabled".green()
        )
        .ok();

        Ok(())
    }
}
