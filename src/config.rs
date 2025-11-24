use anyhow::{anyhow, Context};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::io::Write;
use std::path::PathBuf;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ContextConfig {
    pub url: String,
    pub token: String,
    pub token_name: String,
    pub username: String,
}

#[derive(Debug, Serialize, Deserialize, Default)]
pub struct Config {
    pub context: Option<String>,
    #[serde(default)]
    pub contexts: HashMap<String, ContextConfig>,
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

    pub fn get_current_context(&self) -> Option<&ContextConfig> {
        let context_name = self.context.as_ref()?;
        self.contexts.get(context_name)
    }

    pub fn set_current_context(&mut self, name: String) -> anyhow::Result<()> {
        if !self.contexts.contains_key(&name) {
            return Err(anyhow!("Context '{}' does not exist", name));
        }
        self.context = Some(name);
        Ok(())
    }

    pub fn add_context(&mut self, name: String, context: ContextConfig) {
        self.contexts.insert(name.clone(), context);
        self.context = Some(name);
    }

    pub fn delete_context(&mut self, name: &str) -> anyhow::Result<()> {
        if !self.contexts.contains_key(name) {
            return Err(anyhow!("Context '{}' does not exist", name));
        }

        self.contexts.remove(name);

        if self.context.as_deref() == Some(name) {
            self.context = self.contexts.keys().next().cloned();
        }

        Ok(())
    }

    pub fn list_contexts(&self) -> Vec<(&String, &ContextConfig)> {
        let mut contexts: Vec<_> = self.contexts.iter().collect();
        contexts.sort_by_key(|(name, _)| *name);
        contexts
    }
}
