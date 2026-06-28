use serde::{Deserialize, Serialize};
use std::{env, fs, path::Path};
use core_watch::config::log::logging::LevelFilter;

const CONFIG_PATH: &str = "config.json";
const IDENTITY_PATH: &str = "/etc/piwatch/identity.json";
const DEFAULT_BIND_PORT: u16 = 8887;
const DEFAULT_LISTENING_INTERFACE: &str = "eth0";

#[derive(Debug, Clone, Serialize, Deserialize)]
pub(crate) struct Config {
    pub piwatch_server_url: String,
    pub listening_interface: String,
    pub bind_port: u16,
    pub log_level: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub(crate) struct AgentIdentity {
    pub agent_id: String,
    pub agent_secret: String,
}

impl AgentIdentity {
    pub fn load() -> Result<Self, Box<dyn std::error::Error>> {
        let path = Path::new(IDENTITY_PATH);
        if path.exists() {
            let content = fs::read_to_string(path)?;
            let identity: AgentIdentity = serde_json::from_str(&content)?;
            return Ok(identity);
        }
        Err("No agent identity found. Please register first.".into())
    }

    pub fn save(&self) -> Result<(), Box<dyn std::error::Error>> {
        let path = Path::new(IDENTITY_PATH);
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent)?;
        }
        let json = serde_json::to_string_pretty(self)?;
        fs::write(path, json)?;
        Ok(())
    }

    pub fn install_token() -> String {
        std::env::var("PIWATCH_INSTALL_TOKEN").expect("install token not found in environment variable PIWATCH_INSTALL_TOKEN")
    }
}

impl Default for Config {
    fn default() -> Self {
        Self {
            piwatch_server_url: "piwatch_server_url".to_string(),
            listening_interface: DEFAULT_LISTENING_INTERFACE.to_string(),
            bind_port: DEFAULT_BIND_PORT,
            log_level: LevelFilter::INFO.to_string(),
        }
    }
}

#[derive(Debug, Clone, Deserialize)]
struct PartialConfig {
    pub piwatch_server_url: Option<String>,
    pub listening_interface: Option<String>,
    pub bind_port: Option<u16>,
    pub log_level: Option<String>,
}

pub(crate) fn load_config() -> Result<Config, Box<dyn std::error::Error>> {
    let mut config = load_from_file_or_default()?;
    apply_env_overrides(&mut config);
    Ok(config)
}

fn load_from_file_or_default() -> Result<Config, Box<dyn std::error::Error>> {
    if Path::new(CONFIG_PATH).exists() {
        let content = fs::read_to_string(CONFIG_PATH)?;
        let partial: PartialConfig = serde_json::from_str(&content)?;

        let mut cfg = Config::default();

        if let Some(v) = partial.piwatch_server_url {
            cfg.piwatch_server_url = v;
        }
        if let Some(v) = partial.listening_interface {
            cfg.listening_interface = v;
        }
        if let Some(v) = partial.bind_port {
            cfg.bind_port = v;
        }
        if let Some(v) = partial.log_level {
            cfg.log_level = v;
        }

        return Ok(cfg);
    }

    create_default_file()?;
    Ok(Config::default())
}

fn apply_env_overrides(config: &mut Config) {
    if let Ok(v) = env::var("PIWATCH_SERVER_URL") {
        config.piwatch_server_url = v;
    }

    if let Ok(v) = env::var("LISTENING_INTERFACE") {
        config.listening_interface = v;
    }

    if let Ok(v) = env::var("BIND_PORT") {
        if let Ok(p) = v.parse::<u16>() {
            config.bind_port = p;
        }
    }

    if let Ok(v) = env::var("LOG_LEVEL") {
        config.log_level = v;
    }
}

fn create_default_file() -> Result<(), Box<dyn std::error::Error>> {
    let cfg = Config::default();
    let json = serde_json::to_string_pretty(&cfg)?;
    fs::write(CONFIG_PATH, json)?;
    Ok(())
}