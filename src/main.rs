// #![allow(unused_imports)]
// #![allow(unused_variables)]
// #![allow(dead_code)]
// #![allow(unreachable_code)]
// #![allow(non_snake_case)]
// #![allow(clippy::clone_on_copy)]

use std::{
  fs,
  path::{Path, PathBuf},
};

use chrono::Local;
use error::MyError;
use log::{debug, info, trace};

mod error;
#[cfg(test)] mod tests;
mod utils;

fn main() -> Result<(), MyError> {
  let _cli = utils::setup()?;
  let source_content = fs::read_to_string(&_cli.source).expect("could not read file");
  debug!("source_filename: {:?}", _cli.source);
  let target = get_target(_cli.source);
  debug!("target: {target:?}");

  if target.exists() {
    // check if content differs
    if source_content == fs::read_to_string(&target)? {
      info!("source and target content are the same, no update needed");
      return Ok(());
    }
  }

  update_target(target, &source_content)?;

  Ok(())
}

/// replace file at target with source content.
/// if target does not exist, create it.
/// if image directory does not exist, create it.
fn update_target(target: PathBuf, source_content: &str) -> Result<(), MyError> {
  let _target_dir = target.parent().expect("invalid target path");
  let target_filename = target.file_name().expect("invalid target filename");
  let img_dir =
    format!("/home/thor/projects/blog/static/photos/{}", target_filename.to_str().unwrap());
  let _img_dir = Path::new(&img_dir);

  let _img_dir =
    format!("/home/thor/projects/blog/static/photos/{}", target_filename.to_str().unwrap());
  if !target.exists() {
    // create target file
    info!("target does not exist, creating new file");
    if let Some(parent) = target.parent() {
      info!("creating parent directory: {parent:?}");
      fs::create_dir_all(parent)?;
    }
    fs::File::create(target.clone())?;
  }

  update_images(target, source_content)?;
  todo!()
}

fn create_target(_target: PathBuf, _create: bool) -> Result<(), MyError> { Ok(()) }

/// - if an image is found in SOURCE (with syntax `![[image-name.png]]`:
/// - let image subdirectory name be same as TARGET
/// - make new subdirectory in `~/projects/blog/public/photos`
///     - if directory already exists, remove it first
/// - copy any images in the post (from dir: `~/obsidian/media/photos/` to the the blog image
///   directory
fn update_images(_target: PathBuf, _source_content: &str) -> Result<(), MyError> { todo!() }

/// given source filename, target filename="YYYY-MM-DD-$SOURCE"
pub fn get_target(source: PathBuf) -> PathBuf {
  let date = Local::now().format("%Y-%m-%d").to_string();
  let source_filename = &source
    .file_name()
    .expect("Invalid source file name")
    .to_str()
    .expect("Non-UTF8 source file name");

  let target_path = "/home/thor/projects/blog/content/posts";
  let target = format!("{target_path}/{date}-{source_filename}");
  trace!("target: {}", target);
  PathBuf::from(target)
}
