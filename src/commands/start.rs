use std::process::Command;
use std::thread;
use std::time::{Duration, Instant};

use anyhow::{Context as _, bail};
use chrono::{DateTime, Days, Local, NaiveTime, TimeZone};
use reqwest::blocking::Client;
use tracing::{error, info, warn};

use crate::cache::CacheRepository;
use crate::commands::prelude::*;
use crate::config::{ConfigRepository, Daylight, GeoCoordinate};
use crate::state::{Pause, StateRepository};
use crate::store::FileStore;
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
            let now = Local::now();

            if let Err(err) = tick(ctx.store(), now) {
                error!("tick failed: {err}");

                for cause in err.chain() {
                    error!("  Cause: {cause}");
                }
            }

            sleep_until(next_tick);
        }
    }
}

#[derive(Debug, Clone, Copy)]
struct TickInput {
    now: DateTime<Local>,
    pause_state: Pause,
    current_theme: Theme,
    cached_theme: Theme,
    daylight: Daylight,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum TickAction {
    None,
    SetTheme {
        target_theme: Theme,
    },
    PauseUntil {
        current_theme: Theme,
        next_theme_change: DateTime<Local>,
    },
}

fn evaluate_tick(input: &TickInput) -> TickAction {
    if input.pause_state.is_paused(&input.now) {
        return TickAction::None;
    }

    let target_theme = get_target_theme(input.daylight, input.now.time());

    if input.current_theme == target_theme {
        return TickAction::None;
    }

    if is_manual_override(input.current_theme, target_theme, input.cached_theme) {
        let next_theme_change = get_next_theme_change(input.daylight, &input.now);
        return TickAction::PauseUntil {
            current_theme: input.current_theme,
            next_theme_change,
        };
    }

    TickAction::SetTheme { target_theme }
}

fn tick(store: &FileStore, now: DateTime<Local>) -> anyhow::Result<()> {
    let pause_state = store.pause().context("failed to get pause state")?;

    let coordinates = resolve_coordinates(store, &now)?;
    let daylight = coordinates
        .and_then(|coordinates| {
            resolve_daylight(store, &now, coordinates)
                .inspect_err(|err| warn!("failed to resolve daylight times: {err}"))
                .ok()
        })
        .unwrap_or(store.fallback().unwrap_or_default().daylight());

    let current_theme = Theme::from_system().unwrap_or_default();
    let cached_theme = store
        .theme()
        .inspect_err(|err| warn!("failed to get cached theme: {err}"))
        .unwrap_or_default();

    let input = TickInput {
        now,
        pause_state,
        current_theme,
        cached_theme,
        daylight,
    };

    match evaluate_tick(&input) {
        TickAction::None => {}
        TickAction::PauseUntil {
            current_theme,
            next_theme_change,
        } => {
            store
                .set_theme(current_theme)
                .context("failed to cache theme")?;

            store
                .set_pause(Pause::Until(next_theme_change))
                .context("failed to set pause state")?;
        }
        TickAction::SetTheme { target_theme } => {
            store
                .set_theme(target_theme)
                .context("failed to cache theme")?;

            if let Err(err) = set_system_theme(target_theme) {
                error!("failed to set theme: {err}");
                store.set_theme(current_theme)?;
            }

            if matches!(pause_state, Pause::Until(_)) {
                store
                    .set_pause(Pause::None)
                    .context("failed to set pause state")?;
            }
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

fn resolve_coordinates(
    store: &FileStore,
    now: &DateTime<Local>,
) -> anyhow::Result<Option<GeoCoordinate>> {
    if !store.location_enabled().unwrap_or(false) {
        return Ok(None);
    }

    if let Some(coordinates) = store.location().unwrap_or(None) {
        return Ok(Some(coordinates));
    }

    let was_attempted_today = store
        .last_location_attempt()
        .context("failed to get last location attempt")?
        .is_some_and(|last_attempt| {
            // Attempt to get the user's location once per day.
            now.date_naive() == last_attempt.date_naive()
        });

    if was_attempted_today {
        bail!("already attempted to get geographic coordinates today");
    }

    store
        .set_last_location_attempt(*now)
        .context("failed to cache last attempt")?;

    let coordinates = get_coordinates()?;

    store
        .set_location(coordinates)
        .context("failed to set coordinates: {err}")?;

    Ok(Some(coordinates))
}

fn resolve_daylight(
    store: &FileStore,
    now: &DateTime<Local>,
    coordinates: GeoCoordinate,
) -> anyhow::Result<Daylight> {
    let was_updated_today = store
        .last_updated_at()
        .context("failed to get last update timestamp")?
        .is_some_and(|last_update| last_update == now.date_naive());

    if was_updated_today
        && let Some(daylight) = store
            .daylight()
            .context("failed to get cached daylight times")?
    {
        return Ok(daylight);
    }

    store
        .set_last_updated_at(now.date_naive())
        .context("failed to cache last updated at: {err}")?;

    let daylight = get_daylight(coordinates)?;

    store
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

    info!("querying external API for geographic location");
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

    info!("querying external API for sunrise/sunset times");
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

fn get_target_theme(daylight: Daylight, time: NaiveTime) -> Theme {
    if daylight.is_daytime(time) {
        Theme::Light
    } else {
        Theme::Dark
    }
}

fn is_manual_override(current: Theme, target: Theme, cached: Theme) -> bool {
    // 18:00
    // - Current: Light
    // - Target: Light
    // - Cached: Light
    // 18:00 (override)
    // - Current = Dark
    // - Target = Light
    // - Cached = Light
    // 20:00
    // - Current: Light
    // - Target: Dark
    // - Cached: Light
    current != target && current != cached
}

fn get_next_theme_change(daylight: Daylight, now: &DateTime<Local>) -> DateTime<Local> {
    let is_daytime = daylight.is_daytime(now.time());

    let time = if is_daytime {
        daylight.sunset
    } else {
        daylight.sunrise
    };

    let date = if is_daytime && daylight.sunset_is_next_day() {
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

#[cfg(test)]
mod tests {
    use chrono::NaiveDate;

    use super::*;

    fn daylight() -> Daylight {
        Daylight {
            sunrise: NaiveTime::from_hms_opt(6, 30, 0).unwrap(),
            sunset: NaiveTime::from_hms_opt(18, 30, 0).unwrap(),
        }
    }

    #[test]
    fn before_sunrise_is_dark() {
        let time = NaiveTime::from_hms_opt(2, 0, 0).unwrap();
        let target_theme = get_target_theme(daylight(), time);
        assert_eq!(target_theme, Theme::Dark);
    }

    #[test]
    fn at_sunrise_is_light() {
        let time = NaiveTime::from_hms_opt(6, 30, 0).unwrap();
        let target_theme = get_target_theme(daylight(), time);
        assert_eq!(target_theme, Theme::Light);
    }

    #[test]
    fn after_sunrise_is_light() {
        let time = NaiveTime::from_hms_opt(10, 0, 0).unwrap();
        let target_theme = get_target_theme(daylight(), time);
        assert_eq!(target_theme, Theme::Light);
    }

    #[test]
    fn at_sunset_is_dark() {
        let time = NaiveTime::from_hms_opt(18, 30, 0).unwrap();
        let target_theme = get_target_theme(daylight(), time);
        assert_eq!(target_theme, Theme::Dark);
    }

    fn daylight_sunset_next_day() -> Daylight {
        Daylight {
            sunrise: NaiveTime::from_hms_opt(6, 30, 0).unwrap(),
            sunset: NaiveTime::from_hms_opt(2, 30, 0).unwrap(), // Sun sets on the next day
        }
    }

    #[test]
    fn at_midnight_sunset_next_day() {
        let time = NaiveTime::from_hms_opt(0, 0, 0).unwrap();
        let target_theme = get_target_theme(daylight_sunset_next_day(), time);
        assert_eq!(target_theme, Theme::Light);
    }

    fn expired_at_sunrise() -> DateTime<Local> {
        let Daylight { sunrise, sunset: _ } = daylight();

        let datetime = NaiveDate::from_ymd_opt(2026, 5, 29)
            .unwrap()
            .and_time(sunrise);

        Local.from_local_datetime(&datetime).unwrap()
    }

    // fn expired_at_sunset() -> DateTime<Local> {
    //     let Daylight { sunrise: _, sunset } = daylight();

    //     let datetime = NaiveDate::from_ymd_opt(2026, 5, 29)
    //         .unwrap()
    //         .and_time(sunset);

    //     Local.from_local_datetime(&datetime).unwrap()
    // }

    #[test]
    fn paused_state() {
        let input = TickInput {
            now: Local.with_ymd_and_hms(2026, 5, 29, 16, 0, 0).unwrap(),
            pause_state: Pause::Indefinite,
            current_theme: Theme::Dark,
            cached_theme: Theme::Dark,
            daylight: daylight(),
        };

        assert_eq!(evaluate_tick(&input), TickAction::None);
    }

    #[test]
    fn expired_pause() {
        let input = TickInput {
            now: Local.with_ymd_and_hms(2026, 5, 29, 8, 0, 0).unwrap(),
            pause_state: Pause::Until(expired_at_sunrise()),
            current_theme: Theme::Dark,
            cached_theme: Theme::Dark,
            daylight: daylight(),
        };

        let action = evaluate_tick(&input);

        assert_eq!(action, TickAction::SetTheme(Theme::Light));
    }

    #[test]
    fn manual_override() {
        let input = TickInput {
            now: Local.with_ymd_and_hms(2026, 5, 29, 16, 0, 0).unwrap(),
            pause_state: Pause::None,
            current_theme: Theme::Dark,
            cached_theme: Theme::Light,
            daylight: Daylight {
                sunrise: NaiveTime::from_hms_opt(6, 30, 0).unwrap(),
                sunset: NaiveTime::from_hms_opt(18, 30, 0).unwrap(),
            },
        };

        let action = evaluate_tick(&input);

        assert!(matches!(action, TickAction::PauseUntil(_)));
    }
}
