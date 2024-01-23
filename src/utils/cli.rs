use std::path::PathBuf;

use chrono::Local;
use clap::Parser;
use log::debug;

/// My Blog Tool
#[derive(Parser, Debug)]
#[clap(version = "1.0", author = "Thor Kamphefner")]
#[command(name = "tkblogpub")]
#[command(bin_name = "tkblogpub")]
#[command(about = "a tool to publish my blog posts from my Obsidian vault to my website")]
pub struct Cli {
  /// Sets the source file to use
  #[arg(index = 1)]
  pub source: PathBuf,
  /// Sets a custom config file
  #[arg(short, long, value_name = "FILE")]
  config:     Option<PathBuf>,
}
