use std::{fs, path::PathBuf};

use log::{debug, info, trace, warn};
use regex::Regex;
use serde::{Deserialize, Serialize};

use super::*;
use crate::{error::MyError, utils::get_original_date};

/// My toml config file
#[derive(Deserialize, Serialize, Debug)]
pub(crate) struct Config {
  pub(crate) files: Vec<FilePair>,
}

/// mapping from source file to destination file
#[derive(Deserialize, Debug, Serialize, Default)]
pub(crate) struct FilePair {
  pub(crate) source: PathBuf,
  pub(crate) target: PathBuf,
}

impl FilePair {
  /// update a single file
  pub fn update_file(&self) -> Result<(), MyError> {
    debug!("source_filename: {:?}", &self.source);
    let content = fs::read_to_string(self.source.clone()).expect("could not read file");
    let original_date = get_original_date(&self.source, &content)?;

    debug!("target: {:?}", self.target);

    // hugo doesn't recognize double backslash in align blocks, so
    // replace trailing double-backslash with triple-backslash in align blocks
    let backslash_re = Regex::new(r"([^\\])\\{2}\n").unwrap();
    let content = backslash_re
      .replace_all(&content, |caps: &regex::Captures| {
        let c1 = &caps[1];
        debug!("string before: {:?}", &caps[0]);
        let s = format!("{c1} \\\\\\\n");
        debug!("string after: {:?}", s);
        s
      })
      .to_string();

    // update original file with new `last-update` field
    {
      let mut file = fs::File::create(self.source.clone())?;
      file.write_all(content.as_bytes())?;
      trace!("updated source file created on date: {original_date}");
    }

    // if the target is a blog post, update the images
    let update_images = self.target.to_str().unwrap().contains("/blog/");
    if update_images {
      self.update_images(&original_date, &content)?;
    } else {
      // replace file at target with source content. open scope to auto flush.
      let mut file = fs::File::create(self.target.clone())?;
      file.write_all(content.as_bytes())?;
      info!("updated target file: {:?}", self.target);
    }
    Ok(())
  }

  /// copy the images over to the blog
  /// and update the image links in a file to match blog format
  pub(crate) fn update_images(&self, original_date: &str, content: &str) -> Result<(), MyError> {
    // update images in the target image directory:
    // - if target image directory does not exist, create it.
    // - create a list of image filenames in the source content.
    // - for each image, copy the image from the source_img_dir to the target image dir.

    // Name the images
    let first_word = self
      .target
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

    // Find and move the images
    let target_image_dir_name = target_image_dir.file_name().unwrap().to_str().unwrap();

    // update image links to match hugo syntax
    // match against syntax ![[image.png]]
    let mut image_filenames = Vec::new();
    let re = Regex::new(r"\!\[\[([^\]]+)\]\]").unwrap();
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
      let mut file = fs::File::create(self.target.clone())?;
      file.write_all(content.as_bytes())?;
      info!("updated target file: {:?}", self.target);
    }

    // target image directory may not yet exist
    if !target_image_dir.exists() {
      warn!("dir does not exist, creating: {:?}", target_image_dir);
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
}
