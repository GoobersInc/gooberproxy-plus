use tracing::subscriber::SetGlobalDefaultError;
use tracing_subscriber::FmtSubscriber;

/// Initializes a global `tracing` subscriber that logs everything in this crate
pub fn setup() -> Result<(), SetGlobalDefaultError> {
    let subscriber = FmtSubscriber::builder()
        .with_env_filter("gooberproxy_plus=trace")
        .with_file(false)
        .with_target(false)
        .finish();

    tracing::subscriber::set_global_default(subscriber)?;

    Ok(())
}
