use core_watch::dto::http_payloads::{RegisterPayload, Heartbeat, IpReconciliationPayload};
use core_watch::logging::{debug, error, info};
use anyhow::Result;

#[derive(Clone)]
pub(crate) struct ApiClient {
   client: reqwest::Client,
   server_url: String,
   hostname: String,
}

impl ApiClient {
    pub(crate) fn new(client: reqwest::Client, piwatch_server_url: &str) -> Result<Self> {
        Ok(Self {
            client: client,
            server_url: piwatch_server_url.to_string(),
            hostname: hostname::get()?.to_string_lossy().to_string(),
        })
    }

    pub(crate) async fn register_agent(&self, ipv4: Option<String>) -> Result<()> {
        let _ = self.client
            .post(format!("{}/register", self.server_url))
            .json(&RegisterPayload {
                hostname: self.hostname.to_string(),
                agent_version: env!("CARGO_PKG_VERSION").to_string(),
                ipv4,
            })
            .send()
            .await?
            .error_for_status()?;

        Ok(())
    }

    pub(crate) async fn send_heartbeat(&self) -> Result<()> {
        let _ = self.client
            .post(format!("{}/heartbeat", self.server_url))
            .json(&Heartbeat {
                hostname: self.hostname.to_string(),
            })
            .send()
            .await?
            .error_for_status()?;

        debug!("Sent heartbeat");

        Ok(())
    }

    pub(crate) async fn reconcile_ip(&self, ipv4: Option<String>) -> Result<()> {
        debug!("Sending IP reconciliation to server: ip={}", ipv4.as_deref().unwrap_or("None"));

        let payload = IpReconciliationPayload {
                hostname: self.hostname.to_string(),
                ipv4,
            };

        let response = self.client
            .post(format!("{}/reconcile", self.server_url))
            .json(&payload)
            .send()
            .await
            .inspect_err(|e| error!("REQWEST ERROR: {:?}", e))?;

        response
            .error_for_status()
            .inspect_err(|e| error!("HTTP ERROR: {:?}", e))?;

        info!("IP reconciliated successfully");
        Ok(())
    }
}