use std::{
  fs,
  io::Write,
  path::{Path, PathBuf},
};

use anyhow::{anyhow, Context};
use clap::{Args, Parser};
use log::{debug, info};

use crate::{
  error::MyError, Config, FilePair, CONFIG_FILE_PATH, DEFAULT_PRIVATE_TARGET_PATH_STR,
  DEFAULT_TARGET_PATH_STR,
};

#[derive(Parser, Debug)]
#[clap(version = "1.0", author = "Thor Kamphefner")]
#[command(name = "tk-blog-publish")]
#[command(bin_name = "tk-blog-publish")]
#[command(about = "a tool to publish my blog posts from my Obsidian vault to my website")]
pub enum Cli {
  /// Update from config file
  #[clap(name = "all")]
  Update(ConfigPath),
  /// Add a new path to update to the config file
  #[clap(name = "add")]
  Add(AddRemove),
  /// Add a new path to update to the config file
  #[clap(name = "remove")]
  Remove(AddRemove),
}

/// Sets a custom config file
#[derive(Args, Debug)]
pub struct ConfigPath {
  #[arg(index = 1)]
  pub config: Option<PathBuf>,
}

impl ConfigPath {
  /// parse config file and update all files
  pub fn update_files(&self) -> Result<(), MyError> {
    let config_path: PathBuf =
      self.config.clone().unwrap_or_else(|| PathBuf::from(CONFIG_FILE_PATH));
    let config_content = fs::read_to_string(config_path).expect("could not read config file");
    let config: Config = toml::from_str(&config_content).expect("Could not parse config file");
    debug!("updating files: {config:?}");

    for file_pair in config.files {
      file_pair.update_file()?;
    }
    Ok(())
  }
}

/// Arguments to add or remove a file
#[derive(Args, Debug)]
pub struct AddRemove {
  /// Sets the source file to use.
  #[arg(index = 1)]
  pub source_path:   PathBuf,
  /// Sets the target file to use. If none provided, assume we are publisihng to the blog.
  #[arg(index = 2)]
  pub target_path:   Option<PathBuf>,
  #[arg(short, long)]
  pub config:        Option<PathBuf>,
  #[arg(short, long, default_value = "false")]
  pub private:       bool,
  #[arg(short, long, default_value = "false")]
  pub update_images: bool,
}

impl AddRemove {
  /// add a file to config file
  pub fn add_file(&self) -> Result<(), MyError> {
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
      &self.target_path.clone().map(|path| fs::canonicalize(path).unwrap()).unwrap_or_else(|| {
        let path_str = absolute_source_path.to_str().unwrap();
        let filename = path_str.rsplit_once('/').unwrap_or(("", path_str)).1;
        let content = fs::read_to_string(&absolute_source_path).unwrap();
        let date = crate::get_original_date(&absolute_source_path, &content).unwrap();
        if self.private {
          format!("{DEFAULT_PRIVATE_TARGET_PATH_STR}/{date}-{filename}").into()
        } else {
          format!("{DEFAULT_TARGET_PATH_STR}/{date}-{filename}").into()
        }
      });

    // Check that source path is not already in config
    if config.files.iter().any(|fp| fp.source == absolute_source_path) {
      return Err(anyhow::anyhow!("Source path already exists in config").into());
    }
    println!("Adding file to config: {:?} {:?}", absolute_source_path, absolute_target_path);

    // Create a new file pair
    let new_file_pair =
      FilePair { source: absolute_source_path, target: absolute_target_path.clone() };
    // Add the new file pair to the configuration
    config.files.push(new_file_pair);
    // Serialize the updated configuration back to TOML
    let updated_config = toml::to_string(&config).context("Could not serialize config")?;
    // Write the updated TOML content back to the configuration file
    let mut file = fs::File::create(&config_path).context("Could not write to config file")?;
    file.write_all(updated_config.as_bytes()).context("Could not write config file")?;

    Ok(())
  }

  pub fn remove_file(&self) -> Result<(), MyError> {
    let config_path: PathBuf =
      self.config.clone().unwrap_or_else(|| PathBuf::from(CONFIG_FILE_PATH));
    let config_content =
      fs::read_to_string(config_path.clone()).expect("could not read config file");
    let mut config: Config = toml::from_str(&config_content).expect("Could not parse config file");

    // Look for any file matching end of source_path name and remove it
    let file_name = self
      .source_path
      .file_name()
      .ok_or_else(|| anyhow::anyhow!("Invalid source path"))?
      .to_str()
      .ok_or_else(|| anyhow::anyhow!("Non-UTF8 source file name"))?;

    let initial_len = config.files.len();
    config.files.retain(|fp| fp.source.file_name().map_or(false, |name| name != file_name));

    if config.files.len() == initial_len {
      // No file was removed, indicating the file was not found
      return Err(anyhow::anyhow!("Source file not found in config").into());
    }

    info!("Removing file from config: {:?}", self.source_path);

    // Serialize the updated configuration back to TOML
    let updated_config = toml::to_string(&config).with_context(|| "Could not serialize config")?;

    // Write the updated TOML content back to the configuration file
    fs::File::create(&config_path)
      .and_then(|mut file| file.write_all(updated_config.as_bytes()))
      .with_context(|| format!("Could not write to config file at {:?}", config_path))?;

    Ok(())
  }
}
