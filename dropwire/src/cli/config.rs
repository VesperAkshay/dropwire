use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;

pub const DEFAULT_RELAY: &str = "ws://relay.dropwire.tyes.dev:9010";

#[derive(Serialize, Deserialize, Debug, Default)]
pub struct DropwireConfig {
    pub relay: Option<String>,
}

impl DropwireConfig {
    fn config_path() -> PathBuf {
        let mut path = dirs::config_dir().unwrap_or_else(|| PathBuf::from("."));
        path.push("dropwire");
        path.push("config.json");
        path
    }

    pub fn load() -> Self {
        let path = Self::config_path();
        if path.exists() {
            if let Ok(content) = fs::read_to_string(path) {
                if let Ok(config) = serde_json::from_str(&content) {
                    return config;
                }
            }
        }
        Self::default()
    }

    pub fn save(&self) -> Result<(), std::io::Error> {
        let path = Self::config_path();
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent)?;
        }
        let content = serde_json::to_string_pretty(self)?;
        fs::write(path, content)?;
        Ok(())
    }

    pub fn get_relay(&self) -> String {
        self.relay.clone().unwrap_or_else(|| DEFAULT_RELAY.to_string())
    }
}

pub fn run_set(key: String, value: String) {
    if key.to_lowercase() == "relay" {
        let mut config = DropwireConfig::load();
        config.relay = Some(value.clone());
        if let Err(e) = config.save() {
            eprintln!("Failed to save config: {}", e);
        } else {
            println!("Config updated: {} = {}", key, value);
        }
    } else {
        eprintln!("Unknown config key: {}. Only 'relay' is supported.", key);
    }
}

pub fn run_show() {
    let config = DropwireConfig::load();
    println!("Current Dropwire Configuration:");
    println!("-------------------------------");
    println!("relay: {}", config.relay.unwrap_or_else(|| format!("{} (default)", DEFAULT_RELAY)));
    println!("-------------------------------");
    println!("Config file location: {}", DropwireConfig::config_path().display());
}
