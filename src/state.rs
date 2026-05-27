use std::str::FromStr;

use anyhow::Context;
use chrono::{DateTime, Local};
use tracing::warn;

use crate::repository::Repository;
use crate::theme::Theme;

#[derive(Debug)]
pub struct State<R> {
    repo: R,
    data: StateData,
}

impl<R> State<R>
where
    R: Repository<StateData>,
{
    /// Construct a new state manager using data from the repository.
    /// Upon failure, load using default data instead.
    #[must_use]
    pub fn from_repo_or_default(repo: R) -> Self {
        let data = repo.load().unwrap_or_default();
        Self { repo, data }
    }

    /// Synchronize our data with the repository.
    pub fn reload(&mut self) {
        self.data = self
            .repo
            .load()
            .inspect_err(|err| warn!("failed to load existing state: {err}"))
            .unwrap_or_default();
    }

    /// Helper function to save current data to the repository.
    pub fn save(&mut self) -> Result<(), R::Err> {
        self.repo.save(&self.data)
    }

    /// Disable the automatic theme switcher until re-enabled.
    pub fn pause_indefinitely(&mut self) -> Result<(), R::Err> {
        self.data.pause = Some(Pause::Indefinite);
        self.save()
    }

    /// Pause the automatic theme switcher until the provided deadline.
    pub fn pause_until(&mut self, deadline: DateTime<Local>) -> Result<(), R::Err> {
        self.data.pause = Some(Pause::Until(deadline));
        self.save()
    }

    /// Unpause the automatic theme switcher.
    pub fn resume(&mut self) -> Result<(), R::Err> {
        self.data.pause = None;
        self.save()
    }

    /// Whether the automatic theme switcher is currently paused.
    #[must_use]
    pub fn is_paused(&self, now: &DateTime<Local>) -> bool {
        self.data.pause.is_some_and(|pause| pause.is_active(now))
    }

    /// Represents the data stored within our repository.
    #[must_use]
    pub fn data(&self) -> &StateData {
        &self.data
    }

    #[must_use]
    pub fn theme(&self) -> Theme {
        self.data.theme
    }

    pub fn set_theme(&mut self, current_theme: Theme) -> Result<(), R::Err> {
        self.data.theme = current_theme;
        self.save()
    }
}

#[derive(Debug, Clone, Default, serde::Serialize, serde::Deserialize)]
pub struct StateData {
    pause: Option<Pause>,
    theme: Theme, // To track whether the user manually changed the theme.
}

impl StateData {
    /// Whether the automatic theme switcher is currently on or off.
    #[must_use]
    pub const fn pause(&self) -> Option<&Pause> {
        self.pause.as_ref()
    }
}

#[derive(Debug, Clone, Copy, serde::Serialize, serde::Deserialize)]
#[serde(tag = "type", content = "data")]
pub enum Pause {
    Until(DateTime<Local>),
    Indefinite,
}

impl Pause {
    /// Whether we are still paused based on the current time.
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
