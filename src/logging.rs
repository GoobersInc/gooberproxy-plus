use tracing::subscriber::SetGlobalDefaultError;
use tracing_subscriber::FmtSubscriber;

pub fn setup() -> Result<(), SetGlobalDefaultError> {
    let subscriber = FmtSubscriber::builder()
        .with_env_filter("gooberproxy_plus=trace")
        .with_file(false)
        .with_target(false)
        .finish();

    tracing::subscriber::set_global_default(subscriber)?;

    return Ok(());
}
