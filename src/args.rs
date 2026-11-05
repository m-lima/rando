pub fn parse() -> Args {
    <Inner as clap::Parser>::parse().into()
}

#[derive(Debug, clap::Parser)]
struct Inner {
    /// Verbosity level
    #[arg(short, action = clap::ArgAction::Count)]
    verbosity: u8,

    /// Log external crates
    #[arg(short, long)]
    external: bool,

    /// Port on which to serve
    #[arg(short, long, default_value_t = unsafe { std::num::NonZeroU16::new_unchecked(7845) })]
    port: std::num::NonZeroU16,
}

#[derive(Debug)]
pub struct Args {
    pub verbosity: tracing::Level,
    pub external: bool,
    pub port: std::num::NonZeroU16,
}

impl From<Inner> for Args {
    fn from(value: Inner) -> Self {
        Self {
            verbosity: to_verbosity(value.verbosity),
            external: value.external,
            port: value.port,
        }
    }
}

fn to_verbosity(value: u8) -> tracing::Level {
    match value {
        0 => tracing::Level::ERROR,
        1 => tracing::Level::WARN,
        2 => tracing::Level::INFO,
        3 => tracing::Level::DEBUG,
        _ => tracing::Level::TRACE,
    }
}
