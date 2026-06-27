use std::time::Duration;
use tokio::time::sleep;
use tokio::task::JoinHandle;
use crate::api_client::ApiClient;
use crate::network::IpChangeListener;
use core_watch::logging::{error, info, warn};

pub fn start_heartbeat(
    api: ApiClient,
    ip_listener: std::sync::Arc<IpChangeListener>,
) -> JoinHandle<()> {
    return tokio::spawn(async move {
        loop {
            match api.send_heartbeat().await {
                Ok(_) => {}

                Err(e) => {
                    if let Some(reqwest_err) = e.downcast_ref::<reqwest::Error>() {
                        if let Some(status) = reqwest_err.status() {
                            if status == reqwest::StatusCode::NOT_FOUND {
                                warn!("Agent not registered. Re-registering...");

                                let ip = ip_listener
                                    .get_current_ip()
                                    .await
                                    .map(|ip| ip.to_string());

                                if let Err(e) = api.register_agent(ip).await {
                                    error!("Re-register failed: {e}");
                                    std::process::exit(1);
                                } else {
                                    info!("Re-registered successfully");
                                }
                            } else if status == reqwest::StatusCode::FORBIDDEN {
                                error!("Identity mismatch detected. Exiting...");
                                std::process::exit(1);
                            } else {
                                error!("Heartbeat error: {e}");
                            }
                        }
                    } else {
                        error!("Heartbeat error: {e}");
                    }
                }
            }

            sleep(Duration::from_secs(30)).await;
        }
    });
}