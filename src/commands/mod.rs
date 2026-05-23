mod start;

pub trait Run {
    fn run(&self) -> anyhow::Result<()>;
}

#[derive(Debug, Clone, clap::Subcommand)]
pub enum Subcommand {
    /// Start the automatic theme switcher
    Start(start::Start),
}

impl Subcommand {
    pub fn run(&self) -> anyhow::Result<()> {
        match *self {
            Self::Start(ref inner) => inner.run(),
        }
    }
}

pub mod prelude {
    pub use super::Run;
}
