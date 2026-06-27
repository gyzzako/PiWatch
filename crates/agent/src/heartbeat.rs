use std::time::Duration;
use tokio::time::sleep;
use tokio::task::JoinHandle;
use std::sync::Arc;

use crate::api_client::ApiClient;
use crate::network::IpChangeListener;
use core_watch::{dto::api::DefaultApiResponse, logging::{error, info, warn}};
use reqwest::StatusCode;

pub struct HeartbeatResponse {
    pub status: StatusCode,
    pub body: DefaultApiResponse,
}

pub fn start_heartbeat(
    api: ApiClient,
    ip_listener: Arc<IpChangeListener>,
) -> JoinHandle<()> {
    tokio::spawn(async move {
        loop {
            match api.send_heartbeat().await {
                Ok(resp) => {
                    if !resp.body.success {
                        error!(
                            "Heartbeat rejected (status {}): {}",
                            resp.status,
                            resp.body.message.unwrap_or_else(|| "unknown error".to_string())
                        );
                    }

                    match resp.status {
                        StatusCode::NOT_FOUND => {
                            warn!("Agent not registered. Re-registering...");

                            let ip = ip_listener
                                .get_current_ip()
                                .await
                                .map(|ip| ip.to_string());

                            if let Err(e) = api.register_agent(ip).await {
                                error!("Re-register failed: {e}");
                                std::process::exit(1);
                            }

                            info!("Re-registered successfully");
                        }

                        StatusCode::FORBIDDEN => {
                            error!("Identity mismatch detected. Exiting...");
                            std::process::exit(1);
                        }

                        _ => {}
                    }
                }

                Err(e) => {
                    error!("Heartbeat transport error: {e}");
                }
            }

            sleep(Duration::from_secs(30)).await;
        }
    })
}