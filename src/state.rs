use std::str::FromStr;

use chrono::{DateTime, Local};
use serde::{Deserialize, Serialize};

use crate::theme::Theme;

/// Details common operations on the application's state.
pub trait StateRepository {
    type Err;

    /// Get the full application state.
    fn state(&self) -> Result<State, Self::Err>;

    /// Persist the full application state.
    fn set_state(&self, data: State) -> Result<(), Self::Err>;

    /// Get the theme switching service's pause state.
    fn pause_state(&self) -> Result<PauseState, Self::Err>;

    /// Persist the application pause state.
    fn set_pause_state(&self, pause_state: PauseState) -> Result<(), Self::Err>;

    /// Last theme set by the theme switching service.
    fn theme(&self) -> Result<Theme, Self::Err>;

    /// Persist the last theme set by the theme switching service.
    fn set_theme(&self, theme: Theme) -> Result<(), Self::Err>;
}

#[derive(Debug, Clone, Copy, Default, Serialize, Deserialize)]
pub struct State {
    pub pause_state: PauseState,
    pub theme: Theme,
}

impl FromStr for State {
    type Err = serde_json::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        serde_json::from_str(s)
    }
}

/// Represents whether the theme switching service is active or paused.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "type", content = "data")]
pub enum PauseState {
    /// Theme switching service is active (not paused).
    #[default]
    Active,

    /// Theme switching service is disabled until the given local date and time.
    Until(DateTime<Local>),

    /// Theme switching service is disabled indefinitely (no automatic reactivation).
    Indefinite,
}

impl PauseState {
    /// Whether we are still paused based on the current time.
    pub fn is_paused(&self, now: &DateTime<Local>) -> bool {
        match *self {
            Self::Active => false,
            Self::Indefinite => true,
            Self::Until(ref until) => now < until,
        }
    }
}
