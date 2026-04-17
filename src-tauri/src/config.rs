use crate::direction::Direction;
use directories::ProjectDirs;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::path::PathBuf;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Modifier(pub String);

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Shortcut {
    pub modifiers: Vec<Modifier>,
    pub key: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub label: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GestroConfig {
    pub threshold: f64,
    pub bindings: HashMap<Direction, Shortcut>,
    pub launch_at_login: bool,
}

impl Default for GestroConfig {
    fn default() -> Self {
        Self {
            threshold: 50.0,
            bindings: HashMap::new(),
            launch_at_login: false,
        }
    }
}

impl GestroConfig {
    /// Get the config file path using the `directories` crate.
    fn config_path() -> Option<PathBuf> {
        ProjectDirs::from("com", "gestro", "gestro")
            .map(|dirs| dirs.config_dir().join("config.json"))
    }

    /// Load config from disk, falling back to defaults if missing or invalid.
    pub fn load() -> Self {
        let Some(path) = Self::config_path() else {
            log::warn!("Could not determine config directory, using defaults");
            return Self::default();
        };

        match fs::read_to_string(&path) {
            Ok(contents) => match serde_json::from_str(&contents) {
                Ok(config) => {
                    log::info!("Loaded config from {}", path.display());
                    config
                }
                Err(e) => {
                    log::warn!("Failed to parse config: {e}, using defaults");
                    Self::default()
                }
            },
            Err(_) => {
                log::info!("No config file found, using defaults");
                Self::default()
            }
        }
    }

    /// Save config to disk.
    pub fn save(&self) -> Result<(), String> {
        let path = Self::config_path().ok_or("Could not determine config directory")?;

        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent)
                .map_err(|e| format!("Failed to create config directory: {e}"))?;
        }

        let json = serde_json::to_string_pretty(self)
            .map_err(|e| format!("Failed to serialize config: {e}"))?;

        fs::write(&path, json).map_err(|e| format!("Failed to write config: {e}"))?;

        log::info!("Saved config to {}", path.display());
        Ok(())
    }
}
