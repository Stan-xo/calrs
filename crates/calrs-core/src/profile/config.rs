//! Profile configuration: persisted list of profiles and active selection.

use crate::APP_NAME;
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::{Path, PathBuf};

/// Configuration file storing profile metadata.
///
/// Persisted as TOML in the OS data directory.
#[derive(Debug, Serialize, Deserialize)]
pub struct Config {
    /// Name of the currently active profile.
    pub active_profile: String,
    /// List of all known profiles.
    pub profiles: Vec<ProfileEntry>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ProfileEntry {
    pub name: String,
    pub timezone: String,
}

impl Config {
    /// Creates an empty configuration with no profiles.
    pub fn new() -> Self {
        Config {
            active_profile: String::new(),
            profiles: Vec::new(),
        }
    }
}

impl Default for Config {
    fn default() -> Self {
        Self::new()
    }
}

/// Returns the path to the config file in the OS data directory.
pub fn config_path() -> PathBuf {
    dirs::data_dir()
        .expect("Could not find data directory")
        .join(APP_NAME)
        .join("config.toml")
}

/// Loads the configuration from disk.
///
/// Returns a default empty config if the file doesn't exist yet.
pub fn load(path: Option<&Path>) -> Result<Config, String> {
    let default_path = config_path();
    let path = path.unwrap_or(&default_path);
    if !path.exists() {
        return Ok(Config::new());
    }
    let content = fs::read_to_string(path).map_err(|e| format!("Failed to read config: {}", e))?;
    toml::from_str(&content).map_err(|e| format!("Failed to parse config: {}", e))
}

/// Saves the configuration to disk.
pub fn save(config: &Config, path: Option<&Path>) -> Result<(), String> {
    let default_path = config_path();
    let path = path.unwrap_or(&default_path);
    fs::create_dir_all(path.parent().unwrap())
        .map_err(|e| format!("Failed to create directory: {}", e))?;
    let content =
        toml::to_string_pretty(config).map_err(|e| format!("Failed to serialize config: {}", e))?;
    fs::write(path, content).map_err(|e| format!("Failed to write config: {}", e))?;
    Ok(())
}
