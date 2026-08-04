use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;

pub const DEFAULT_RELAY: &str = "ws://relay.dropwire.tyes.dev:9010";

#[derive(Serialize, Deserialize, Debug)]
pub struct DropwireConfig {
    pub relay: Option<String>,
    pub no_lan: Option<bool>,
    pub download_dir: Option<String>,
    pub default_mode: Option<String>,
    pub parallel_streams: Option<u8>,
    pub chunk_size_kb: Option<u32>,
    pub theme: Option<String>,
}

impl Default for DropwireConfig {
    fn default() -> Self {
        Self {
            relay: Some(DEFAULT_RELAY.to_string()),
            no_lan: Some(false),
            download_dir: None,
            default_mode: Some("browser".to_string()),
            parallel_streams: Some(4),
            chunk_size_kb: Some(1024),
            theme: Some("cyberpunk".to_string()),
        }
    }
}

impl DropwireConfig {
    pub fn config_path() -> PathBuf {
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

    pub fn get_no_lan(&self) -> bool {
        self.no_lan.unwrap_or(false)
    }

    pub fn get_download_dir(&self) -> Option<PathBuf> {
        self.download_dir.as_ref().map(PathBuf::from)
    }

    pub fn get_default_mode(&self) -> String {
        self.default_mode.clone().unwrap_or_else(|| "browser".to_string())
    }

    pub fn get_parallel_streams(&self) -> u8 {
        self.parallel_streams.unwrap_or(4)
    }

    pub fn get_chunk_size_kb(&self) -> u32 {
        self.chunk_size_kb.unwrap_or(1024)
    }

    pub fn get_theme(&self) -> String {
        self.theme.clone().unwrap_or_else(|| "cyberpunk".to_string())
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
