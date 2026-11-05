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

    /// Location to serve on either a port for serving on TCP, or a path to a Unix Domain Socket.
    ///
    /// If the argument starts with `unix:`, it will be interpreted as a path.
    /// Otherwise, it will be interpreted as a port
    #[arg(short, long, default_value_t = Socket::Port(80), value_parser = parse_socket)]
    socket: Socket,
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

#[derive(Debug, Clone)]
pub enum Socket {
    Port(u16),
    Unix(std::path::PathBuf),
}

impl Drop for Socket {
    fn drop(&mut self) {
        match self {
            Socket::Port(_) => {}
            Socket::Unix(path) => {
                if let Err(error) = std::fs::remove_file(&path) {
                    tracing::error!(%error, "Could not remove unix socket `{}`", path.display());
                }
            }
        }
    }
}

impl std::fmt::Display for Socket {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Socket::Port(port) => port.fmt(f),
            Socket::Unix(path) => path.display().fmt(f),
        }
    }
}

fn parse_socket(input: &str) -> Result<Socket, &'static str> {
    if let Some(path) = input.strip_prefix("unix:") {
        Ok(Socket::Unix(std::path::PathBuf::from(path)))
    } else {
        match input.parse() {
            Ok(port @ 1..) => Ok(Socket::Port(port)),
            _ => Err("Invalid port"),
        }
    }
}

#[derive(Debug)]
pub struct Args {
    pub verbosity: tracing::Level,
    pub external: bool,
    pub socket: Socket,
}

impl From<Inner> for Args {
    fn from(value: Inner) -> Self {
        Self {
            verbosity: to_verbosity(value.verbosity),
            external: value.external,
            socket: value.socket,
        }
    }
}
