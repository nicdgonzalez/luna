use std::fs;
use std::path::PathBuf;
use std::sync::Arc;

use anyhow::Context as _;

use crate::cache::{Cache, CacheData};
use crate::config::{Config, ConfigData};
use crate::repository::Repository;
use crate::state::{State, StateData};

#[derive(Debug)]
pub struct Context {
    state: State<Arc<FileRepository>>,
    config: Config<Arc<FileRepository>>,
    cache: Cache<Arc<FileRepository>>,
}

impl Context {
    #[must_use]
    pub const fn new(
        state: State<Arc<FileRepository>>,
        config: Config<Arc<FileRepository>>,
        cache: Cache<Arc<FileRepository>>,
    ) -> Self {
        Self {
            state,
            config,
            cache,
        }
    }

    #[must_use]
    pub fn state(&self) -> &State<Arc<FileRepository>> {
        &self.state
    }

    #[must_use]
    pub fn state_mut(&mut self) -> &mut State<Arc<FileRepository>> {
        &mut self.state
    }

    #[must_use]
    pub fn config(&self) -> &Config<Arc<FileRepository>> {
        &self.config
    }

    #[expect(unused)]
    #[must_use]
    pub fn config_mut(&mut self) -> &mut Config<Arc<FileRepository>> {
        &mut self.config
    }

    #[expect(unused)]
    #[must_use]
    pub fn cache(&self) -> &Cache<Arc<FileRepository>> {
        &self.cache
    }

    #[expect(unused)]
    #[must_use]
    pub fn cache_mut(&mut self) -> &mut Cache<Arc<FileRepository>> {
        &mut self.cache
    }
}

#[derive(Debug)]
pub struct FileRepository {
    state: PathBuf,
    config: PathBuf,
    cache: PathBuf,
}

impl FileRepository {
    #[must_use]
    pub const fn new(state: PathBuf, config: PathBuf, cache: PathBuf) -> Self {
        Self {
            state,
            config,
            cache,
        }
    }
}

impl Repository<StateData> for FileRepository {
    type Err = anyhow::Error;

    fn load(&self) -> Result<StateData, Self::Err> {
        fs::read_to_string(&self.state)
            .context("failed to read existing state")?
            .parse::<StateData>()
    }

    fn save(&self, data: &StateData) -> Result<(), Self::Err> {
        let contents = serde_json::to_string_pretty(data).context("failed to serialize data")?;
        fs::write(&self.state, contents).context("failed to write to file")
    }
}

impl Repository<ConfigData> for FileRepository {
    type Err = anyhow::Error;

    fn load(&self) -> Result<ConfigData, Self::Err> {
        fs::read_to_string(&self.config)
            .context("failed to read existing configuration")?
            .parse::<ConfigData>()
    }

    fn save(&self, data: &ConfigData) -> Result<(), Self::Err> {
        let contents = toml::to_string_pretty(data).context("failed to serialize data")?;
        fs::write(&self.state, contents).context("failed to write to file")
    }
}

impl Repository<CacheData> for FileRepository {
    type Err = anyhow::Error;

    fn load(&self) -> Result<CacheData, Self::Err> {
        fs::read_to_string(&self.cache)
            .context("failed to read existing state")?
            .parse::<CacheData>()
    }

    fn save(&self, data: &CacheData) -> Result<(), Self::Err> {
        let contents = serde_json::to_string_pretty(data).context("failed to serialize data")?;
        fs::write(&self.state, contents).context("failed to write to file")
    }
}
