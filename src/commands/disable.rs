use std::io::{self, Write as _};

use anyhow::Context as _;
use colored::Colorize as _;

use crate::commands::prelude::*;
use crate::state::{PauseState, StateRepository};

#[derive(Debug, Clone, clap::Args)]
pub struct Disable;

impl Run for Disable {
    fn run(&self, ctx: &mut Context) -> anyhow::Result<()> {
        let pause_state = ctx
            .store()
            .pause_state()
            .context("failed to get pause state")?;

        if pause_state == PauseState::Indefinite {
            writeln!(
                io::stdout(),
                "{}",
                "Theme switcher already disabled".yellow()
            )
            .ok();

            return Ok(());
        }

        ctx.store()
            .set_pause_state(PauseState::Indefinite)
            .context("failed to set pause state")?;

        writeln!(
            io::stdout(),
            "{}",
            "Theme switcher has been disabled".green()
        )
        .ok();

        Ok(())
    }
}
