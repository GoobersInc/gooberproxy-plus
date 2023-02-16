use anyhow::{Context, Result};
use clap::Parser;
use std::path::PathBuf;

use crate::{app::App, config::Config};

mod app;
mod config;
mod conn;
mod join;
mod listener;
mod logging;

#[derive(Parser)]
struct CliArgs {
    #[arg(short, long, default_value = "config.toml")]
    config_path: PathBuf,

    #[arg(long, default_value = "false")]
    overwrite_config: bool,
}

#[tokio::main]
async fn main() -> Result<()> {
    let args = CliArgs::parse();
    let config_path = args.config_path;

    // Set up a global tracing listener
    logging::setup().context("Failed to set up logging")?;

    // Create a config file if it doesn't exist or if the user wants to overwrite it
    if !config_path.exists() || args.overwrite_config {
        if !config_path.exists() {
            eprintln!("Config file does not exist, creating one with default values");
        }
        let config = Config::default();
        config.save(&config_path).await?;
        return Ok(());
    }

    // Load the config
    let config = Config::load(&config_path)
        .await
        .context("Failed to load the config")?;

    // Initialize the app
    let mut app = App::init(config)
        .await
        .context("Failed to initialize the app")?;

    // Run le app
    app.run().await.context("Failed to run the app")?;

    Ok(())
}
