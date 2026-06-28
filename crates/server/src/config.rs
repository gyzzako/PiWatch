use serde::{Deserialize, Serialize};
use std::{env, fs, path::Path};
use core_watch::config::log::{logging::LevelFilter};

const CONFIG_PATH: &str = "config.json";
const DEFAULT_BIND_PORT: u16 = 8888;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub(crate) struct Config {
    pub pihole_url: String,
    pub pihole_pass: String,
    pub bind_port: u16,
    pub log_level: String,
    pub hostname_suffix: Option<String>,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            pihole_url: "pihole_url".to_string(),
            pihole_pass: "pihole_pass".to_string(),
            bind_port: DEFAULT_BIND_PORT,
            log_level: LevelFilter::INFO.to_string(),
            hostname_suffix: None,
        }
    }
}

#[derive(Debug, Clone, Deserialize)]
struct PartialConfig {
    pub pihole_url: Option<String>,
    pub pihole_pass: Option<String>,
    pub bind_port: Option<u16>,
    pub log_level: Option<String>,
    pub hostname_suffix: Option<String>,
}

pub(crate) fn load_config() -> Result<Config, Box<dyn std::error::Error>> {
    let mut config = load_from_file_or_default()?;
    apply_env_overrides(&mut config);

    Ok(config)
}

/// Load JSON if exists, otherwise default
fn load_from_file_or_default() -> Result<Config, Box<dyn std::error::Error>> {
    if Path::new(CONFIG_PATH).exists() {
        let content = fs::read_to_string(CONFIG_PATH)?;
        let partial: PartialConfig = serde_json::from_str(&content)?;

        let mut cfg = Config::default();

        if let Some(v) = partial.pihole_url {
            cfg.pihole_url = v;
        }
        if let Some(v) = partial.pihole_pass {
            cfg.pihole_pass = v;
        }
        if let Some(v) = partial.bind_port {
            cfg.bind_port = v;
        }
        if let Some(v) = partial.log_level {
            cfg.log_level = v;
        }
        if let Some(v) = partial.hostname_suffix {
            cfg.hostname_suffix = Some(v);
        }

        return Ok(cfg);
    }

    // Create default file if missing
    create_default_file()?;
    Ok(Config::default())
}

fn apply_env_overrides(config: &mut Config) {
    if let Ok(v) = env::var("PIHOLE_URL") {
        config.pihole_url = v;
    }

    if let Ok(v) = env::var("PIHOLE_PASS") {
        config.pihole_pass = v;
    }

    if let Ok(v) = env::var("BIND_PORT") {
        if let Ok(p) = v.parse::<u16>() {
            config.bind_port = p;
        }
    }

    if let Ok(v) = env::var("LOG_LEVEL") {
        config.log_level = v;
    }

    if let Ok(v) = env::var("HOSTNAME_SUFFIX") {
        config.hostname_suffix = Some(v);
    }
}

fn create_default_file() -> Result<(), Box<dyn std::error::Error>> {
    let cfg = Config::default();
    let json = serde_json::to_string_pretty(&cfg)?;
    fs::write(CONFIG_PATH, json)?;
    Ok(())
}