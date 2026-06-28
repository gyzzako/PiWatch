use core_watch::dto::http_payloads::{RegisterPayload, RegisterResponse, IpReconciliationPayload};
use core_watch::dto::api::DefaultApiResponse;
use core_watch::logging::{debug, info};
use anyhow::Result;

use crate::heartbeat::HeartbeatResponse;
use crate::config::AgentIdentity;

#[derive(Clone)]
pub(crate) struct ApiClient {
    client: reqwest::Client,
    server_url: String,
    hostname: String,
    identity: AgentIdentity,
}

impl ApiClient {
    pub(crate) fn new(client: reqwest::Client, piwatch_server_url: &str, identity: AgentIdentity) -> Result<Self> {
        Ok(Self {
            client: client,
            server_url: piwatch_server_url.to_string(),
            hostname: hostname::get()?.to_string_lossy().to_string(),
            identity,
        })
    }

    pub(crate) async fn register_agent(&self) -> Result<AgentIdentity> {
        let response = self.client
            .post(format!("{}/register", self.server_url))
            .json(&RegisterPayload {
                install_token: AgentIdentity::install_token(),
                hostname: self.hostname.to_string(),
                agent_version: env!("CARGO_PKG_VERSION").to_string(),
            })
            .send()
            .await?;

        let status = response.status();
        if !status.is_success() {
            let body: DefaultApiResponse = response.json().await?;
            return Err(anyhow::anyhow!(
                "registration failed (status {}): {}",
                status,
                body.message.unwrap_or_else(|| "unknown error".to_string())
            ));
        }

        let register_resp: RegisterResponse = response.json().await?;
        let identity = AgentIdentity {
            agent_id: register_resp.agent_id,
            agent_secret: register_resp.agent_secret,
        };

        match identity.save() {
            Ok(_) => info!("Agent identity saved"),
            Err(e) => return Err(anyhow::anyhow!("Failed to save agent identity: {}", e)),
        }
        Ok(identity)
    }

    pub(crate) async fn send_heartbeat(&self) -> Result<HeartbeatResponse> {
        let response = self.client
            .post(format!("{}/heartbeat", self.server_url))
            .header("X-Agent-Id", &self.identity.agent_id)
            .header("X-Agent-Secret", &self.identity.agent_secret)
            .send()
            .await?;

        let status = response.status();
        let body: DefaultApiResponse = response.json()
            .await
            .unwrap_or(DefaultApiResponse {
                success: false,
                message: Some("invalid response body".to_string()),
            });

        Ok(HeartbeatResponse { status, body })
    }

    pub(crate) async fn reconcile_ip(&self, ipv4: Option<String>) -> Result<()> {
        debug!("Sending IP reconciliation to server: ip={}", ipv4.as_deref().unwrap_or("None"));

        let response = self.client
            .post(format!("{}/reconcile", self.server_url))
            .header("X-Agent-Id", &self.identity.agent_id)
            .header("X-Agent-Secret", &self.identity.agent_secret)
            .json(&IpReconciliationPayload {
                hostname: self.hostname.to_string(),
                ipv4,
            })
            .send()
            .await?;

        let status = response.status();
        if !status.is_success() {
            let body: DefaultApiResponse = response.json().await?;
            return Err(anyhow::anyhow!(
                "IP reconciliation failed (status {}): {}",
                status,
                body.message.unwrap_or_else(|| "unknown error".to_string())
            ));
        }

        info!("IP reconciliated successfully");
        Ok(())
    }
}