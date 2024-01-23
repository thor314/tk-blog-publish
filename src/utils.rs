use anyhow::{anyhow, Result};
use clap::Parser;
use log::trace;

use crate::{cli::Cli, error::MyError};

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

  let args = Cli::parse();
  Ok(args)
}
