use serde::{Deserialize, Serialize};
use std::path::Path;

const CONFIG_PATH: &str = "config.json";
const DEFAULT_BIND_PORT: u16 = 8887;
const DEFAULT_LISTENING_INTERFACE: &'static str = "eth0";

pub fn load_config() -> Result<Config, Box<dyn std::error::Error>> {
    match load_config_from_env() {
        Ok(config) => Ok(config),
        Err(_) => match Path::new(CONFIG_PATH).exists() {
            true => load_config_from_json(),
            false => {
                create_default_config_file()?;
                Err(Box::new(std::io::Error::new(
                    std::io::ErrorKind::NotFound,
                    format!("You need to set a valid PIWATCH_SERVER_URL in environment variables or edit the created {} file", CONFIG_PATH),
                )))
            }
        },
    }
}

fn load_config_from_env() -> Result<Config, std::io::Error> {
    let piwatch_server_url = std::env::var("PIWATCH_SERVER_URL").map_err(|_| {
        std::io::Error::new(
            std::io::ErrorKind::InvalidInput,
            "PIWATCH_SERVER_URL env var is undefined",
        )
    })?;

    let listening_interface = std::env::var("LISTENING_INTERFACE")
        .unwrap_or(DEFAULT_LISTENING_INTERFACE.to_string());

    let bind_port = std::env::var("BIND_PORT")
        .unwrap_or(DEFAULT_BIND_PORT.to_string())
        .parse::<u16>()
        .map_err(|_| {
            std::io::Error::new(
                std::io::ErrorKind::InvalidInput,
                "BIND_PORT must be a valid number between 1 and 65535",
            )
        })?;

    Ok(Config {
        piwatch_server_url,
        listening_interface,
        bind_port,
    })
}

fn load_config_from_json() -> Result<Config, Box<dyn std::error::Error>> {
    if Path::new(CONFIG_PATH).exists() {
        let file = std::fs::File::open(CONFIG_PATH)?;
        let reader = std::io::BufReader::new(file);
        let config: Config = serde_json::from_reader(reader)?;
        let default_config = Config::default();
        if config.piwatch_server_url == default_config.piwatch_server_url {
            return Err(Box::new(std::io::Error::new(
                std::io::ErrorKind::InvalidInput,
                format!("Set a valid PIWATCH_SERVER_URL in {}", CONFIG_PATH),
            )));
        }
        return Ok(config);
    }
    
    create_default_config_file()?;
    Err(Box::new(std::io::Error::new(
        std::io::ErrorKind::NotFound,
        "Configuration file not found, created a default one",
    )))
}

fn create_default_config_file() -> Result<(), Box<dyn std::error::Error>> {
    let default_config = Config::default();
    let file = std::fs::File::create(CONFIG_PATH)?;
    serde_json::to_writer_pretty(file, &default_config)?;

    Ok(())
}

#[derive(Serialize, Deserialize)]
pub(crate) struct Config {
    pub piwatch_server_url: String,
    pub listening_interface: String,
    pub bind_port: u16,
}

impl Default for Config {
    fn default() -> Self {
        Config {
            piwatch_server_url: "piwatch_server_url".to_string(),
            listening_interface: DEFAULT_LISTENING_INTERFACE.to_string(),
            bind_port: DEFAULT_BIND_PORT,
        }
    }
}