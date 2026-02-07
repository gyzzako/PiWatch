use core::dto::register_payload::RegisterPayload;
use core::dto::heart_beat::Heartbeat;
use core::dto::update_id::IpUpdatePayload;
use anyhow::Result;

#[derive(Clone)]
pub(crate) struct ApiClient {
   client: reqwest::Client,
   server_url: &'static str,
   hostname: String,
}

impl ApiClient {
    const SERVER_URL: &'static str = "http://192.168.129.33:8080";

    pub(crate) fn new(client: reqwest::Client) -> Result<Self> {
        Ok(Self {
            client: client,
            server_url: Self::SERVER_URL,
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