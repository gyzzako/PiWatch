mod network;
mod api_client;
mod config;

use std::{time::Duration};
use tokio::time::sleep;
use crate::config::load_config;
use crate::{api_client::ApiClient};
use crate::network::IpChangeListener;
use anyhow::Result;

#[tokio::main(flavor = "current_thread")]
async fn main() -> Result<()> {
    let config = match load_config() {
        Ok(cfg) => cfg,
        Err(e) => {
            eprintln!("Failed to load configuration: {}", e);
            return Err(anyhow::anyhow!(e.to_string()));
        }
    };

    let client = reqwest::Client::new();
    let api = ApiClient::new(client.clone(), &config.piwatch_server_url)?;
    let ip_listener: IpChangeListener = match IpChangeListener::init(api.clone(), &config.listening_interface).await {
        Ok(listener) => listener,
        Err(e) => {
            eprintln!("Failed to create IP change listener: {}", e);
            return Err(e.into());
        }
    };

    let ip: Option<String> = match ip_listener.get_initial_ipv4().await {
        Some(ip) => Some(ip.to_string()),
        None => None,
    };

    match api.register_agent(ip).await {
        Ok(_) => println!("Successfully registered agent."),
        Err(e) => {
            eprintln!("Failed to register agent: {}", e);
            return Err(e.into());
        }
    };

    // heartbeat
    {
        let api = api.clone();
        tokio::spawn(async move {
            loop {
                match api.send_heartbeat().await {
                    Ok(_) => (),
                    Err(e) => eprintln!("Failed to send heartbeat: {}", e),
                }

                sleep(Duration::from_secs(30)).await;
            }
        });
    }

    println!("Node started");
    let _ = ip_listener.start().await?.await?;

    Ok(())
}
