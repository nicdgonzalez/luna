use std::io::{self, Write as _};

use anyhow::Context as _;
use colored::Colorize as _;

use crate::commands::prelude::*;
use crate::state::{PauseState, StateRepository};
use crate::theme::Theme;

#[derive(Debug, Clone, clap::Args)]
pub struct Enable;

impl Run for Enable {
    fn run(&self, ctx: &mut Context) -> anyhow::Result<()> {
        let pause_state = ctx
            .store()
            .pause_state()
            .context("failed to get pause state")?;

        if pause_state == PauseState::None {
            writeln!(
                io::stdout(),
                "{}",
                "Theme switcher already enabled".yellow()
            )
            .ok();

            return Ok(());
        }

        let current_theme = Theme::from_system().context("failed to get current theme")?;

        ctx.store()
            .set_pause_state(PauseState::None)
            .context("failed to set pause state")?;

        ctx.store().set_theme(current_theme)?;

        writeln!(
            io::stdout(),
            "{}",
            "Theme switcher has been enabled".green()
        )
        .ok();

        Ok(())
    }
}
