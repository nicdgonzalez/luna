use std::str::FromStr;

use chrono::NaiveTime;
use serde::{Deserialize, Serialize};

/// Details common operations on the application's configuration.
pub trait ConfigRepository {
    type Err;

    /// Get the full application configuration.
    fn config(&self) -> Result<Config, Self::Err>;

    /// Persist the full application configuration.
    fn set_config(&self, data: Config) -> Result<(), Self::Err>;

    /// Default sunrise/sunset values used when location-based calculations are unavailable.
    fn fallback(&self) -> Result<Fallback, Self::Err>;

    /// Persist the fallback sunrise/sunset values.
    #[expect(dead_code)]
    fn set_fallback(&self, fallback: Fallback) -> Result<(), Self::Err>;

    /// Geographical position used for sunrise/sunset calculations.
    fn location(&self) -> Result<Option<GeoCoordinate>, Self::Err>;

    /// Persist the geographical position used for sunrise/sunset calculations.
    fn set_location(&self, location: GeoCoordinate) -> Result<(), Self::Err>;

    /// Whether location-based sunrise/sunset calculations are enabled.
    fn location_enabled(&self) -> Result<bool, Self::Err>;

    /// Enable or disable location-based sunrise/sunset calculations.
    #[expect(dead_code)]
    fn set_location_enabled(&self, enabled: bool) -> Result<(), Self::Err>;
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct Config {
    /// Default values for when location-based sunrise/sunset times are not available
    pub fallback: Fallback,
    /// Geographical position for determining sunrise/sunset times
    pub location: Location,
}

impl FromStr for Config {
    type Err = toml::de::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        toml::from_str(s)
    }
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct Fallback {
    pub sunrise: NaiveTime,
    pub sunset: NaiveTime,
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
    pub fn daylight(&self) -> Daylight {
        Daylight {
            sunrise: self.sunrise,
            sunset: self.sunset,
        }
    }
}

#[derive(Debug, Clone, Copy)]
pub struct Daylight {
    pub sunrise: NaiveTime,
    pub sunset: NaiveTime,
}

impl Daylight {
    pub fn is_daytime(&self, now: NaiveTime) -> bool {
        if self.sunset < self.sunrise {
            // Indicates that the sun sets on the next day.
            now < self.sunset || now >= self.sunrise
        } else {
            now >= self.sunrise && now < self.sunset
        }
    }
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct Location {
    pub enabled: bool,
    pub longitude: Option<f32>,
    pub latitude: Option<f32>,
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

#[derive(Debug, Clone, Copy)]
pub struct GeoCoordinate {
    /// Position on the Earth horizontally
    pub longitude: f32,
    /// Position on the Earth vertically
    pub latitude: f32,
}
