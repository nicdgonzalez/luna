use std::fmt;
use std::process::Command;
use std::str::FromStr;

use anyhow::{Context as _, anyhow};
use tracing::warn;

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub enum Theme {
    #[default]
    Light,
    Dark,
}

impl Theme {
    pub fn from_system() -> anyhow::Result<Self> {
        let output = Command::new("gsettings")
            .args(["get", "org.gnome.desktop.interface", "color-scheme"])
            .output()
            .context("failed to execute gsettings")?;

        let stdout = String::from_utf8(output.stdout).context("output is not valid UTF-8")?;
        let color_scheme = stdout.trim().trim_matches('\'').to_lowercase();

        Ok(color_scheme
            .parse::<Self>()
            .inspect_err(|err| warn!("{err}"))
            .unwrap_or_default())
    }

    pub fn as_color_scheme(self) -> &'static str {
        match self {
            Self::Light => "default",
            Self::Dark => "prefer-dark",
        }
    }
}

impl FromStr for Theme {
    type Err = anyhow::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "default" => Ok(Self::Light),
            "prefer-dark" => Ok(Self::Dark),
            other => Err(anyhow!("unknown gsettings color-scheme: {other}")),
        }
    }
}

impl fmt::Display for Theme {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match *self {
            Self::Light => "Light".fmt(f),
            Self::Dark => "Dark".fmt(f),
        }
    }
}
