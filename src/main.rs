#![warn(
    clippy::correctness,
    clippy::suspicious,
    clippy::complexity,
    clippy::perf,
    clippy::style,
    clippy::pedantic
)]

mod commands;
mod context;
mod state;

use std::fs;
use std::io::{self, Write as _};
use std::process::ExitCode;

use anyhow::Context as _;
use clap::Parser as _;
use colored::Colorize;
use tracing_subscriber::EnvFilter;
use tracing_subscriber::prelude::*;

use crate::context::{Context, FileRepository};
use crate::state::StateManager;

/// Automatically update the system theme based on local sunrise and sunset times.
#[derive(Debug, clap::Parser)]
struct Parser {
    #[clap(subcommand)]
    subcommand: commands::Subcommand,
}

fn main() -> ExitCode {
    tokio::runtime::Builder::new_current_thread()
        .build()
        .unwrap()
        .block_on(async {
            try_main().unwrap_or_else(|err| {
                let mut stderr = io::stderr().lock();
                writeln!(stderr, "{} failed", env!("CARGO_PKG_NAME")).ok();

                for cause in err.chain() {
                    writeln!(stderr, "  {}: {}", "Cause".bold(), cause).ok();
                }

                ExitCode::FAILURE
            })
        })
}

fn try_main() -> anyhow::Result<ExitCode> {
    tracing_subscriber::registry()
        .with(
            EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| format!("{}=trace", env!("CARGO_CRATE_NAME")).into()),
        )
        .with(tracing_subscriber::fmt::layer())
        .init();

    let args = Parser::parse();

    let mut state_path = dirs::data_dir().context("failed to get data directory")?;
    state_path.extend(["luna", "state.json"]);
    fs::create_dir_all(state_path.parent().unwrap())
        .context("failed to create subdirectory in data directory")?;

    let repo = FileRepository::new(state_path);
    let state = StateManager::new_or_default(repo);
    let ctx = Context::new(state);

    args.subcommand.run(ctx).map(|()| ExitCode::SUCCESS)
}
