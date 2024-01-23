use std::path::PathBuf;

use clap::{Args, Parser};

/// My Blog Tool
#[derive(Parser, Debug)]
#[clap(version = "1.0", author = "Thor Kamphefner")]
#[command(name = "tkblogpub")]
#[command(bin_name = "tkblogpub")]
#[command(about = "a tool to publish my blog posts from my Obsidian vault to my website")]
pub struct Cli_ {
  /// Sets the source file to use
  #[arg(index = 1)]
  pub source_path: PathBuf,
  /// Sets the source file to use
  #[arg(index = 2)]
  pub target_path: Option<PathBuf>,
  /// Sets a custom config file
  #[arg(short, long, value_name = "FILE")]
  pub config:      Option<PathBuf>,
}

#[derive(Parser, Debug)]
#[clap(version = "1.0", author = "Thor Kamphefner")]
#[command(name = "tkblogpublish")]
#[command(bin_name = "tkblogpub")]
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
}
