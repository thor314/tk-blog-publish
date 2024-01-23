use std::path::PathBuf;

use clap::Parser;

/// My Blog Tool
#[derive(Parser, Debug)]
#[clap(version = "1.0", author = "Thor Kamphefner")]
#[command(name = "tkblogpub")]
#[command(bin_name = "tkblogpub")]
#[command(about = "a tool to publish my blog posts from my Obsidian vault to my website")]
pub struct Cli {
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
