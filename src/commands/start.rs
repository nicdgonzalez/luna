use std::process::Command;
use std::thread;
use std::time::{Duration, Instant};

use anyhow::{Context as _, bail};
use chrono::{DateTime, Days, Local, NaiveTime, TimeZone};
use reqwest::blocking::Client;
use tracing::{debug, error, warn};

use crate::cache::Daylight;
use crate::commands::prelude::*;
use crate::config::GeoCoordinate;
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

    ctx.state_mut().reload(); // Ensure we have the latest changes.

    if ctx.state().is_paused(&now) {
        debug!("theme switcher is paused");
        return Ok(());
    }

    ctx.config_mut().reload();
    ctx.cache_mut().reload();

    let coordinates = resolve_coordinates(ctx, &now);
    let daylight = coordinates
        .and_then(|coordinates| resolve_daylight(ctx, &now, coordinates))
        .unwrap_or(ctx.config().fallback().daylight());

    let current_theme = Theme::from_system().unwrap_or_default();
    let target_theme = get_target_theme(daylight, &now);

    if current_theme == target_theme {
        return Ok(());
    }

    debug!("updating theme: {current_theme} => {target_theme}");

    if is_manual_override(current_theme, target_theme, ctx.state().theme()) {
        debug!("manual override detected");
        let next_theme_change = get_next_theme_change(daylight, &now);

        ctx.state_mut()
            .pause_until(next_theme_change)
            .context("failed to pause for manual override")?;

        return Ok(());
    }

    if let Err(err) = ctx.state_mut().set_theme(target_theme) {
        error!("failed to cache current theme: {err}");
        warn!("manual override until next theme change may not work properly");
    }

    if let Err(err) = set_theme(target_theme) {
        error!("failed to set theme: {err}");

        if let Err(err) = ctx.state_mut().set_theme(current_theme) {
            bail!("failed to revert cached current theme: {err}");
        }
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

fn resolve_coordinates(ctx: &mut Context, now: &DateTime<Local>) -> Option<GeoCoordinate> {
    if !ctx.config().location().is_enabled() {
        return None;
    }

    if let Some(coordinates) = ctx.config().coordinates() {
        return Some(coordinates);
    }

    let was_attempted_today = ctx
        .cache()
        .last_location_attempt()
        .is_some_and(|last_attempt| {
            // Attempt to get the user's location once per day.
            now.date_naive() == last_attempt.date_naive()
        });

    if was_attempted_today {
        return None;
    }

    if let Err(err) = ctx.cache_mut().set_last_location_attempt(*now) {
        error!("failed to cache last location attempt: {err}");
        return None; // Don't ping the API if we cannot track our attempt.
    }

    let coordinates = get_coordinates()?;

    if let Err(err) = ctx.config_mut().set_coordinates(coordinates) {
        error!("failed to set coordinates: {err}");
    }

    Some(coordinates)
}

fn resolve_daylight(
    ctx: &mut Context,
    now: &DateTime<Local>,
    coordinates: GeoCoordinate,
) -> Option<Daylight> {
    let was_updated_today = now.date_naive() == ctx.cache().last_updated_at().date_naive();

    if was_updated_today && let Some(daylight) = ctx.cache().daylight() {
        return Some(daylight);
    }

    if let Err(err) = ctx.cache_mut().set_last_updated_at(now) {
        error!("failed to cache last updated at: {err}");
        return None; // Don't ping the API if we cannot track our attempt.
    }

    let daylight = get_daylight(coordinates)?;

    if let Err(err) = ctx.cache_mut().set_daylight(daylight) {
        error!("failed to cache daylight times: {err}");
    }

    Some(daylight)
}

/// Query external API to get the user's geographic location.
fn get_coordinates() -> Option<GeoCoordinate> {
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
        .inspect_err(|err| warn!("failed to query API for geolocation: {err}"))
        .ok()?;

    let text = response
        .text()
        .inspect_err(|err| warn!("response was not valid UTF-8: {err}"))
        .ok()?;

    let data = serde_json::from_str::<Response>(&text)
        .inspect_err(|err| warn!("failed to parse response: {err}"))
        .ok()?;

    Some(GeoCoordinate {
        longitude: data.longitude,
        latitude: data.latitude,
    })
}

/// Query external API for sunrise and sunset times.
fn get_daylight(coordinates: GeoCoordinate) -> Option<Daylight> {
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
        .inspect_err(|err| warn!("failed to query API for daylight: {err}"))
        .ok()?;

    let text = response
        .text()
        .inspect_err(|err| warn!("response was not valid UTF-8: {err}"))
        .ok()?;

    let data = serde_json::from_str::<Response>(&text)
        .inspect_err(|err| warn!("failed to parse response: {err}"))
        .ok()?;

    Some(Daylight {
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

    let &time = if is_daytime {
        daylight.sunset()
    } else {
        daylight.sunrise()
    };

    let date = if is_daytime && daylight.sunset() < daylight.sunrise() {
        now.date_naive().checked_add_days(Days::new(1)).unwrap()
    } else {
        now.date_naive()
    };

    let datetime = date.and_time(time);
    Local.from_local_datetime(&datetime).unwrap()
}

/// Update the system theme.
fn set_theme(theme: Theme) -> anyhow::Result<()> {
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
