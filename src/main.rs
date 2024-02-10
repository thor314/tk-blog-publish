#![allow(unused_imports)]
#![allow(dead_code)]
#![allow(unused_variables)]

use std::{io::Write, option, path::Path};

use anyhow::Context;
use chrono::Local;
use error::MyError;
use regex::Regex;
use serde::{Deserialize, Serialize};

use crate::cli::Cli;

mod cli;
mod error;
#[cfg(test)] mod tests;
mod utils;

pub const DATE_FORMAT: &str = "%Y-%m-%d";
pub const DEFAULT_TARGET_PATH_STR: &str = "/home/thor/projects/blog/content/posts";
pub const DEFAULT_PRIVATE_TARGET_PATH_STR: &str = "/home/thor/projects/blog/content/private";
pub const SOURCE_IMG_PATH_STR: &str = "/home/thor/obsidian/media/image";
pub const TARGET_IMG_PATH_STR: &str = "/home/thor/projects/blog/static/photos";
pub const CONFIG_FILE_PATH: &str = "/home/thor/projects/tk-blog-publish/config.toml";

fn main() -> Result<(), MyError> {
  let cli = &utils::setup()?;
  match cli {
    Cli::Update(c) => c.update_files()?,
    Cli::Add(c) => c.add_file()?,
    Cli::Remove(c) => c.remove_file()?,
  }

  Ok(())
}

mod filepair;
