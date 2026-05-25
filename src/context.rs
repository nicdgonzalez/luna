use std::fs;
use std::path::PathBuf;

use anyhow::Context as _;

use crate::state::{StateData, StateManager, StateRepository};

#[derive(Debug)]
pub struct Context {
    state: StateManager<FileRepository>,
}

impl Context {
    #[must_use]
    pub const fn new(state: StateManager<FileRepository>) -> Self {
        Self { state }
    }

    // #[must_use]
    // pub fn state(&self) -> &StateManager<FileRepository> {
    //     &self.state
    // }

    #[must_use]
    pub fn state_mut(&mut self) -> &mut StateManager<FileRepository> {
        &mut self.state
    }
}

#[derive(Debug)]
pub struct FileRepository {
    state_path: PathBuf,
}

impl StateRepository for FileRepository {
    type Err = anyhow::Error;

    fn load_state(&self) -> Result<StateData, Self::Err> {
        fs::read_to_string(&self.state_path)
            .context("failed to read existing state")?
            .parse::<StateData>()
    }

    fn save_state(&mut self, state: &StateData) -> Result<(), Self::Err> {
        let contents = serde_json::to_string_pretty(state).context("failed to serialize data")?;
        fs::write(&self.state_path, contents).context("failed to write to file")
    }
}

impl FileRepository {
    #[must_use]
    pub const fn new(state_path: PathBuf) -> Self {
        Self { state_path }
    }
}
