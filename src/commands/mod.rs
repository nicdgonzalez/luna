use crate::context::Context;

mod disable;
mod enable;
mod start;

pub trait Run {
    fn run(&self, ctx: &mut Context) -> anyhow::Result<()>;
}

#[derive(Debug, Clone, clap::Subcommand)]
pub enum Subcommand {
    /// Start the automatic theme switcher
    Start(start::Start),

    /// Turn on the automatic theme switcher
    Enable(enable::Enable),

    /// Turn off the automatic theme switcher
    Disable(disable::Disable),
}

impl Subcommand {
    pub fn run(&self, mut ctx: Context) -> anyhow::Result<()> {
        match *self {
            Self::Start(ref inner) => inner.run(&mut ctx),
            Self::Enable(ref inner) => inner.run(&mut ctx),
            Self::Disable(ref inner) => inner.run(&mut ctx),
        }
    }
}

pub mod prelude {
    pub use crate::commands::Run;
    pub use crate::context::Context;
}
