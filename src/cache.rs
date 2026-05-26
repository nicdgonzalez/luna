use std::str::FromStr;

use anyhow::Context as _;
use chrono::{DateTime, Local, NaiveTime};
use tracing::warn;

use crate::repository::Repository;

#[derive(Debug)]
pub struct Cache<R> {
    repo: R,
    data: CacheData,
}

impl<R> Cache<R>
where
    R: Repository<CacheData>,
{
    /// Construct a new state manager using data from the repository.
    #[expect(unused, reason = "not needed, but for sake of completeness...")]
    pub fn from_repo(repo: R) -> Result<Self, R::Err> {
        let data = repo.load()?;
        Ok(Self { repo, data })
    }

    /// Construct a new state manager using data from the repository.
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
            .inspect_err(|err| warn!("failed to load existing state: {err}"))
            .unwrap_or_default();
    }

    fn save(&mut self) -> Result<(), R::Err> {
        self.repo.save(&self.data)
    }

    /// Represents the data stored within our repository.
    #[expect(unused)]
    #[must_use]
    pub fn data(&self) -> &CacheData {
        &self.data
    }

    /// Last attempt at retrieving the user's location (this is to avoid hitting any rate limits).
    pub fn last_location_attempt(&self) -> Option<&DateTime<Local>> {
        self.data.last_location_attempt.as_ref()
    }

    pub fn set_last_location_attempt(&mut self, now: DateTime<Local>) -> Result<(), R::Err> {
        self.data.last_location_attempt = Some(now);
        self.save()
    }

    pub fn daylight(&self) -> Option<Daylight> {
        match (self.data.sunrise, self.data.sunset) {
            (Some(sunrise), Some(sunset)) => Some(Daylight { sunrise, sunset }),
            _ => None,
        }
    }

    pub fn set_last_updated_at(&mut self, last_updated_at: &DateTime<Local>) -> Result<(), R::Err> {
        self.data.last_updated_at = *last_updated_at;
        self.save()
    }

    pub fn set_daylight(&mut self, daylight: Daylight) -> Result<(), R::Err> {
        self.data.sunrise = Some(daylight.sunrise);
        self.data.sunset = Some(daylight.sunset);
        self.save()
    }
}

#[derive(Debug, Clone, Copy, Default, serde::Serialize, serde::Deserialize)]
pub struct CacheData {
    last_updated_at: DateTime<Local>,
    sunrise: Option<NaiveTime>,
    sunset: Option<NaiveTime>,
    last_location_attempt: Option<DateTime<Local>>,
}

#[derive(Debug, Clone, Copy)]
pub struct Daylight {
    pub sunrise: NaiveTime,
    pub sunset: NaiveTime,
}

impl Daylight {
    #[expect(unused)]
    pub const fn sunrise(&self) -> &NaiveTime {
        &self.sunrise
    }

    #[expect(unused)]
    pub const fn sunset(&self) -> &NaiveTime {
        &self.sunset
    }

    pub fn is_daytime(&self, now: NaiveTime) -> bool {
        if self.sunset < self.sunrise {
            // The sun sets after midnight, so disregard it for now.
            now >= self.sunrise
        } else {
            now >= self.sunrise && now < self.sunset
        }
    }
}

impl FromStr for CacheData {
    type Err = anyhow::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        serde_json::from_str(s).context("failed to parse state")
    }
}
