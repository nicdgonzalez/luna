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
mod state;
mod store;
mod theme;

use std::fs;
use std::io::{self, Write as _};
use std::path::PathBuf;
use std::process::ExitCode;

use anyhow::Context as _;
use clap::Parser as _;
use colored::Colorize;
use tracing_subscriber::EnvFilter;
use tracing_subscriber::prelude::*;

use crate::cache::CacheRepository as _;
use crate::config::ConfigRepository as _;
use crate::context::Context;
use crate::state::StateRepository as _;
use crate::store::FileStore;

/// Automatically update the system theme based on local sunrise and sunset times.
#[derive(Debug, clap::Parser)]
struct Parser {
    #[clap(subcommand)]
    subcommand: commands::Subcommand,
}

fn main() -> ExitCode {
    try_main().unwrap_or_else(|err| {
        let mut stderr = io::stderr().lock();
        writeln!(stderr, "{} failed", env!("CARGO_PKG_NAME")).ok();

        for cause in err.chain() {
            writeln!(stderr, "  {}: {}", "Cause".bold(), cause).ok();
        }

        ExitCode::FAILURE
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

    let state = get_state_path()?;
    let config = get_config_path()?;
    let cache = get_cache_path()?;
    let store = FileStore::new(state, config, cache);

    store.set_state(store.state().unwrap_or_default())?;
    store.set_config(store.config().unwrap_or_default())?;
    store.set_cache(store.cache().unwrap_or_default())?;

    let ctx = Context::new(store);

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

fn get_cache_path() -> anyhow::Result<PathBuf> {
    let mut path = dirs::cache_dir().context("failed to get cache directory")?;
    path.extend(["luna", "cache.json"]);

    fs::create_dir_all(path.parent().unwrap())
        .context("failed to create subdirectory in cache directory")?;

    Ok(path)
}
