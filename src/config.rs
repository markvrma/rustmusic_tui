use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use std::fs;
use std::io::{self, Write};
use std::path::PathBuf;

#[derive(Serialize, Deserialize)]
pub struct Config {
    pub music_directory: PathBuf,
}

impl Default for Config {
    fn default() -> Self {
        Config {
            music_directory: PathBuf::new(),
        }
    }
}

impl Config {
    fn get_config_path() -> Result<PathBuf> {
        let config_dir =
            dirs::config_dir().ok_or_else(|| anyhow::anyhow!("Could not find config directory"))?;
        Ok(config_dir.join("rustmusic_tui").join("config.toml"))
    }

    pub fn load() -> Result<Self> {
        let config_path = Self::get_config_path()?;

        if !config_path.exists() {
            return Self::setup();
        }

        let content = fs::read_to_string(&config_path).context("Failed to read config file")?;

        let config: Config = toml::from_str(&content).context("Failed to parse config file")?;

        Ok(config)
    }

    fn setup() -> Result<Self> {
        println!("Welcome to RustMusic TUI!");
        println!("Please enter the path to your music directory:");
        print!("> ");
        io::stdout().flush()?;

        let mut input = String::new();
        io::stdin().read_line(&mut input)?;
        let path_str = input.trim();
        let path = PathBuf::from(path_str);

        if !path.exists() || !path.is_dir() {
            return Err(anyhow::anyhow!(
                "The path '{}' does not exist or is not a directory.",
                path_str
            ));
        }

        let config = Config {
            music_directory: fs::canonicalize(&path)?,
        };

        config.save()?;
        Ok(config)
    }

    fn save(&self) -> Result<()> {
        let config_path = Self::get_config_path()?;
        if let Some(parent) = config_path.parent() {
            fs::create_dir_all(parent)?;
        }

        let content = toml::to_string_pretty(self)?;
        fs::write(config_path, content)?;
        Ok(())
    }
}
