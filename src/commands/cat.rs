use std::io::{self, Write as _};

use anyhow::Context as _;

use crate::cache::CacheRepository as _;
use crate::commands::prelude::*;
use crate::config::ConfigRepository as _;
use crate::state::StateRepository as _;

#[derive(Debug, Clone, Copy, clap::ValueEnum)]
enum Data {
    State,
    Cache,
    Config,
}

#[derive(Debug, Clone, clap::Args)]
pub struct Cat {
    data: Data,
}

impl Run for Cat {
    fn run(&self, ctx: &mut Context) -> anyhow::Result<()> {
        match self.data {
            Data::State => {
                let data = ctx.store().state().context("failed to get state")?;
                let contents = serde_json::to_string_pretty(&data).unwrap();
                write!(io::stdout(), "{contents}").ok();
            }
            Data::Cache => {
                let data = ctx.store().cache().context("failed to get cache")?;
                let contents = serde_json::to_string_pretty(&data).unwrap();
                write!(io::stdout(), "{contents}").ok();
            }
            Data::Config => {
                let data = ctx.store().config().context("failed to get config")?;
                let contents = toml::to_string_pretty(&data).unwrap();
                write!(io::stdout(), "{contents}").ok();
            }
        }

        Ok(())
    }
}
