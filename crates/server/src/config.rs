use serde::{Deserialize, Serialize};
use std::path::Path;

const CONFIG_PATH: &'static str = "config.json";
const DEFAULT_BIND_PORT: u16 = 8888;

pub fn load_config() -> Result<Config, Box<dyn std::error::Error>> {
    match load_config_from_env() {
        Ok(config) => Ok(config),
        Err(_) => match Path::new(CONFIG_PATH).exists() {
            true => load_config_from_json(),
            false => {
                create_default_config_file()?;
                Err(Box::new(std::io::Error::new(
                    std::io::ErrorKind::NotFound,
                    format!("You need to set a valid PIHOLE_URL and PIHOLE_PASS in environment variables or edit the created {} file.", CONFIG_PATH),
                )))
            }
        },
    }
}

fn load_config_from_env() -> Result<Config, std::io::Error> {
    let pihole_url = std::env::var("PIHOLE_URL").map_err(|_| {
        std::io::Error::new(
            std::io::ErrorKind::InvalidInput,
            "PIHOLE_URL env var is undefined",
        )
    })?;

    let pihole_pass = std::env::var("PIHOLE_PASS").map_err(|_| {
        std::io::Error::new(
            std::io::ErrorKind::InvalidInput,
            "PIHOLE_PASS env var is undefined",
        )
    })?;

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
        pihole_url,
        pihole_pass,
        bind_port,
    })
}

fn load_config_from_json() -> Result<Config, Box<dyn std::error::Error>> {
    if Path::new(CONFIG_PATH).exists() {
        let file = std::fs::File::open(CONFIG_PATH)?;
        let reader = std::io::BufReader::new(file);
        let config: Config = serde_json::from_reader(reader)?;
        let default_config = Config::default();
        if config.pihole_url == default_config.pihole_url || config.pihole_pass == default_config.pihole_pass {
            return Err(Box::new(std::io::Error::new(
                std::io::ErrorKind::InvalidInput,
                format!("Set a valid PIHOLE_URL and PIHOLE_PASS in {}", CONFIG_PATH),
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
    pub pihole_url: String,
    pub pihole_pass: String,
    pub bind_port: u16,
}

impl Default for Config {
    fn default() -> Self {
        Config {
            pihole_url: "pihole_url".to_string(),
            pihole_pass: "pihole_pass".to_string(),
            bind_port: DEFAULT_BIND_PORT,
        }
    }
}