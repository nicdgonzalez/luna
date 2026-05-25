use std::io::{self, Write as _};

use colored::Colorize as _;

use crate::commands::prelude::*;
use crate::state::Pause;

#[derive(Debug, Clone, clap::Args)]
pub struct Disable;

impl Run for Disable {
    fn run(&self, ctx: &mut Context) -> anyhow::Result<()> {
        ctx.state_mut().set_pause(Some(Pause::Indefinite))?;

        writeln!(
            io::stdout(),
            "{}",
            "Theme switcher has been disabled".green()
        )
        .ok();

        Ok(())
    }
}
