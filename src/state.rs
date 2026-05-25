use std::fmt;
use std::str::FromStr;

use anyhow::Context;
use chrono::{DateTime, Local};
use tracing::error;

pub trait StateRepository {
    type Err: fmt::Display;

    fn load_state(&self) -> Result<StateData, Self::Err>;
    fn save_state(&mut self, state: &StateData) -> Result<(), Self::Err>;
}

#[derive(Debug)]
pub struct StateManager<R: StateRepository> {
    repo: R,
    data: StateData,
}

impl<R: StateRepository> StateManager<R> {
    #[must_use]
    pub fn new_or_default(repo: R) -> Self {
        let data = repo.load_state().unwrap_or_default();
        Self { repo, data }
    }

    pub fn reload(&mut self) {
        self.data = self
            .repo
            .load_state()
            .inspect_err(|err| error!("failed to load existing state: {err}"))
            .unwrap_or_default();
    }

    /// Helper function for saving data to a persistent storage.
    fn save(&mut self) -> Result<(), R::Err> {
        self.repo.save_state(&self.data)
    }

    pub fn pause_indefinitely(&mut self) -> Result<(), R::Err> {
        self.data.pause = Some(Pause::Indefinite);
        self.save()
    }

    pub fn pause_until(&mut self, deadline: DateTime<Local>) -> Result<(), R::Err> {
        self.data.pause = Some(Pause::Until(deadline));
        self.save()
    }

    pub fn resume(&mut self) -> Result<(), R::Err> {
        self.data.pause = None;
        self.save()
    }

    pub fn is_paused(&self, now: &DateTime<Local>) -> bool {
        self.data.pause.is_some_and(|pause| pause.is_active(now))
    }
}

#[derive(Debug, Clone, Default, serde::Serialize, serde::Deserialize)]
pub struct StateData {
    pause: Option<Pause>,
}

#[derive(Debug, Clone, Copy, serde::Serialize, serde::Deserialize)]
#[serde(tag = "type", content = "data")]
pub enum Pause {
    Until(DateTime<Local>),
    Indefinite,
}

impl Pause {
    pub fn is_active(&self, now: &DateTime<Local>) -> bool {
        match *self {
            Self::Indefinite => true,
            Self::Until(ref until) => now < until,
        }
    }
}

impl FromStr for StateData {
    type Err = anyhow::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        serde_json::from_str(s).context("failed to parse state")
    }
}
