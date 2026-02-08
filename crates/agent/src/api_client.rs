use core::dto::register_payload::RegisterPayload;
use core::dto::heart_beat::Heartbeat;
use core::dto::update_id::IpUpdatePayload;
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
            .await?;

        Ok(())
    }

    pub(crate) async fn send_heartbeat(&self) -> Result<()> {
        let _ = self.client
            .post(format!("{}/heartbeat", self.server_url))
            .json(&Heartbeat {
                hostname: self.hostname.to_string(),
            })
            .send()
            .await?;

        Ok(())
    }

    pub(crate) async fn update_ip(&self, ipv4: Option<String>, event: String) -> Result<()> {
        let _ = self.client
            .post(format!("{}/update", self.server_url))
            .json(&IpUpdatePayload {
                hostname: self.hostname.to_string(),
                ipv4,
                event,
            })
            .send()
            .await?;

        Ok(())
    }
}