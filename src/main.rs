#![warn(clippy::pedantic)]

use anyhow::Context;

mod args;
mod logger;
mod server;

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
        port = %args.port,
        "Initializing server"
    );

    let app = server::make_routes();

    let runtime = tokio::runtime::Builder::new_multi_thread()
        .enable_all()
        .build()
        .context("while building the runtime")?;

    tokio::runtime::Runtime::block_on(&runtime, async {
        let listener = tokio::net::TcpListener::bind(("0.0.0.0", u16::from(args.port)))
            .await
            .context("while binding to socket")?;
        axum::serve(listener, app.into_make_service())
            .await
            .context("while serving on socket")
    })
    .context("while executing async context")
}
