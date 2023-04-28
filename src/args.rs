use clap::Parser;
use std::path::PathBuf;

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
pub struct Args {
    /// Path to the config.toml to use
    #[arg(required = true)]
    pub config_path: Option<PathBuf>,
}
