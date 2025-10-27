use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};

#[derive(Debug, Serialize, Deserialize, Default)]
pub struct Config {
    pub servers: HashMap<String, PathBuf>,
}

impl Config {
    pub fn load_or_default() -> Result<Self> {
        let config_path = Self::config_path();
        if config_path.exists() {
            let content = fs::read_to_string(&config_path)
                .with_context(|| format!("Failed to read config file: {:?}", config_path))?;
            let config: Config = toml::from_str(&content)
                .with_context(|| format!("Failed to parse config file: {:?}", config_path))?;
            Ok(config)
        } else {
            Ok(Config::default())
        }
    }

    pub fn save(&self) -> Result<()> {
        let config_path = Self::config_path();
        let content = toml::to_string(self)
            .context("Failed to serialize config")?;
        fs::create_dir_all(config_path.parent().unwrap())
            .with_context(|| format!("Failed to create config directory: {:?}", config_path.parent()))?;
        fs::write(&config_path, content)
            .with_context(|| format!("Failed to write config file: {:?}", config_path))?;
        Ok(())
    }

    pub fn add_server(&mut self, name: String, path: PathBuf) -> Result<()> {
        if self.servers.contains_key(&name) {
            anyhow::bail!("Server '{}' already exists", name);
        }
        self.servers.insert(name, path);
        Ok(())
    }

    pub fn get_server_path(&self, name: &str) -> Result<&PathBuf> {
        self.servers.get(name)
            .with_context(|| format!("Server '{}' not found", name))
    }

    pub fn list_servers(&self) -> Vec<String> {
        self.servers.keys().cloned().collect()
    }

    fn config_path() -> PathBuf {
        dirs::config_dir()
            .unwrap_or_else(|| PathBuf::from("."))
            .join("cs2-server-cli")
            .join("config.toml")
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ServerConfig {
    pub hostname: String,
    pub rcon_password: String,
    pub sv_password: String,
    pub maxplayers: u32,
    pub map: String,
    pub game_mode: String,
    pub game_type: String,
}

impl Default for ServerConfig {
    fn default() -> Self {
        Self {
            hostname: "CS2 Server".to_string(),
            rcon_password: "changeme".to_string(),
            sv_password: "".to_string(),
            maxplayers: 10,
            map: "de_dust2".to_string(),
            game_mode: "0".to_string(),
            game_type: "0".to_string(),
        }
    }
}

impl ServerConfig {
    pub fn load(path: &Path) -> Result<Self> {
        let content = fs::read_to_string(path)
            .with_context(|| format!("Failed to read server config: {:?}", path))?;
        Self::parse_from_cfg(&content)
    }

    pub fn save(&self, path: &Path) -> Result<()> {
        let content = self.to_cfg_string();
        fs::write(path, content)
            .with_context(|| format!("Failed to write server config: {:?}", path))?;
        Ok(())
    }

    pub fn set(&mut self, key: &str, value: &str) -> Result<()> {
        match key {
            "hostname" => self.hostname = value.to_string(),
            "rcon_password" => self.rcon_password = value.to_string(),
            "sv_password" => self.sv_password = value.to_string(),
            "maxplayers" => self.maxplayers = value.parse().context("Invalid maxplayers value")?,
            "map" => self.map = value.to_string(),
            "game_mode" => self.game_mode = value.to_string(),
            "game_type" => self.game_type = value.to_string(),
            _ => anyhow::bail!("Unknown configuration key: {}", key),
        }
        Ok(())
    }

    fn to_cfg_string(&self) -> String {
        format!(
            "// CS2 Server Configuration\n\
             hostname \"{}\"\n\
             rcon_password \"{}\"\n\
             sv_password \"{}\"\n\
             maxplayers {}\n\
             map {}\n\
             game_mode {}\n\
             game_type {}\n",
            self.hostname, self.rcon_password, self.sv_password, self.maxplayers,
            self.map, self.game_mode, self.game_type
        )
    }

    fn parse_from_cfg(content: &str) -> Result<Self> {
        let mut config = Self::default();

        for line in content.lines() {
            let line = line.trim();
            if line.starts_with("//") || line.is_empty() {
                continue;
            }

            if let Some((key, value)) = Self::parse_cfg_line(line) {
                config.set(&key, &value)?;
            }
        }

        Ok(config)
    }

    fn parse_cfg_line(line: &str) -> Option<(String, String)> {
        let line = line.trim();
        if line.contains('"') {
            // Handle quoted values
            let parts: Vec<&str> = line.split('"').collect();
            if parts.len() >= 3 {
                let key = parts[0].trim().to_string();
                let value = parts[1].to_string();
                Some((key, value))
            } else {
                None
            }
        } else {
            // Handle unquoted values
            let parts: Vec<&str> = line.split_whitespace().collect();
            if parts.len() >= 2 {
                let key = parts[0].to_string();
                let value = parts[1].to_string();
                Some((key, value))
            } else {
                None
            }
        }
    }
}