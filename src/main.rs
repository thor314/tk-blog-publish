#![allow(unused_imports)]
// #![allow(unused_variables)]
#![allow(dead_code)]
// #![allow(unreachable_code)]
// #![allow(non_snake_case)]
// #![allow(clippy::clone_on_copy)]

use std::{
  fs,
  io::Write,
  path::{Path, PathBuf},
};

use chrono::Local;
use error::MyError;
use log::{debug, info, trace};

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

fn main() -> Result<(), MyError> {
  let Cli { source_path, target_path, config: _ } = &utils::setup()?;
  let mut content = fs::read_to_string(source_path).expect("could not read file");
  let original_date =
    content.lines().find(|line| line.starts_with("date: ")).unwrap().replace("date: ", "");
  debug!("source_filename: {:?}", &source_path);

  let target_path = {
    if let Some(path) = target_path {
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

  // update `last-update` field in content
  let new_line = format!("last-update: {}", Local::now().format(DATE_FORMAT));
  content = content
    .lines()
    .map(|line| if line.starts_with("last-update:") { &new_line } else { line })
    .collect::<Vec<_>>()
    .join("\n");

  // update original file with new `last-update` field
  {
    let mut file = fs::File::create(source_path)?;
    file.write_all(content.as_bytes())?;
    info!("updated source file created on date: {original_date}");
  }

  // Now we update images in the target image directory:
  // - if target image directory does not exist, create it.
  // - create a list of image filenames in the source content.
  // - for each image, copy the image from the source_img_dir to the target image dir.

  // update image links in target (but not source)
  let target_image_dir = PathBuf::from(format!(
    "{TARGET_IMG_PATH_STR}/{}",
    original_date // just use date, since it won't ever have weird edge cases or spaces
  ));
  let target_image_dir_name = original_date.to_string();

  // update image links to match hugo syntax
  // match against syntax ![[image.png]]
  let re = regex::Regex::new(r"\!\[\[([^\]]+)\]\]").unwrap();

  let mut image_filenames = Vec::new();
  content = re
    .replace_all(&content, |caps: &regex::Captures| {
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
    let mut file = fs::File::create(target_path.clone())?;
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
