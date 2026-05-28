use std::str::FromStr;

use chrono::{DateTime, Local, NaiveDate, NaiveTime};
use serde::{Deserialize, Serialize};

use crate::config::Daylight;

pub trait CacheRepository {
    type Err;

    fn cache(&self) -> Result<Cache, Self::Err>;

    fn set_cache(&self, data: Cache) -> Result<(), Self::Err>;

    fn daylight(&self) -> Result<Option<Daylight>, Self::Err>;

    fn set_daylight(&self, daylight: Daylight) -> Result<(), Self::Err>;

    fn last_updated_at(&self) -> Result<Option<NaiveDate>, Self::Err>;

    fn set_last_updated_at(&self, last_update_at: NaiveDate) -> Result<(), Self::Err>;

    fn last_location_attempt(&self) -> Result<Option<DateTime<Local>>, Self::Err>;

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
