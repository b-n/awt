use clap::{Parser, ValueEnum};
use std::path::PathBuf;

pub fn log_level(level: LogLevel) -> log::Level {
    match level {
        LogLevel::Error => log::Level::Error,
        LogLevel::Warn => log::Level::Warn,
        LogLevel::Info => log::Level::Info,
        LogLevel::Debug => log::Level::Debug,
        LogLevel::Trace => log::Level::Trace,
    }
}

#[derive(Copy, Clone, Debug, ValueEnum)]
pub enum LogLevel {
    Error,
    Warn,
    Info,
    Debug,
    Trace,
}

impl core::fmt::Display for LogLevel {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(f, "{}", format!("{self:?}").to_lowercase())
    }
}

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
pub struct Args {
    /// Path to the config.toml to use
    #[arg(required = true)]
    pub config_path: Option<PathBuf>,
    #[arg(long, default_value_t = LogLevel::Error)]
    pub log_level: LogLevel,
}
