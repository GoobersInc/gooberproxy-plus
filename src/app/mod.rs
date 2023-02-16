use anyhow::{Context, Result};
use tokio::net::TcpListener;
use tracing::info;

use crate::config::Config;

mod conn_handler;
mod keep_alive;

#[derive(Clone)]
pub struct App {
    pub config: Config,
}

impl App {
    pub async fn init(config: Config) -> Result<Self> {
        Ok(Self { config })
    }

    pub async fn run(&mut self) -> Result<()> {
        let listener = TcpListener::bind(&self.config.listen_addr)
            .await
            .context("Failed to bind to socket")?;

        info!("Listening on {}", listener.local_addr()?);
        self.listen_for_connections(listener)
            .await
            .context("Failed to listen for connections (what)")?;

        Ok(())
    }
}
