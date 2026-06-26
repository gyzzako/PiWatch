use tokio::sync::Mutex;

use core_watch::logging::{debug, info};
use url::{Url, form_urlencoded};
use crate::{config::Config, pihole::dto::AuthResponse};

pub(crate) struct PiholeClient {
    client: reqwest::Client,
    config: Config,
    pihole_url: String,
    current_sid: Mutex<Option<String>>,
}

impl PiholeClient {
    pub(crate) fn new(client: reqwest::Client, config: Config) -> Self {
        Self {
            client,
            config: config.clone(),
            pihole_url: format!("{}/{}", &config.pihole_url, "api"),
            current_sid: Mutex::new(None),
        }
    }

    pub(crate) async fn reconcile_ip_for_hostname(&self, hostname: &str, new_ip: &str) -> Result<(), Box<dyn std::error::Error>> {
        self.use_auth().await?;
        let processed_hostname = if let Some(suffix) = &self.config.hostname_suffix {
            format!("{}.{}", hostname, suffix)
        } else {
            hostname.to_string()
        };

        let sid = self
            .get_current_sid()
            .await
            .ok_or("Unexpected authentication failure")?;

        // 1. Fetch current hosts config
        let url = self.api_path("config/dns/hosts");

        let response = self
            .client
            .get(url)
            .header("sid", &sid)
            .send()
            .await?
            .error_for_status()?;

        let body: serde_json::Value = response.json().await?;

        let hosts = body["config"]["dns"]["hosts"]
            .as_array()
            .ok_or("Invalid hosts format")?;

        // 2. Collect IPs currently associated with hostname
        let mut ips_to_delete: Vec<String> = Vec::new();

        for entry in hosts {
            if let Some(entry_str) = entry.as_str() {
                let mut parts = entry_str.split_whitespace();

                let ip = parts.next();
                let host = parts.next();

                if let (Some(ip), Some(host)) = (ip, host) {
                    if host == processed_hostname {
                        ips_to_delete.push(ip.to_string());
                    }
                }
            }
        }

        // 3. Delete old entries
        for ip in ips_to_delete {
            let kv = form_urlencoded::byte_serialize(format!("{} {}", ip, processed_hostname).as_bytes())
                .collect::<String>();

            let delete_url = Url::parse(&self.api_path(&format!("config/dns/hosts/{}", kv)))?;

            let resp = self
                .client
                .delete(delete_url)
                .header("sid", &sid)
                .send()
                .await?;

            if !resp.status().is_success() {
                return Err(format!(
                    "Failed to delete old IP {} for {}: HTTP {}",
                    ip,
                    processed_hostname,
                    resp.status()
                )
                .into());
            }

            info!("Deleted old mapping {} -> {}", processed_hostname, ip);
        }

        // 4. Add new mapping
        let kv = form_urlencoded::byte_serialize(format!("{} {}", new_ip, processed_hostname).as_bytes())
            .collect::<String>();

        let put_url = Url::parse(&self.api_path(&format!("config/dns/hosts/{}", kv)))?;

        let resp = self
            .client
            .put(put_url)
            .header("sid", &sid)
            .send()
            .await?;

        if resp.status().is_success() {
            info!(
                "Reconciled IP for {}: now {} (old entries removed)",
                processed_hostname, new_ip
            );
            Ok(())
        } else {
            Err(format!("Failed to put new IP: HTTP {}", resp.status()).into())
        }
    }

    async fn use_auth(&self) -> Result<(), Box<dyn std::error::Error>> {
        let is_auth_valid: bool = self.is_auth_valid().await?;

        if !is_auth_valid {
            debug!("Creating new Pihole auth session");

            let auth_response: AuthResponse = self.create_auth().await?;

            if !auth_response.session.valid || auth_response.session.sid.is_none() {
                return Err("Authentication failed".into());
            }

            self.set_current_sid(auth_response.session.sid.unwrap()).await;
        }

        Ok(())
    }

    async fn is_auth_valid(&self) -> Result<bool, Box<dyn std::error::Error>> {
        let sid = match self.get_current_sid().await {
            Some(s) => s,
            None => return Ok(false)
        };

        let response = self.client
            .get(self.api_path("auth/sessions"))
            .header("sid",  sid)
            .send()
            .await?;

        Ok(response.status().is_success())
    }

    async fn create_auth(&self) -> Result<AuthResponse, Box<dyn std::error::Error>> {
        let response = self.client
            .post(self.api_path("auth"))
            .json(&serde_json::json!({
                "password": &self.config.pihole_pass,
            }))
            .send()
            .await?
            .json::<AuthResponse>()
            .await?;

        Ok(response)
    }

    fn api_path(&self, path: &str) -> String {
        format!("{}/{}", self.pihole_url, path)
    }

    async fn get_current_sid(&self) -> Option<String> {
        let sid_lock = self.current_sid.lock().await;
        match &*sid_lock {
            Some(s) => Some(s.clone()),
            None => None,
        }
    }

    async fn set_current_sid(&self, sid: String) {
        let mut sid_lock = self.current_sid.lock().await;
        *sid_lock = Some(sid);
    }

}