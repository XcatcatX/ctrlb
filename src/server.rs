pub mod auth;
pub mod download_api;
pub mod download_headers;
pub mod list_api;
pub mod pages;
pub mod pins_api;
pub mod replies;
pub mod routes;
pub mod state;
pub mod temp_reader;
pub mod upload_api;

use state::AppState;
use std::time::Duration;
use tokio::net::TcpListener;
use tokio::sync::oneshot;
use tokio::task::JoinHandle;

pub struct RunningServer {
    shutdown: oneshot::Sender<()>,
    task: JoinHandle<()>,
}

/// Starts the web server on the given port. Only this computer can connect directly.
pub async fn start(state: AppState, port: u16) -> Result<RunningServer, String> {
    let root = state.storage_root();
    tokio::fs::create_dir_all(&root)
        .await
        .map_err(|error| format!("Cannot create the storage folder: {error}"))?;
    let listener = TcpListener::bind(("127.0.0.1", port)).await.map_err(|error| {
        format!("Cannot use port {port}: {error}\nUse \"set port <number>\", then \"restart\".")
    })?;
    let app = routes::build_router(state);
    let (shutdown, shutdown_receiver) = oneshot::channel::<()>();
    let task = tokio::spawn(async move {
        let result = axum::serve(listener, app)
            .with_graceful_shutdown(async move {
                let _ = shutdown_receiver.await;
            })
            .await;
        if let Err(error) = result {
            println!("Server error: {error}");
        }
    });
    Ok(RunningServer { shutdown, task })
}

/// Stops the server. Open connections get 3 seconds, then they are cut.
pub async fn stop(server: RunningServer) {
    let _ = server.shutdown.send(());
    let mut task = server.task;
    if tokio::time::timeout(Duration::from_secs(3), &mut task)
        .await
        .is_err()
    {
        task.abort();
    }
}
