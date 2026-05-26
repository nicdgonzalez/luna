use std::str::FromStr;

use anyhow::Context as _;
use chrono::{DateTime, Local};
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
}

#[derive(Debug, Clone, Default, serde::Serialize, serde::Deserialize)]
pub struct CacheData {
    last_update: DateTime<Local>,
    sunrise: Option<DateTime<Local>>,
    sunset: Option<DateTime<Local>>,
    last_location_attempt: Option<DateTime<Local>>,
}

impl FromStr for CacheData {
    type Err = anyhow::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        serde_json::from_str(s).context("failed to parse state")
    }
}
