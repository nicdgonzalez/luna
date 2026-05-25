use std::str::FromStr;

use anyhow::Context as _;
use chrono::NaiveTime;
use tracing::warn;

use crate::repository::Repository;

#[derive(Debug)]
pub struct Config<R> {
    repo: R,
    data: ConfigData,
}

impl<R> Config<R>
where
    R: Repository<ConfigData>,
{
    /// Construct a new configuration manager using data from the repository.
    #[expect(unused, reason = "not needed, but for sake of completeness...")]
    pub fn from_repo(repo: R) -> Result<Self, R::Err> {
        let data = repo.load()?;
        Ok(Self { repo, data })
    }

    /// Construct a new configuration manager using data from the repository.
    /// Upon failure, load using default data instead.
    #[must_use]
    pub fn from_repo_or_default(repo: R) -> Self {
        let data = repo.load().unwrap_or_default();
        Self { repo, data }
    }

    /// Synchronize our data with the repository.
    #[expect(unused)]
    pub fn reload(&mut self) {
        self.data = self
            .repo
            .load()
            .inspect_err(|err| warn!("failed to load existing configuration: {err}"))
            .unwrap_or_default();
    }

    /// Represents the data stored within our repository.
    #[expect(unused)]
    #[must_use]
    pub fn data(&self) -> &ConfigData {
        &self.data
    }
}

#[derive(Debug, Clone, Default, serde::Serialize, serde::Deserialize)]
pub struct ConfigData {
    /// Default values for when location-based sunrise/sunset times are not available
    fallback: Fallback,
    /// Geographical position for determining sunrise/sunset times
    location: Location,
}

impl ConfigData {
    #[expect(unused)]
    #[must_use]
    pub fn fallback(&self) -> &Fallback {
        &self.fallback
    }

    #[expect(unused)]
    #[must_use]
    pub fn location(&self) -> &Location {
        &self.location
    }
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct Fallback {
    sunrise: NaiveTime,
    sunset: NaiveTime,
}

impl Default for Fallback {
    fn default() -> Self {
        Self {
            sunrise: NaiveTime::from_hms_opt(6, 30, 0).unwrap(),
            sunset: NaiveTime::from_hms_opt(18, 30, 0).unwrap(),
        }
    }
}

impl Fallback {
    /// Time to turn on light mode
    #[expect(unused)]
    #[must_use]
    pub fn sunrise(&self) -> &NaiveTime {
        &self.sunrise
    }

    /// Time to turn on dark mode
    #[expect(unused)]
    #[must_use]
    pub fn sunset(&self) -> &NaiveTime {
        &self.sunset
    }
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct Location {
    enabled: bool,
    longitude: Option<f32>,
    latitude: Option<f32>,
}

#[expect(unused)]
#[derive(Debug, Clone, Copy)]
pub struct GeoCoordinate {
    /// Position on the Earth horizontally
    pub longitude: f32,
    /// Position on the Earth vertically
    pub latitude: f32,
}

impl Default for Location {
    fn default() -> Self {
        Self {
            enabled: true,
            longitude: None,
            latitude: None,
        }
    }
}

impl Location {
    /// Whether to use the user's current location to determine sunrise/sunset times
    #[expect(unused)]
    #[must_use]
    pub const fn is_enabled(&self) -> bool {
        self.enabled
    }

    /// Returns the stored geographic coordinates if both longitude and latitude are set.
    #[expect(unused)]
    pub fn coordinates(&self) -> Option<GeoCoordinate> {
        match (self.longitude, self.latitude) {
            (Some(longitude), Some(latitude)) => Some(GeoCoordinate {
                longitude,
                latitude,
            }),
            _ => None,
        }
    }
}

impl FromStr for ConfigData {
    type Err = anyhow::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        toml::from_str(s).context("failed to parse state")
    }
}
