use std::time::Duration;
use tokio::time::sleep;
use tokio::task::JoinHandle;

use crate::api_client::ApiClient;
use core_watch::{dto::api::DefaultApiResponse, logging::{error, warn}};
use reqwest::StatusCode;

pub(crate) struct HeartbeatResponse {
    pub status: StatusCode,
    pub body: DefaultApiResponse,
}

pub(crate) fn start_heartbeat(api: ApiClient) -> JoinHandle<()> {
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
                        StatusCode::UNAUTHORIZED | StatusCode::NOT_FOUND => {
                            warn!("Agent not authenticated, please register. Exiting...");
                            std::process::exit(1);
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

            sleep(Duration::from_secs(60)).await;
        }
    })
}