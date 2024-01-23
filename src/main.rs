#![allow(unused_imports)]
#![allow(dead_code)]
#![allow(unused_variables)]
// #![allow(unreachable_code)]
// #![allow(non_snake_case)]
// #![allow(clippy::clone_on_copy)]

use std::{
  fs,
  io::Write,
  option,
  path::{Path, PathBuf},
};

use anyhow::Context;
use chrono::Local;
use error::MyError;
use log::{debug, info, trace};
use serde::{Deserialize, Serialize};

use crate::cli::Cli;

mod cli;
mod error;
#[cfg(test)] mod tests;
mod utils;

const DATE_FORMAT: &str = "%Y-%m-%d";
// const SOURCE_PATH_STR: &str = "/home/thor/obsidian/writing/blog";
const TARGET_PATH_STR: &str = "/home/thor/projects/blog/content/posts";
const SOURCE_IMG_PATH_STR: &str = "/home/thor/obsidian/media/image";
const TARGET_IMG_PATH_STR: &str = "/home/thor/projects/blog/static/photos";
const CONFIG_FILE_PATH: &str = "/home/thor/.files/scripts/tkblogpublish/config.toml";

fn main() -> Result<(), MyError> {
  let cli = &utils::setup()?;
  match cli {
    Cli::One(c) => update_file(&c.source_path, &c.target_path)?,
    Cli::All(c) => update_files(&c.config)?,
    Cli::ConfigAdd(c) => add_file(&c.config, &c.source_path, &c.target_path)?,
    Cli::ConfigRemove(c) => remove_file(&c.config, &c.source_path)?,
  }

  Ok(())
}

fn remove_file(
  config: &Option<PathBuf>,
  source_path: &Path,
  // target_path: &Option<PathBuf>,
) -> Result<(), MyError> {
  let config_path: PathBuf = config.clone().unwrap_or_else(|| PathBuf::from(CONFIG_FILE_PATH));
  let config_content = fs::read_to_string(config_path.clone()).expect("could not read config file");
  let mut config: Config = toml::from_str(&config_content).expect("Could not parse config file");

  // Look for any file matching end of source_path name and remove it
  let file_name = source_path
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

  info!("Removing file from config: {:?}", source_path);

  // Serialize the updated configuration back to TOML
  let updated_config = toml::to_string(&config).with_context(|| "Could not serialize config")?;

  // Write the updated TOML content back to the configuration file
  fs::File::create(&config_path)
    .and_then(|mut file| file.write_all(updated_config.as_bytes()))
    .with_context(|| format!("Could not write to config file at {:?}", config_path))?;

  Ok(())
}

/// add a file to config file
fn add_file(
  config: &Option<PathBuf>,
  source_path: &Path,
  target_path: &Option<PathBuf>,
) -> Result<(), MyError> {
  let config_path: PathBuf = config.clone().unwrap_or_else(|| PathBuf::from(CONFIG_FILE_PATH));
  let config_content = fs::read_to_string(config_path.clone()).expect("could not read config file");
  let mut config: Config = toml::from_str(&config_content).expect("Could not parse config file");

  // assert that source path exists
  if !source_path.exists() {
    return Err(anyhow::anyhow!("source path does not exist").into());
  }

  // Replace relative paths with absolute paths
  let absolute_source_path = fs::canonicalize(source_path)
    .with_context(|| format!("Could not canonicalize source path {:?}", source_path))?;
  let absolute_target_path = match target_path {
    Some(path) => Some(
      fs::canonicalize(path)
        .with_context(|| format!("Could not canonicalize target path {:?}", path))?,
    ),
    None => None,
  };

  // Check that source path is not already in config
  if config.files.iter().any(|fp| fp.source == absolute_source_path) {
    return Err(anyhow::anyhow!("Source path already exists in config").into());
  }

  info!("Adding file to config: {:?} {:?}", absolute_source_path, absolute_target_path);

  // Create a new file pair
  let new_file_pair = FilePair { source: absolute_source_path, target: absolute_target_path };

  // Add the new file pair to the configuration
  config.files.push(new_file_pair);

  // Serialize the updated configuration back to TOML
  let updated_config = toml::to_string(&config).expect("Could not serialize config");

  // Write the updated TOML content back to the configuration file
  let mut file = fs::File::create(config_path).expect("Could not write to config file");
  file.write_all(updated_config.as_bytes()).expect("Could not write config file");

  Ok(())
}

fn update_files(config: &Option<PathBuf>) -> Result<(), MyError> {
  // parse config file
  let config_path: PathBuf = config.clone().unwrap_or_else(|| PathBuf::from(CONFIG_FILE_PATH));
  let config_content = fs::read_to_string(config_path).expect("could not read config file");
  let config: Config = toml::from_str(&config_content).expect("Could not parse config file");
  info!("updating files: {config:?}");

  // Iterate over file pairs and update each one
  for file_pair in config.files {
    update_file(&file_pair.source, &file_pair.target)?;
  }

  Ok(())
}

#[derive(Deserialize, Serialize, Debug)]
struct Config {
  files: Vec<FilePair>,
}

#[derive(Deserialize, Debug, Serialize)]
struct FilePair {
  source: PathBuf,
  target: Option<PathBuf>,
}

fn update_file(source_path: &Path, target_path: &Option<PathBuf>) -> Result<(), MyError> {
  debug!("source_filename: {:?}", &source_path);
  let content = fs::read_to_string(source_path).expect("could not read file");
  let original_date =
    content.lines().find(|line| line.starts_with("date: ")).unwrap().replace("date: ", "");

  let mut assume_blog = true;
  let target_path = {
    if let Some(path) = target_path {
      assume_blog = false;
      path.clone()
    } else {
      let source_filename = source_path
        .file_name()
        .expect("Invalid source file name")
        .to_str()
        .expect("Non-UTF8 source file name");

      let target = format!("{TARGET_PATH_STR}/{original_date}-{source_filename}");
      PathBuf::from(target)
    }
  };
  debug!("target: {target_path:?}");

  // update original file with new `last-update` field
  {
    let mut file = fs::File::create(source_path)?;
    file.write_all(content.as_bytes())?;
    info!("updated source file created on date: {original_date}");
  }

  // look for instances of:
  // $$\begin{align} ... \end{align}$$
  // and replace with $$line$$ for each line
  // let re = regex::Regex::new(r"\$\$\\begin\{align\}(.*\n?)+\\end\{align\}\$\$").unwrap();
  // let content = re
  //   .replace_all(&content, |caps: &regex::Captures| {
  //     let lines = caps[1].lines();
  //     let mut new_content = String::new();
  //     for line in lines {
  //       new_content.push_str(&format!("$$ {} $$\n", line));
  //     }
  //     debug!("new_content: {:?}", new_content);
  //     new_content
  //   })
  //   .to_string();

  if assume_blog {
    update_images(&original_date, &content, &target_path)?;
  }
  Ok(())
}

fn update_images(original_date: &str, content: &str, target_path: &Path) -> Result<(), MyError> {
  // update images in the target image directory:
  // - if target image directory does not exist, create it.
  // - create a list of image filenames in the source content.
  // - for each image, copy the image from the source_img_dir to the target image dir.

  // update image links in target (but not source)
  let first_word = target_path
    .file_name()
    .expect("Invalid target file name")
    .to_str()
    .expect("Non-UTF8 target file name")
    .split_whitespace()
    .next()
    .expect("No first word in target file name")
    .split('-')
    .last()
    .expect("No first word after dash in target file name");

  let target_image_dir = PathBuf::from(format!(
    "{TARGET_IMG_PATH_STR}/{}-{}",
    original_date, // just use date, since it won't ever have weird edge cases or spaces
    first_word
  ));
  let target_image_dir_name = target_image_dir.file_name().unwrap().to_str().unwrap();

  // update image links to match hugo syntax
  // match against syntax ![[image.png]]
  let mut image_filenames = Vec::new();
  let re = regex::Regex::new(r"\!\[\[([^\]]+)\]\]").unwrap();
  let content = re
    .replace_all(content, |caps: &regex::Captures| {
      let image_filename = &caps[1];
      image_filenames.push(image_filename.to_string());
      debug!("image_filename: {:?}", image_filename);

      // Replace with new link format
      let new_link = format!("![](/photos/{target_image_dir_name}/{image_filename})");
      debug!("link: {:?}", new_link);
      new_link
    })
    .to_string();

  // replace file at target with source content. open scope to auto flush.
  {
    let mut file = fs::File::create(target_path)?;
    file.write_all(content.as_bytes())?;
    info!("updated target file: {target_path:?}");
  }

  // target image directory may not yet exist
  if !target_image_dir.exists() {
    info!("dir does not exist, creating: {:?}", target_image_dir);
    fs::create_dir_all(target_image_dir.clone())?;
  }

  // copy images over
  let source_image_dir = PathBuf::from(SOURCE_IMG_PATH_STR);
  for image in image_filenames {
    let source_path = source_image_dir.join(image.clone());
    let target_path = target_image_dir.join(image.clone());
    fs::copy(&source_path, &target_path)?;
    debug!("copied image: {image:?} from {source_path:?} to {target_path:?}");
  }

  Ok(())
}
