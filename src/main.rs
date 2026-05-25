#![warn(
    clippy::correctness,
    clippy::suspicious,
    clippy::complexity,
    clippy::perf,
    clippy::style,
    clippy::pedantic
)]

mod cache;
mod commands;
mod config;
mod context;
mod repository;
mod state;

use std::fs;
use std::io::{self, Write as _};
use std::path::PathBuf;
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

    let state_path = get_state_path()?;
    let config_path = get_config_path()?;

    let repo = FileRepository::new(state_path, config_path);
    let state = StateManager::from_repo_or_default(repo);
    let ctx = Context::new(state);

    args.subcommand.run(ctx).map(|()| ExitCode::SUCCESS)
}

fn get_state_path() -> anyhow::Result<PathBuf> {
    let mut path = dirs::data_dir().context("failed to get data directory")?;
    path.extend(["luna", "state.json"]);

    fs::create_dir_all(path.parent().unwrap())
        .context("failed to create subdirectory in data directory")?;

    Ok(path)
}

fn get_config_path() -> anyhow::Result<PathBuf> {
    let mut path = dirs::config_dir().context("failed to get config directory")?;
    path.extend(["luna", "Luna.toml"]);

    fs::create_dir_all(path.parent().unwrap())
        .context("failed to create subdirectory in config directory")?;

    Ok(path)
}
