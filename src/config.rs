use anyhow::{Context, Result};
use serde::Deserialize;
use std::fs;
use std::path::PathBuf;

#[derive(Debug, Deserialize, Clone)]
pub struct Config {
    pub twitter: TwitterConfig,
}

#[derive(Debug, Deserialize, Clone)]
pub struct TwitterConfig {
    pub api_key: String,
    pub api_secret: String,
    pub access_token: String,
    pub access_token_secret: String,
}

impl Config {
    pub fn load() -> Result<Self> {
        let config_path = Self::config_path()?;
        
        if !config_path.exists() {
            anyhow::bail!(
                "Config file not found at: {}\n\n\
                Please create this file with your X API credentials:\n\n\
                [twitter]\n\
                api_key = \"your_api_key\"\n\
                api_secret = \"your_api_secret\"\n\
                access_token = \"your_access_token\"\n\
                access_token_secret = \"your_access_token_secret\"\n\n\
                Get your credentials at: https://developer.x.com/en/portal/dashboard",
                config_path.display()
            );
        }

        let config_str = fs::read_to_string(&config_path)
            .context("Failed to read config file")?;
        
        let config: Config = toml::from_str(&config_str)
            .context("Failed to parse config file")?;

        //set permissions to 600 (user read/write only)
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            let metadata = fs::metadata(&config_path)?;
            let mut permissions = metadata.permissions();
            permissions.set_mode(0o600);
            fs::set_permissions(&config_path, permissions)?;
        }

        Ok(config)
    }

    fn config_path() -> Result<PathBuf> {
        let home = std::env::var("HOME")
            .context("HOME environment variable not set")?;
        let config_dir = PathBuf::from(home).join(".config").join("xpost");
        
        fs::create_dir_all(&config_dir)?;
        
        Ok(config_dir.join("config.toml"))
    }
}
