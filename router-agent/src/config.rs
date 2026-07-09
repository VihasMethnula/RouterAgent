use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Config {
    pub web: WebConfig,
    pub router: RouterConfig,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WebConfig {
    pub port: u16,
    pub host: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RouterConfig {
    pub url: String,
    pub network: String,
}

impl Default for Config {
    fn default() -> Self {
        Config {
            web: WebConfig {
                port: 5090,
                host: "0.0.0.0".to_string(),
            },
            router: RouterConfig {
                url: "http://192.168.4.1".to_string(),
                network: "192.168.4.0/24".to_string(),
            },
        }
    }
}

impl Config {
    pub fn load() -> Self {
        let config_path = Self::config_path();
        
        if config_path.exists() {
            match fs::read_to_string(&config_path) {
                Ok(content) => {
                    match serde_yaml::from_str(&content) {
                        Ok(config) => {
                            println!("Loaded config from: {}", config_path.display());
                            return config;
                        }
                        Err(e) => {
                            eprintln!("Failed to parse config: {}. Using defaults.", e);
                        }
                    }
                }
                Err(e) => {
                    eprintln!("Failed to read config: {}. Using defaults.", e);
                }
            }
        } else {
            println!("No config file found at: {}", config_path.display());
            println!("Creating default config...");
            let default_config = Config::default();
            default_config.save();
            return default_config;
        }
        
        Config::default()
    }
    
    pub fn save(&self) {
        let config_path = Self::config_path();
        
        if let Some(parent) = config_path.parent() {
            if let Err(e) = fs::create_dir_all(parent) {
                eprintln!("Failed to create config directory: {}", e);
                return;
            }
        }
        
        match serde_yaml::to_string(self) {
            Ok(yaml) => {
                if let Err(e) = fs::write(&config_path, yaml) {
                    eprintln!("Failed to write config: {}", e);
                } else {
                    println!("Config saved to: {}", config_path.display());
                }
            }
            Err(e) => {
                eprintln!("Failed to serialize config: {}", e);
            }
        }
    }
    
    fn config_path() -> PathBuf {
        let home = dirs::home_dir().unwrap_or_else(|| PathBuf::from("."));
        home.join(".config").join("router").join("config.yaml")
    }
}
