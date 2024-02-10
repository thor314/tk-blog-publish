use std::{fs, path::Path};

use anyhow::{anyhow, Result};
use chrono::Local;
use clap::Parser;
use log::trace;

use crate::{cli::Cli, error::MyError, DATE_FORMAT};

/// Set up crate logging and environment variables.
pub(crate) fn setup() -> Result<Cli, MyError> {
  dotenv::dotenv().ok();
  env_logger::init();
  if std::env::var("DOTENV_OK").is_ok() {
    trace!("loaded dotenv");
  } else {
    return Err(anyhow!("failed to load dotenv").into());
  }

  let args = Cli::parse();
  Ok(args)
}

pub fn get_original_date(source: &Path, content: &str) -> Result<String, MyError> {
  let original_date = match content.lines().find(|line| line.starts_with("date: ")) {
    Some(l) => l.replace("date: ", ""),
    None => {
      log::info!(
        "could not find line starting with 'date: '; using file-last-modified date instead"
      );
      let metadata = fs::metadata(source)?;
      let system_time = metadata.created().unwrap();
      let datetime: chrono::DateTime<Local> = system_time.into();
      // Format the datetime to yyyy-mm-dd
      datetime.format(DATE_FORMAT).to_string()
    },
  };
  Ok(original_date)
}
