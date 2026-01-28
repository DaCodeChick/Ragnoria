use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Config {
    pub server: ServerConfig,
    pub game_path: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServerConfig {
    pub ip: String,
    pub port: u16,
}

impl Default for Config {
    fn default() -> Self {
        // Try to find the game path automatically
        let default_game_path = vec![
            "/run/media/admin/FE6407F46407AE89/Gravity/Ragnarok Online 2 - Jawaii/SHIPPING/Rag2.exe",
            "C:\\Program Files (x86)\\Gravity\\Ragnarok Online 2\\SHIPPING\\Rag2.exe",
            "C:\\Program Files\\Gravity\\Ragnarok Online 2\\SHIPPING\\Rag2.exe",
        ]
        .iter()
        .find(|path| std::path::Path::new(path).exists())
        .map(|s| s.to_string())
        .unwrap_or_default();

        Self {
            server: ServerConfig {
                ip: String::from("127.0.0.1"),
                port: 7101,
            },
            game_path: default_game_path,
        }
    }
}

impl Config {
    /// Get the config file path
    pub fn config_path() -> Result<PathBuf> {
        let config_dir =
            dirs::config_dir().ok_or_else(|| anyhow::anyhow!("Could not find config directory"))?;

        let app_config_dir = config_dir.join("ragnoria");

        // Create directory if it doesn't exist
        if !app_config_dir.exists() {
            fs::create_dir_all(&app_config_dir)?;
        }

        Ok(app_config_dir.join("launcher.toml"))
    }

    /// Load config from file, or return default if file doesn't exist
    pub fn load() -> Result<Self> {
        let path = Self::config_path()?;

        if !path.exists() {
            return Ok(Self::default());
        }

        let contents = fs::read_to_string(path)?;
        let config: Config = toml::from_str(&contents)?;

        Ok(config)
    }

    /// Save config to file
    pub fn save(&self) -> Result<()> {
        let path = Self::config_path()?;
        let contents = toml::to_string_pretty(self)?;

        fs::write(path, contents)?;

        Ok(())
    }
}
