use std::str::FromStr;

use chrono::{DateTime, Local, NaiveDate, NaiveTime};
use serde::{Deserialize, Serialize};

use crate::config::Daylight;

pub trait CacheRepository {
    type Err;

    /// Get the full cache state.
    fn cache(&self) -> Result<Cache, Self::Err>;

    /// Persist the full cache state.
    fn set_cache(&self, data: Cache) -> Result<(), Self::Err>;

    /// Daylight information derived from the current configuration.
    ///
    /// Returns `None` when daylight information has not yet been resolved or could not
    /// be determined.
    fn daylight(&self) -> Result<Option<Daylight>, Self::Err>;

    /// Persist cached daylight information.
    fn set_daylight(&self, daylight: Daylight) -> Result<(), Self::Err>;

    /// Date the daylight information was last refreshed.
    ///
    /// Returns `None` when the cache has never been refreshed.
    fn last_updated_at(&self) -> Result<Option<NaiveDate>, Self::Err>;

    /// Persist the date the daylight information was last refreshed.
    fn set_last_updated_at(&self, last_update_at: NaiveDate) -> Result<(), Self::Err>;

    /// Timestamp of the most recent attempt to resolve the configured location.
    ///
    /// Returns `None` when no location resolution attempt has been made.
    fn last_location_attempt(&self) -> Result<Option<DateTime<Local>>, Self::Err>;

    /// Persist the timestamp of the most recent location resolution attempt.
    fn set_last_location_attempt(
        &self,
        last_location_attempt: DateTime<Local>,
    ) -> Result<(), Self::Err>;
}

#[derive(Debug, Clone, Copy, Default, Serialize, Deserialize)]
pub struct Cache {
    pub sunrise: Option<NaiveTime>,
    pub sunset: Option<NaiveTime>,
    pub last_updated_at: Option<NaiveDate>,
    pub last_location_attempt: Option<DateTime<Local>>,
}

impl FromStr for Cache {
    type Err = serde_json::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        serde_json::from_str(s)
    }
}
