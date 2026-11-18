/// Waits until the user closes the program (Ctrl+C or closing the window).
#[cfg(unix)]
pub async fn wait_for_exit_signal() {
    use tokio::signal::unix::{SignalKind, signal};
    let Ok(mut terminate) = signal(SignalKind::terminate()) else {
        let _ = tokio::signal::ctrl_c().await;
        return;
    };
    tokio::select! {
        _ = tokio::signal::ctrl_c() => {}
        _ = terminate.recv() => {}
    }
}

#[cfg(windows)]
pub async fn wait_for_exit_signal() {
    use tokio::signal::windows::ctrl_close;
    let Ok(mut close) = ctrl_close() else {
        let _ = tokio::signal::ctrl_c().await;
        return;
    };
    tokio::select! {
        _ = tokio::signal::ctrl_c() => {}
        _ = close.recv() => {}
    }
}
