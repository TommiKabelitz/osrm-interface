use std::{fmt::Debug, io::BufRead};

use thiserror::Error;

/// Load a value from a .env
///
/// Scans through the file looking for a line `key=value`
pub fn load_dotenv_value<'a>(path: &str, key: &'a str) -> Result<String, DotEnvError<'a>> {
    let file = std::fs::File::open(path).map_err(DotEnvError::DotEnvNotFound)?;
    let reader = std::io::BufReader::new(file);

    for (i, line) in reader.lines().enumerate() {
        let line = line.map_err(DotEnvError::Other)?; // unwrap Result
        let line = line.trim();
        if line.starts_with(key) {
            return line
                .split_once("=")
                .ok_or(DotEnvError::SyntaxError(i))
                .map(|(_, v)| v.to_string());
        }
    }
    Err(DotEnvError::KeyNotFound(key))
}

#[derive(Error, Debug)]
pub enum DotEnvError<'a> {
    #[error(".env not found: {0}")]
    DotEnvNotFound(std::io::Error),
    #[error("Syntax error on line {0}")]
    SyntaxError(usize),
    #[error("Key '{0}' not found in .env")]
    KeyNotFound(&'a str),
    #[error("Other IO error: {0}")]
    Other(std::io::Error),
}
