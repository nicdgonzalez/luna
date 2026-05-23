#![warn(
    clippy::correctness,
    clippy::suspicious,
    clippy::complexity,
    clippy::perf,
    clippy::style,
    clippy::pedantic
)]

mod commands;

use std::io::{self, Write as _};
use std::process::ExitCode;

use clap::Parser as _;
use colored::Colorize;
use tracing_subscriber::EnvFilter;
use tracing_subscriber::prelude::*;

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
    args.subcommand.run().map(|()| ExitCode::SUCCESS)
}
