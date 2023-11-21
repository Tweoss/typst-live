use argh::FromArgs;
use axum::{routing::get, Router, Server};
use eyre::Result;
use state::ServerState;
use std::{fs, sync::Arc};
use tempdir::TempDir;
use tokio::{
    runtime::Runtime,
    signal::ctrl_c,
    sync::{broadcast, Notify},
};

mod routes;
mod state;
mod watcher;

#[derive(FromArgs)]
/// hot reloading for typst.
struct Args {
    #[argh(positional)]
    /// specifies file to recompile when changes are made. If `--watch` is used it should be pdf file.
    filename: String,
    #[argh(option, default = "String::from(\".\")")]
    /// specifies the root directory for typst's file paths
    root: String,
    #[argh(option, short = 'A', default = "String::from(\"127.0.0.1\")")]
    /// specifies the listen address. Defaults to 127.0.0.1
    address: String,
    #[argh(option, short = 'P', default = "5599")]
    /// specifies the port to listen on. Defaults to 5599
    port: u16,
}

async fn run(state: Arc<ServerState>) -> Result<()> {
    let router = Router::new()
        .route("/", get(routes::root))
        .route("/:file", get(routes::target))
        .route("/listen", get(routes::listen))
        .with_state(state.clone());

    let addr = format!("{}:{}", state.args.address, state.args.port);
    let server = Server::bind(&addr.parse()?).serve(router.into_make_service());

    println!(
        "[INFO] Server is listening on http://{}/",
        server.local_addr()
    );
    open::that_detached(format!("http://{}", server.local_addr()))?;

    tokio::select! {
        _ = server => {},
        _ = state.shutdown.notified() => {},
    };

    Ok(())
}

fn main() -> Result<()> {
    #[cfg(debug_assertions)]
    std::env::set_var("RUST_LOG", "hyper=error,debug");

    let args: Args = argh::from_env();

    if fs::metadata(&args.filename).is_err() {
        println!("[ERR] File `{}` doesn't exist", args.filename);
        return Ok(());
    }

    let temp_dir = TempDir::new("typst-watch")?;

    let tokio = Runtime::new()?;
    let (tx, _) = broadcast::channel(8);
    let state = Arc::new(ServerState {
        shutdown: Notify::new(),
        changed: tx,
        tokio,
        args,
        directory: temp_dir,
    });

    state.tokio.spawn(graceful_shutdown(state.clone()));

    let watcher = state
        .tokio
        .block_on(watcher::setup_watching_typst(state.clone()))?;
    state.tokio.block_on(run(state.clone()))?;
    drop(watcher);

    Ok(())
}

async fn graceful_shutdown(state: Arc<ServerState>) {
    ctrl_c().await.unwrap();

    // Reset to prevent ^C from appearing.
    print!("\r");

    state.shutdown.notify_waiters();
}
