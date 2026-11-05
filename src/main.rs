#![warn(clippy::pedantic)]

use anyhow::Context;

mod args;
mod logger;
mod server;
mod shutdown;

type Result<T = ()> = anyhow::Result<T>;

fn setup_tracing(args: &args::Args) -> Result {
    use tracing_subscriber::layer::SubscriberExt;

    let layer = treetrace::Layer::builder(treetrace::Stdout).build();
    let subscriber = tracing_subscriber::registry().with(layer).with(
        tracing::level_filters::LevelFilter::from_level(args.verbosity),
    );

    if args.external {
        tracing::subscriber::set_global_default(subscriber).map_err(Into::into)
    } else {
        let subscriber = subscriber.with(
            tracing_subscriber::filter::Targets::new()
                .with_target(env!("CARGO_CRATE_NAME"), args.verbosity),
        );

        tracing::subscriber::set_global_default(subscriber).map_err(Into::into)
    }
}

fn main() -> Result {
    let args = args::parse();

    setup_tracing(&args).context("while setting up tracing")?;

    tracing::info!(
        verbosity = %args.verbosity,
        external = %args.external,
        socket = %args.socket,
        "Initializing server"
    );

    let app = server::make_routes();

    let runtime = tokio::runtime::Builder::new_multi_thread()
        .enable_all()
        .build()
        .context("while building the runtime")?;

    tokio::runtime::Runtime::block_on(&runtime, async {
        let shutdown = shutdown::Shutdown::new()?;

        match &args.socket {
            args::Socket::Port(port) => {
                let address = std::net::SocketAddrV4::new(std::net::Ipv4Addr::UNSPECIFIED, *port);
                let listener = tokio::net::TcpListener::bind(address)
                    .await
                    .context("while binding to socket")?;
                axum::serve(listener, app.into_make_service())
                    .with_graceful_shutdown(shutdown)
                    .await
                    .context("while serving on socket")
            }
            args::Socket::Unix(path) => {
                let listener =
                    tokio::net::UnixListener::bind(path).context("while binding to socket")?;
                axum::serve(listener, app.into_make_service())
                    .with_graceful_shutdown(shutdown)
                    .await
                    .context("while serving on socket")
            }
        }
    })
    .context("while executing async context")
}
