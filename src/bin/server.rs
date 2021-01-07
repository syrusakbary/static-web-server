#![deny(warnings)]

#[cfg(all(target_env = "musl", target_pointer_width = "64"))]
#[global_allocator]
static ALLOC: jemallocator::Jemalloc = jemallocator::Jemalloc;

extern crate static_web_server;

use structopt::StructOpt;
use tracing::warn;
use warp::Filter;

use self::static_web_server::core::*;

/// It creates a new server instance with given options.
async fn server(opts: config::Options) -> Result {
    logger::init(&opts.log_level)?;

    let public_head = warp::head().and(warp::fs::dir(opts.root.clone()));
    let public_get = warp::get().and(warp::fs::dir(opts.root));

    let routes = public_head.or(public_get.with(warp::compression::gzip()));

    let host = opts.host.parse::<std::net::IpAddr>()?;
    let port = opts.port;

    tokio::task::spawn(
        warp::serve(
            routes
                .with(warp::trace::request())
                .recover(rejection::handle_rejection),
        )
        .run((host, port)),
    );

    signals::wait(|sig: signals::Signal| {
        let code = signals::as_int(sig);
        warn!("Signal {} caught. Server execution exited.", code);
        std::process::exit(code)
    });

    Ok(())
}

#[tokio::main(max_threads = 10_000)]
async fn main() -> Result {
    server(config::Options::from_args()).await?;

    Ok(())
}
