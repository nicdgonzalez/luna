use std::fs;
use std::path::PathBuf;

use anyhow::Context as _;

use crate::config::ConfigData;
use crate::repository::Repository;
use crate::state::{StateData, StateManager};

#[derive(Debug)]
pub struct Context {
    state: StateManager<FileRepository>,
}

impl Context {
    #[must_use]
    pub const fn new(state: StateManager<FileRepository>) -> Self {
        Self { state }
    }

    #[must_use]
    pub fn state(&self) -> &StateManager<FileRepository> {
        &self.state
    }

    #[must_use]
    pub fn state_mut(&mut self) -> &mut StateManager<FileRepository> {
        &mut self.state
    }
}

#[derive(Debug)]
pub struct FileRepository {
    state_path: PathBuf,
    config_path: PathBuf,
}

impl FileRepository {
    #[must_use]
    pub const fn new(state_path: PathBuf, config_path: PathBuf) -> Self {
        Self {
            state_path,
            config_path,
        }
    }
}

impl Repository<'_, StateData> for FileRepository {
    type Err = anyhow::Error;

    fn load(&self) -> Result<StateData, Self::Err> {
        fs::read_to_string(&self.state_path)
            .context("failed to read existing state")?
            .parse::<StateData>()
    }

    fn save(&self, data: &StateData) -> Result<(), Self::Err> {
        let contents = serde_json::to_string_pretty(data).context("failed to serialize data")?;
        fs::write(&self.state_path, contents).context("failed to write to file")
    }
}

impl Repository<'_, ConfigData> for FileRepository {
    type Err = anyhow::Error;

    fn load(&self) -> Result<ConfigData, Self::Err> {
        fs::read_to_string(&self.config_path)
            .context("failed to read existing configuration")?
            .parse::<ConfigData>()
    }

    fn save(&self, data: &ConfigData) -> Result<(), Self::Err> {
        let contents = toml::to_string_pretty(data).context("failed to serialize data")?;
        fs::write(&self.state_path, contents).context("failed to write to file")
    }
}
