mod config;
mod network;
mod api_client;

use std::{
    time::Duration
};
use tokio::time::sleep;
use crate::{config::load_or_create_node_id, api_client::ApiClient};
use crate::network::IpChangeListener;
use anyhow::Result;

#[tokio::main(flavor = "current_thread")]
async fn main() -> Result<()> {
    let node_id = match load_or_create_node_id() {
        Ok(id) => id,
        Err(e) => {
            eprintln!("Failed to load or create node ID: {}", e);
            return Err(e.into());
        }
    };

    let client = reqwest::Client::new();
    let api = ApiClient::new(client.clone());
    let ip_listener: IpChangeListener = match IpChangeListener::init(api.clone()).await {
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

    match api.register_agent(node_id.clone(), ip).await {
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
                match api.send_heartbeat(node_id.clone()).await {
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
