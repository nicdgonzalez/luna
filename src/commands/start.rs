use std::process::Command;
use std::thread;
use std::time::{Duration, Instant};

use anyhow::{Context as _, bail};
use chrono::{DateTime, Days, Local, NaiveTime, TimeZone};
use reqwest::blocking::Client;
use tracing::{debug, error, info, warn};

use crate::cache::CacheRepository as _;
use crate::commands::prelude::*;
use crate::config::{ConfigRepository as _, Daylight, GeoCoordinate};
use crate::state::{PauseState, StateRepository as _};
use crate::theme::Theme;

#[derive(Debug, Clone, clap::Args)]
pub struct Start {
    /// Time to wait between checks (in milliseconds)
    #[arg(long, default_value = "3000")]
    interval: u64,
}

impl Run for Start {
    fn run(&self, ctx: &mut Context) -> anyhow::Result<()> {
        let interval = Duration::from_millis(self.interval);

        loop {
            let next_tick = Instant::now() + interval;

            if let Err(err) = tick(ctx) {
                error!("tick failed: {err}");
            }

            sleep_until(next_tick);
        }
    }
}

fn tick(ctx: &mut Context) -> anyhow::Result<()> {
    let now = Local::now();

    if ctx.store().pause_state()?.is_paused(&now) {
        debug!("theme switcher is paused");
        return Ok(());
    }

    let coordinates = resolve_coordinates(ctx, &now)?;
    let daylight = coordinates
        .and_then(|coordinates| {
            resolve_daylight(ctx, &now, coordinates)
                .inspect_err(|err| warn!("failed to resolve daylight times: {err}"))
                .ok()
        })
        .unwrap_or(ctx.store().fallback().unwrap_or_default().daylight());

    let current_theme = Theme::from_system().unwrap_or_default();
    let target_theme = get_target_theme(daylight, &now);
    let cached_theme = ctx
        .store()
        .theme()
        .inspect_err(|err| warn!("failed to get cached theme: {err}"))
        .unwrap_or_default();

    if current_theme == target_theme {
        return Ok(());
    }

    debug!("updating theme: {current_theme} => {target_theme}");

    if is_manual_override(current_theme, target_theme, cached_theme) {
        info!("manual override detected");
        let next_theme_change = get_next_theme_change(daylight, &now);

        ctx.store()
            .set_pause_state(PauseState::Until(next_theme_change))
            .context("failed to pause for manual override")?;

        return Ok(());
    }

    // TODO: Consider doing a transaction/rollback system instead of resetting on failure.
    ctx.store()
        .set_theme(target_theme)
        .context("failed to cache next theme")?;

    if let Err(err) = set_system_theme(target_theme) {
        error!("failed to set theme: {err}");

        ctx.store()
            .set_theme(current_theme)
            .context("failed to revert cached theme after setting system theme failed")?;
    }

    Ok(())
}

/// Puts the current thread to sleep until at least the specified deadline has passed.
fn sleep_until(deadline: Instant) {
    let now = Instant::now();

    if now < deadline {
        thread::sleep(deadline - now);
    }
}

fn resolve_coordinates(
    ctx: &mut Context,
    now: &DateTime<Local>,
) -> anyhow::Result<Option<GeoCoordinate>> {
    if !ctx.store().location_enabled().unwrap_or(false) {
        return Ok(None);
    }

    if let Some(coordinates) = ctx.store().location().unwrap_or(None) {
        return Ok(Some(coordinates));
    }

    let was_attempted_today = ctx
        .store()
        .last_location_attempt()
        .context("failed to get last location attempt")?
        .is_some_and(|last_attempt| {
            // Attempt to get the user's location once per day.
            now.date_naive() == last_attempt.date_naive()
        });

    if was_attempted_today {
        bail!("already attempted to get geographic coordinates today");
    }

    ctx.store()
        .set_last_location_attempt(*now)
        .context("failed to cache last attempt")?;

    let coordinates = get_coordinates()?;

    ctx.store()
        .set_location(coordinates)
        .context("failed to set coordinates: {err}")?;

    Ok(Some(coordinates))
}

fn resolve_daylight(
    ctx: &mut Context,
    now: &DateTime<Local>,
    coordinates: GeoCoordinate,
) -> anyhow::Result<Daylight> {
    let was_updated_today = ctx
        .store()
        .last_updated_at()
        .context("failed to get last update timestamp")?
        .is_some_and(|last_update| last_update == now.date_naive());

    if was_updated_today
        && let Some(daylight) = ctx
            .store()
            .daylight()
            .context("failed to get cached daylight times")?
    {
        return Ok(daylight);
    }

    ctx.store()
        .set_last_updated_at(now.date_naive())
        .context("failed to cache last updated at: {err}")?;

    let daylight = get_daylight(coordinates)?;

    ctx.store()
        .set_daylight(daylight)
        .context("failed to cache daylight times: {err}")?;

    Ok(daylight)
}

/// Query external API to get the user's geographic location.
fn get_coordinates() -> anyhow::Result<GeoCoordinate> {
    #[derive(serde::Deserialize)]
    struct Response {
        longitude: f32,
        latitude: f32,
    }

    debug!("querying external API for geographic location");
    let response = Client::new()
        .get("https://freeipapi.com/api/json")
        .timeout(Duration::from_secs(10))
        .send()
        .context("failed to query API for geolocation: {err}")?;

    let text = response
        .text()
        .context("response was not valid UTF-8: {err}")?;

    let data =
        serde_json::from_str::<Response>(&text).context("failed to parse response: {err}")?;

    Ok(GeoCoordinate {
        longitude: data.longitude,
        latitude: data.latitude,
    })
}

/// Query external API for sunrise and sunset times.
fn get_daylight(coordinates: GeoCoordinate) -> anyhow::Result<Daylight> {
    #[derive(serde::Deserialize)]
    struct Response {
        results: Results,
    }

    #[derive(serde::Deserialize)]
    struct Results {
        sunrise: NaiveTime,
        sunset: NaiveTime,
    }

    let url = format!(
        "https://api.sunrisesunset.io/json?lng={longitude}&lat={latitude}&time_format=24",
        longitude = coordinates.longitude,
        latitude = coordinates.latitude
    );

    debug!("querying external API for sunrise/sunset times");
    let response = Client::new()
        .get(url)
        .timeout(Duration::from_secs(10))
        .send()
        .context("failed to query API for daylight: {err}")?;

    let text = response
        .text()
        .context("response was not valid UTF-8: {err}")?;

    let data =
        serde_json::from_str::<Response>(&text).context("failed to parse response: {err}")?;

    Ok(Daylight {
        sunrise: data.results.sunrise,
        sunset: data.results.sunset,
    })
}

fn get_target_theme(daylight: Daylight, now: &DateTime<Local>) -> Theme {
    if daylight.is_daytime(now.time()) {
        Theme::Light
    } else {
        Theme::Dark
    }
}

fn is_manual_override(current: Theme, target: Theme, cached: Theme) -> bool {
    current != target && target == cached
}

fn get_next_theme_change(daylight: Daylight, now: &DateTime<Local>) -> DateTime<Local> {
    let is_daytime = daylight.is_daytime(now.time());

    let time = if is_daytime {
        daylight.sunset
    } else {
        daylight.sunrise
    };

    let date = if is_daytime && daylight.sunset < daylight.sunrise {
        now.date_naive().checked_add_days(Days::new(1)).unwrap()
    } else {
        now.date_naive()
    };

    let datetime = date.and_time(time);
    Local.from_local_datetime(&datetime).unwrap()
}

/// Update the system theme.
fn set_system_theme(theme: Theme) -> anyhow::Result<()> {
    let status = Command::new("gsettings")
        .args([
            "set",
            "org.gnome.desktop.interface",
            "color-scheme",
            theme.as_color_scheme(),
        ])
        .status()
        .context("failed to execute gsettings")?;

    match status.code() {
        Some(0) => Ok(()),
        Some(code) => bail!("failed with exit code: {code}"),
        None => bail!("process terminated due to signal"),
    }
}
