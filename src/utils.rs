use anyhow::{anyhow, Result};
use clap::Parser;
pub use cli::Cli;
use log::trace;

use crate::error::MyError;

mod cli;

/// Set up crate logging and environment variables.
pub(crate) fn setup() -> Result<Cli, MyError> {
  dotenv::dotenv().ok();
  env_logger::init();
  // tracing_init::init_tracing(); // async alternative
  if std::env::var("DOTENV_OK").is_ok() {
    trace!("loaded dotenv");
  } else {
    return Err(anyhow!("failed to load dotenv").into());
  }

  let args = cli::Cli::parse();
  Ok(args)
}

// mod tracing_init {
//   /// Set up the tracing filter level using the env value, or else set it here. Reads RUST_LOG.
//   /// TRACE < DEBUG < INFO < WARN < ERROR
//   #[tracing::instrument]
//   pub(crate) fn init_tracing() {
//     let filter = tracing::level_filters::LevelFilter::INFO.into();
//     // set level to RUST_LOG env variable, or else INFO
//     let filter =
//       tracing_subscriber::EnvFilter::builder().with_default_directive(filter).from_env_lossy();
//     //  .with_level(false) // don't include levels in formatted output
//     //  .with_target(false) // don't include targets
//     //  .with_thread_ids(true) // include the thread ID of the current thread
//     //  .with_thread_names(true) // include the name of the current thread
//     //  .compact(); // use the `Compact` formatting style.
//     tracing_subscriber::fmt().with_env_filter(filter).init();
//   }
// }
