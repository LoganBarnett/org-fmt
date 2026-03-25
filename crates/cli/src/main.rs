//! org-fmt-cli — command-line org-mode document formatter

mod config;
mod logging;

use clap::Parser;
use config::{CliRaw, Config, ConfigError};
use logging::init_logging;
use org_fmt_lib::format::format_org;
use std::io::Read;
use std::path::PathBuf;
use thiserror::Error;
use tracing::debug;

#[derive(Debug, Error)]
enum ApplicationError {
  #[error("Failed to load configuration during startup: {0}")]
  ConfigurationLoad(#[from] ConfigError),

  #[error("Failed to read {path:?}: {source}")]
  FileRead {
    path: PathBuf,
    #[source]
    source: std::io::Error,
  },

  #[error("Failed to write {path:?}: {source}")]
  FileWrite {
    path: PathBuf,
    #[source]
    source: std::io::Error,
  },

  #[error("Failed to read stdin: {0}")]
  StdinRead(#[source] std::io::Error),
}

fn main() -> Result<(), ApplicationError> {
  let cli = CliRaw::parse();

  let config = Config::from_cli_and_file(cli).map_err(|e| {
    eprintln!("Configuration error: {}", e);
    ApplicationError::ConfigurationLoad(e)
  })?;

  init_logging(config.log_level, config.log_format);

  run(config)
}

fn run(config: Config) -> Result<(), ApplicationError> {
  if config.files.is_empty() {
    debug!("Reading from stdin");
    let mut input = String::new();
    std::io::stdin()
      .read_to_string(&mut input)
      .map_err(ApplicationError::StdinRead)?;
    let output = format_org(&input);
    print!("{output}");
  } else {
    for path in &config.files {
      debug!(?path, "Formatting file");
      let input =
        std::fs::read_to_string(path).map_err(|source| ApplicationError::FileRead {
          path: path.clone(),
          source,
        })?;
      let output = format_org(&input);
      if config.in_place {
        std::fs::write(path, output).map_err(|source| ApplicationError::FileWrite {
          path: path.clone(),
          source,
        })?;
      } else {
        print!("{output}");
      }
    }
  }
  Ok(())
}
