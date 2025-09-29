use std::{fmt::Debug, io::BufRead};

/// Load a value from a .env
///
/// Scans through the file looking for a line `key=value`
pub fn load_dotenv_value(path: &str, key: &str) -> Result<String, DotEnvError> {
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
    Err(DotEnvError::KeyNotFound)
}

pub enum DotEnvError {
    DotEnvNotFound(std::io::Error),
    SyntaxError(usize),
    KeyNotFound,
    Other(std::io::Error),
}

impl Debug for DotEnvError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::DotEnvNotFound(e) => f.debug_struct("DotEnvNotFound").field("error", &e).finish(),
            Self::SyntaxError(l) => f
                .debug_struct("SyntaxError")
                .field("line_number", &l)
                .finish(),
            Self::KeyNotFound => f.debug_struct("KeyNotFound").finish(),
            Self::Other(e) => f.debug_struct("Other").field("error", &e).finish(),
        }
    }
}
