use std::io::{self, Write as _};

use anyhow::Context as _;
use colored::Colorize as _;

use crate::commands::prelude::*;
use crate::theme::Theme;

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

        let current_theme = Theme::from_system().context("failed to get current theme")?;

        ctx.state_mut().resume()?;
        ctx.state_mut().set_theme(current_theme)?;

        writeln!(
            io::stdout(),
            "{}",
            "Theme switcher has been enabled".green()
        )
        .ok();

        Ok(())
    }
}
