use crate::state::ServerState;
use eyre::Result;
use notify::{Event, EventKind, RecommendedWatcher, RecursiveMode, Watcher};
use std::sync::Arc;
use tokio::process::Command;

pub async fn setup_watching_typst(state: Arc<ServerState>) -> Result<RecommendedWatcher> {
    match Command::new("typst")
        .arg("watch")
        .arg(&state.args.filename)
        .arg(state.directory.path().join("output_{n}.svg"))
        .spawn()
    {
        Ok(child) => {
            state.tokio.spawn(async move {
                match child.wait_with_output().await {
                    Ok(out) if !out.status.success() => {
                        println!("[ERR] Typst exited with error code: {}", out.status)
                    }
                    Err(err) => println!("[ERR] Typst exited with error: {err:?}"),
                    _ => {}
                }
            });
        }
        Err(err) => println!("[ERR] Failed to spawn the typst {err:?}"),
    }

    let tx = state.changed.clone();
    let mut watcher = notify::recommended_watcher(move |e: Result<Event, _>| match e {
        Ok(e) if matches!(e.kind, EventKind::Modify(_)) => {
            for path in e.paths {
                let path = path.file_name().unwrap().to_string_lossy();
                if path.starts_with("output_") && path.ends_with(".svg") {
                    let _ = tx.send(
                        path.strip_prefix("output_")
                            .unwrap()
                            .strip_suffix(".svg")
                            .unwrap()
                            .parse()
                            .unwrap(),
                    );
                }
            }
        }
        Err(err) => println!("[ERR] {err}"),
        _ => {}
    })?;
    watcher.watch(state.directory.path(), RecursiveMode::Recursive)?;

    Ok(watcher)
}
