use std::thread;
use std::time::{Duration, Instant};

use chrono::{Local, NaiveTime};
use reqwest::blocking::Client;
use tracing::{error, warn};

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
            let now = Local::now();

            let state = ctx.state_mut();
            state.reload(); // Ensure we have the latest changes.

            if state.is_paused(&now) {
                sleep_until(next_tick);
                continue;
            }

            let coordinates = ctx
                .config()
                .location()
                .is_enabled()
                .then_some(ctx.config().coordinates())
                .flatten()
                .or_else(|| {
                    if ctx
                        .cache()
                        .last_location_attempt()
                        .is_some_and(|last_attempt| {
                            // Attempt to get the user's location once per day.
                            now.date_naive() == last_attempt.date_naive()
                        })
                    {
                        return None;
                    }

                    if let Err(err) = ctx.cache_mut().set_last_location_attempt(now) {
                        error!("failed to cache last location attempt: {err}");
                        return None; // Don't ping the API if we cannot track our attempt.
                    }

                    let coordinates = get_coordinates()?;

                    if let Err(err) = ctx.config_mut().set_coordinates(coordinates) {
                        error!("failed to set coordinates: {err}");
                    }

                    Some(coordinates)
                });

            let daylight = if let Some(coordinates) = coordinates {
                ctx.cache()
                    .daylight()
                    .or_else(|| {
                        if let Err(err) = ctx.cache_mut().set_last_updated_at(&now) {
                            error!("failed to cache last updated at: {err}");
                            return None; // Don't ping the API if we cannot track our attempt.
                        }

                        let daylight = get_daylight(coordinates)?;

                        if let Err(err) = ctx.cache_mut().set_daylight(daylight) {
                            error!("failed to cache daylight times: {err}");
                        }

                        Some(daylight)
                    })
                    .unwrap_or(ctx.config().fallback().daylight())
            } else {
                ctx.config().fallback().daylight()
            };

            let current_theme = Theme::from_system().unwrap_or_default();

            if daylight.is_daytime(now.time()) {
                if current_theme != Theme::Light {
                    // Set theme
                }
            } else {
                if current_theme != Theme::Dark {
                    // Set theme
                }
            }

            sleep_until(next_tick);
        }
    }
}

/// Puts the current thread to sleep until at least the specified deadline has passed.
fn sleep_until(deadline: Instant) {
    let now = Instant::now();

    if now < deadline {
        thread::sleep(deadline - now);
    }
}

/// Query external API to get the user's geographic location.
fn get_coordinates() -> Option<GeoCoordinate> {
    #[derive(serde::Deserialize)]
    struct Response {
        longitude: f32,
        latitude: f32,
    }

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
