use std::{
  fs,
  io::Write,
  path::{Path, PathBuf},
};

use anyhow::{anyhow, Context};
use clap::{Args, Parser};
use log::info;

use crate::{error::MyError, Config, FilePair, CONFIG_FILE_PATH};

#[derive(Parser, Debug)]
#[clap(version = "1.0", author = "Thor Kamphefner")]
#[command(name = "tk-blog-publish")]
#[command(bin_name = "tk-blog-publish")]
#[command(about = "a tool to publish my blog posts from my Obsidian vault to my website")]
pub enum Cli {
  /// Just update a one-off file-mapping
  #[clap(name = "one")]
  One(Mapping),
  /// Update from config file
  #[clap(name = "all")]
  All(ConfigPath),
  /// Add a new path to update to the config file
  #[clap(name = "add")]
  ConfigAdd(ConfigAdd),
  /// Add a new path to update to the config file
  #[clap(name = "remove")]
  ConfigRemove(ConfigAdd),
}

#[derive(Args, Debug)]
pub struct Mapping {
  /// Sets the source file to use.
  #[arg(index = 1)]
  pub source_path: PathBuf,
  /// Sets the target file to use. If none provided, assume we are publisihng to the blog.
  #[arg(index = 2)]
  pub target_path: Option<PathBuf>,
}

/// Sets a custom config file
#[derive(Args, Debug)]
pub struct ConfigPath {
  #[arg(index = 1)]
  pub config: Option<PathBuf>,
}

#[derive(Args, Debug)]
pub struct ConfigAdd {
  /// Sets the source file to use.
  #[arg(index = 1)]
  pub source_path: PathBuf,
  /// Sets the target file to use. If none provided, assume we are publisihng to the blog.
  #[arg(index = 2)]
  pub target_path: Option<PathBuf>,
  #[arg(short, long)]
  pub config:      Option<PathBuf>,
  #[arg(short, long, default_value = "false")]
  pub private:     bool,
}

impl ConfigAdd {
  /// add a file to config file
  pub fn add_file(
    &self,
  ) -> Result<(), MyError> {
    let config_path: PathBuf =
      self.config.clone().unwrap_or_else(|| PathBuf::from(CONFIG_FILE_PATH));
    let config_content = fs::read_to_string(&config_path).context("Could not read config file")?;
    let mut config: Config =
      toml::from_str(&config_content).context("Could not parse config file")?;

    // Assert that source path exists
    if !self.source_path.exists() {
      return Err(anyhow!("Source path does not exist").into());
    }
    // Replace relative paths with absolute paths
    let absolute_source_path = fs::canonicalize(&self.source_path)
      .with_context(|| format!("Could not canonicalize source path {:?}", self.source_path))?;
    let absolute_target_path =
      &self.target_path.clone().map(|path| fs::canonicalize(path).unwrap());

    // Check that source path is not already in config
    if config.files.iter().any(|fp| fp.source == absolute_source_path) {
      return Err(anyhow::anyhow!("Source path already exists in config").into());
    }
    println!("Adding file to config: {:?} {:?}", absolute_source_path, absolute_target_path);

    // Create a new file pair
    let new_file_pair = FilePair { source: absolute_source_path, target: absolute_target_path.clone()};
    // Add the new file pair to the configuration
    config.files.push(new_file_pair);
    // Serialize the updated configuration back to TOML
    let updated_config = toml::to_string(&config).context("Could not serialize config")?;
    // Write the updated TOML content back to the configuration file
    let mut file = fs::File::create(&config_path).context("Could not write to config file")?;
    file.write_all(updated_config.as_bytes()).context("Could not write config file")?;

    Ok(())
  }
}
