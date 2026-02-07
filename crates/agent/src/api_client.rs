use core::dto::register_payload::RegisterPayload;
use core::dto::heart_beat::Heartbeat;
use core::dto::update_id::IpUpdatePayload;
use anyhow::Result;

#[derive(Clone)]
pub(crate) struct ApiClient {
   client: reqwest::Client,
   server_url: &'static str,
}

impl ApiClient {
    const SERVER_URL: &'static str = "http://192.168.129.33:8080";

    pub(crate) fn new(client: reqwest::Client) -> Self {
        Self {
            client: client,
            server_url: Self::SERVER_URL,
        }
    }

    pub(crate) async fn register_agent(&self, node_id: String, ipv4: Option<String>) -> Result<()> {
        let hostname = hostname::get()?.to_string_lossy().to_string();

        let _ = self.client
            .post(format!("{}/register", self.server_url))
            .json(&RegisterPayload {
                node_id,
                hostname,
                agent_version: env!("CARGO_PKG_VERSION").to_string(),
                ipv4,
            })
            .send()
            .await?;

        Ok(())
    }

    pub(crate) async fn send_heartbeat(&self, node_id: String) -> Result<()> {
        let _ = self.client
            .post(format!("{}/heartbeat", self.server_url))
            .json(&Heartbeat {
                node_id,
            })
            .send()
            .await?;

        Ok(())
    }

    pub(crate) async fn update_ip(&self, node_id: String, ipv4: Option<String>, event: String) -> Result<()> {
        let _ = self.client
            .post(format!("{}/update", self.server_url))
            .json(&IpUpdatePayload {
                node_id,
                ipv4,
                event,
            })
            .send()
            .await?;

        Ok(())
    }
}