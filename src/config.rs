use anyhow::Context;
use serde::{Deserialize, Serialize};
use std::fs;
use std::io::Write;
use std::path::PathBuf;

#[derive(Debug, Serialize, Deserialize, Default)]
pub struct Config {
    pub url: Option<String>,
    pub token: Option<String>,
    pub token_name: Option<String>,
    pub username: Option<String>,
}

impl Config {
    pub fn config_path() -> Option<PathBuf> {
        dirs::config_dir().map(|dir| dir.join("edgectl").join("config"))
    }

    pub fn load() -> Self {
        let path = match Self::config_path() {
            Some(p) => p,
            None => return Self::default(),
        };

        if !path.exists() {
            return Self::default();
        }

        let contents = match fs::read_to_string(&path) {
            Ok(c) => c,
            Err(_) => return Self::default(),
        };

        toml::from_str(&contents).unwrap_or_default()
    }

    pub fn save(&self) -> anyhow::Result<()> {
        let path = Self::config_path().context("Failed to determine config directory")?;

        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent)?;
        }

        let contents = toml::to_string_pretty(self).context("Failed to serialize config")?;

        let mut file = fs::File::create(&path).context("Failed to open config")?;

        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            let mut perms = file.metadata()?.permissions();
            perms.set_mode(0o600);
            fs::set_permissions(&path, perms)?;
        }

        file.write_all(contents.as_bytes())?;
        Ok(())
    }

    pub fn delete() -> Result<(), std::io::Error> {
        let path = match Self::config_path() {
            Some(p) => p,
            None => return Ok(()),
        };

        if path.exists() {
            fs::remove_file(path)?;
        }

        Ok(())
    }
}
