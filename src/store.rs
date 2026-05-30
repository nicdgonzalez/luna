use std::path::PathBuf;
use std::{error, fmt, fs, io};

use chrono::{DateTime, Local, NaiveDate};

use crate::cache::{Cache, CacheRepository};
use crate::config::{Config, ConfigRepository, Daylight, Fallback, GeoCoordinate, Location};
use crate::state::{Pause, State, StateRepository};
use crate::theme::Theme;

#[derive(Debug)]
pub struct FileStore {
    state: PathBuf,
    config: PathBuf,
    cache: PathBuf,
}

impl FileStore {
    #[must_use]
    pub fn new(state: PathBuf, config: PathBuf, cache: PathBuf) -> Self {
        Self {
            state,
            config,
            cache,
        }
    }
}

impl StateRepository for FileStore {
    type Err = StateError;

    fn state(&self) -> Result<State, Self::Err> {
        fs::read_to_string(&self.state)?
            .parse::<State>()
            .map_err(|err| StateError::Deserialize { source: err })
    }

    fn set_state(&self, data: State) -> Result<(), Self::Err> {
        let contents = serde_json::to_string_pretty(&data)
            .map_err(|source| StateError::Serialize { source })?;
        fs::write(&self.state, contents)?;
        Ok(())
    }

    fn pause(&self) -> Result<Pause, Self::Err> {
        let data = self.state()?;
        Ok(data.pause)
    }

    fn set_pause(&self, pause_state: Pause) -> Result<(), Self::Err> {
        let mut data = self.state()?;
        data.pause = pause_state;
        self.set_state(data)?;
        Ok(())
    }

    fn theme(&self) -> Result<Theme, Self::Err> {
        let data = self.state()?;
        Ok(data.theme)
    }

    fn set_theme(&self, theme: Theme) -> Result<(), Self::Err> {
        let mut data = self.state()?;
        data.theme = theme;
        self.set_state(data)?;
        Ok(())
    }
}

#[derive(Debug)]
pub enum StateError {
    Io { source: io::Error },
    Serialize { source: serde_json::Error },
    Deserialize { source: serde_json::Error },
}

impl error::Error for StateError {
    fn source(&self) -> Option<&(dyn error::Error + 'static)> {
        match *self {
            Self::Io { ref source } => Some(source),
            Self::Serialize { ref source } | Self::Deserialize { ref source } => Some(source),
        }
    }
}

impl fmt::Display for StateError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match *self {
            Self::Io { source: _ } => "I/O error".fmt(f),
            Self::Serialize { source: _ } => "failed to write JSON".fmt(f),
            Self::Deserialize { source: _ } => "failed to read JSON".fmt(f),
        }
    }
}

impl From<io::Error> for StateError {
    fn from(value: io::Error) -> Self {
        Self::Io { source: value }
    }
}

impl ConfigRepository for FileStore {
    type Err = ConfigError;

    fn config(&self) -> Result<Config, Self::Err> {
        fs::read_to_string(&self.config)?
            .parse::<Config>()
            .map_err(|err| ConfigError::Deserialize { source: err })
    }

    fn set_config(&self, data: Config) -> Result<(), Self::Err> {
        let contents =
            toml::to_string_pretty(&data).map_err(|source| ConfigError::Serialize { source })?;
        fs::write(&self.config, contents)?;
        Ok(())
    }

    fn fallback(&self) -> Result<Fallback, Self::Err> {
        let data = self.config()?;
        Ok(data.fallback)
    }

    fn set_fallback(&self, fallback: Fallback) -> Result<(), Self::Err> {
        let mut data = self.config()?;
        data.fallback = fallback;
        self.set_config(data)?;
        Ok(())
    }

    fn location(&self) -> Result<Option<GeoCoordinate>, Self::Err> {
        let data = self.config()?;

        Ok(match (data.location.longitude, data.location.latitude) {
            (Some(longitude), Some(latitude)) => Some(GeoCoordinate {
                longitude,
                latitude,
            }),
            _ => None,
        })
    }

    fn set_location(&self, location: GeoCoordinate) -> Result<(), Self::Err> {
        let mut data = self.config()?;
        data.location = Location {
            longitude: Some(location.longitude),
            latitude: Some(location.latitude),
            ..data.location
        };
        self.set_config(data)?;
        Ok(())
    }

    fn location_enabled(&self) -> Result<bool, Self::Err> {
        let data = self.config()?;
        Ok(data.location.enabled)
    }

    fn set_location_enabled(&self, enabled: bool) -> Result<(), Self::Err> {
        let mut data = self.config()?;
        data.location.enabled = enabled;
        self.set_config(data)?;
        Ok(())
    }
}

#[derive(Debug)]
pub enum ConfigError {
    Io { source: io::Error },
    Serialize { source: toml::ser::Error },
    Deserialize { source: toml::de::Error },
}

impl error::Error for ConfigError {
    fn source(&self) -> Option<&(dyn error::Error + 'static)> {
        match *self {
            Self::Io { ref source } => Some(source),
            Self::Serialize { ref source } => Some(source),
            Self::Deserialize { ref source } => Some(source),
        }
    }
}

impl fmt::Display for ConfigError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match *self {
            Self::Io { source: _ } => "I/O error".fmt(f),
            Self::Serialize { source: _ } => "failed to write TOML".fmt(f),
            Self::Deserialize { source: _ } => "failed to read TOML".fmt(f),
        }
    }
}

impl From<io::Error> for ConfigError {
    fn from(value: io::Error) -> Self {
        Self::Io { source: value }
    }
}

impl CacheRepository for FileStore {
    type Err = CacheError;

    fn cache(&self) -> Result<Cache, Self::Err> {
        fs::read_to_string(&self.cache)?
            .parse::<Cache>()
            .map_err(|err| CacheError::Deserialize { source: err })
    }

    fn set_cache(&self, data: Cache) -> Result<(), Self::Err> {
        let contents = serde_json::to_string_pretty(&data)
            .map_err(|source| CacheError::Serialize { source })?;
        fs::write(&self.cache, contents)?;
        Ok(())
    }

    fn daylight(&self) -> Result<Option<Daylight>, Self::Err> {
        let data = self.cache()?;
        let daylight = match (data.sunrise, data.sunset) {
            (Some(sunrise), Some(sunset)) => Some(Daylight { sunrise, sunset }),
            _ => None,
        };

        Ok(daylight)
    }

    fn set_daylight(&self, daylight: Daylight) -> Result<(), Self::Err> {
        let mut data = self.cache()?;
        data.sunrise = Some(daylight.sunrise);
        data.sunset = Some(daylight.sunset);
        self.set_cache(data)?;
        Ok(())
    }

    fn last_updated_at(&self) -> Result<Option<NaiveDate>, Self::Err> {
        let data = self.cache()?;
        Ok(data.last_updated_at)
    }

    fn set_last_updated_at(&self, last_update_at: NaiveDate) -> Result<(), Self::Err> {
        let mut data = self.cache()?;
        data.last_updated_at = Some(last_update_at);
        self.set_cache(data)?;
        Ok(())
    }

    fn last_location_attempt(&self) -> Result<Option<DateTime<Local>>, Self::Err> {
        let data = self.cache()?;
        Ok(data.last_location_attempt)
    }

    fn set_last_location_attempt(
        &self,
        last_location_attempt: DateTime<Local>,
    ) -> Result<(), Self::Err> {
        let mut data = self.cache()?;
        data.last_location_attempt = Some(last_location_attempt);
        self.set_cache(data)?;
        Ok(())
    }
}

#[derive(Debug)]
pub enum CacheError {
    Io { source: io::Error },
    Serialize { source: serde_json::Error },
    Deserialize { source: serde_json::Error },
}

impl error::Error for CacheError {
    fn source(&self) -> Option<&(dyn error::Error + 'static)> {
        match *self {
            Self::Io { ref source } => Some(source),
            Self::Serialize { ref source } | Self::Deserialize { ref source } => Some(source),
        }
    }
}

impl fmt::Display for CacheError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match *self {
            Self::Io { source: _ } => "I/O error".fmt(f),
            Self::Serialize { source: _ } => "failed to write JSON".fmt(f),
            Self::Deserialize { source: _ } => "failed to read JSON".fmt(f),
        }
    }
}

impl From<io::Error> for CacheError {
    fn from(value: io::Error) -> Self {
        Self::Io { source: value }
    }
}
