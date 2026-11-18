use crate::server::state::AppState;
use crate::server::{self, RunningServer};
use crate::tunnel;
use tokio::process::Child;

/// Starts and stops the web server and the tunnel.
pub struct Runtime {
    state: AppState,
    server: Option<RunningServer>,
    tunnel: Option<Child>,
}

impl Runtime {
    pub fn new(state: AppState) -> Self {
        Self {
            state,
            server: None,
            tunnel: None,
        }
    }

    pub fn state(&self) -> &AppState {
        &self.state
    }

    pub fn server_running(&self) -> bool {
        self.server.is_some()
    }

    pub fn tunnel_running(&mut self) -> bool {
        match self.tunnel.as_mut() {
            Some(child) => matches!(child.try_wait(), Ok(None)),
            None => false,
        }
    }

    pub async fn start(&mut self) {
        self.state.refresh_storage_root();
        let port = self.state.port();
        match server::start(self.state.clone(), port).await {
            Ok(running) => {
                self.server = Some(running);
                println!("Server is running at http://localhost:{port}");
                self.tunnel = tunnel::start(port, self.state.tunnel_link.clone());
            }
            Err(message) => println!("{message}"),
        }
    }

    pub async fn stop(&mut self) {
        if let Some(child) = self.tunnel.take() {
            tunnel::stop(child, &self.state.tunnel_link).await;
        }
        if let Some(running) = self.server.take() {
            server::stop(running).await;
        }
    }

    pub async fn restart(&mut self) {
        println!("Restarting...");
        self.stop().await;
        self.start().await;
    }
}
