use anyhow::Result;
use azalea_chat::{text_component::TextComponent, FormattedText};
use serde::{Deserialize, Serialize};
use std::{
    net::{IpAddr, SocketAddr},
    path::PathBuf,
};

/// A filesystem-based configuration store
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Config {
    pub listen_addr: SocketAddr,
    pub server_addr: SocketAddr,
    pub account: String,
    pub player: String,
    pub motd: FormattedText,
}

impl Config {
    /// Load a configuration file from the filesystem
    pub async fn load(path: &PathBuf) -> Result<Self> {
        let file = tokio::fs::read_to_string(path).await?;
        let config = toml::from_str(&file)?;
        Ok(config)
    }

    /// Save the current configuration as a file to the filesystem
    pub async fn save(&self, path: &PathBuf) -> Result<()> {
        let file = toml::to_string(&self)?;
        tokio::fs::write(path, file).await?;
        Ok(())
    }
}

impl Default for Config {
    fn default() -> Self {
        Self {
            listen_addr: SocketAddr::new(IpAddr::from([0, 0, 0, 0]), 25565),
            server_addr: SocketAddr::new(IpAddr::from([127, 0, 0, 1]), 25566),
            account: "goober@example.com".to_string(),
            player: "LiveOvergoober".to_string(),
            motd: FormattedText::Text(TextComponent::new("A Terraria server.".to_string())),
        }
    }
}
