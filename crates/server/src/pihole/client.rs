use std::sync::RwLock;

use serde_json::Value;

use core_watch::logging::{info};
use url::{Url, form_urlencoded};
use crate::{config::Config, pihole::dto::AuthResponse};

pub(crate) struct PiholeClient {
    client: reqwest::Client,
    config: Config,
    pihole_url: String,
    current_sid: RwLock<Option<String>>,
}

impl PiholeClient {
    pub(crate) fn new(client: reqwest::Client, config: Config) -> Self {
        Self {
            client,
            config: config.clone(),
            pihole_url: format!("{}/{}", &config.pihole_url, "api"),
            current_sid: RwLock::new(None),
        }
    }

    pub(crate) async fn reconcile_ip_for_hostname(&self, hostname: &str, new_ip: &str) -> Result<(), Box<dyn std::error::Error>> {
        let processed_hostname = if let Some(suffix) = &self.config.hostname_suffix {
            format!("{}.{}", hostname, suffix)
        } else {
            hostname.to_string()
        };

        // 1. Fetch current hosts config
        let hosts = &self.get_ip_entries().await?;

        // 2. Collect IPs currently associated with hostname
        let mut current_associated_ips: Vec<String> = Vec::new();

        for entry in hosts {
            if let Some(entry_str) = entry.as_str() {
                let mut parts = entry_str.split_whitespace();

                let ip = parts.next();
                let host = parts.next();

                if let (Some(ip), Some(host)) = (ip, host) {
                    if host == processed_hostname {
                        current_associated_ips.push(ip.to_string());
                    }
                }
            }
        }

        // Check if new_ip is already present in hosts
        let new_ip_already_present = hosts.iter().any(|entry| {
            entry.as_str()
                .map(|e| e == format!("{} {}", new_ip, processed_hostname))
                .unwrap_or(false)
        });

        // Filter out new_ip from deletion list - don't delete if it's already present
        let ips_to_delete: Vec<String> = current_associated_ips
            .into_iter()
            .filter(|ip| ip.as_str() != new_ip)
            .collect();

        // 3. Delete old entries
        self.delete_ip_for_hostname(&processed_hostname, ips_to_delete).await?;

        // 4. If new_ip was already present, skip PUT
        if new_ip_already_present {
            info!("IP {} already present for hostname={}, no update needed", new_ip, processed_hostname);
            return Ok(());
        }

        // 5. Add new mapping
        self.update_ip_entry_for_hostname(&processed_hostname, new_ip).await
    }

    async fn get_ip_entries(&self) -> Result<Vec<Value>, Box<dyn std::error::Error>> {
        let result: reqwest::Response = self.request_with_sid(|sid| {
            let url = self.api_path("config/dns/hosts");
            async move {
                self.client
                    .get(url)
                    .header("sid", sid)
                    .send()
                    .await
                }
        })
        .await?
        .error_for_status()?;

        let result: Value = result.json().await?;

        let hosts = result["config"]["dns"]["hosts"]
            .as_array()
            .ok_or("Invalid hosts format")?
            .clone();

        Ok(hosts)
    }

    async fn update_ip_entry_for_hostname(
        &self,
        processed_hostname: &str,
        new_ip: &str,
    ) -> Result<(), Box<dyn std::error::Error>> {
        let kv = form_urlencoded::byte_serialize(
            format!("{} {}", new_ip, processed_hostname).as_bytes()
        )
        .collect::<String>();

        let url = Url::parse(&self.api_path(&format!("config/dns/hosts/{}", kv)))?;


        let resp: reqwest::Response = self
            .request_with_sid(|sid| {
                let url = url.clone();
                async move {
                    self.client
                        .put(url)
                        .header("sid", sid)
                        .send()
                        .await
                }
            })
            .await?;

        if resp.status().is_success() {
            info!("Reconciled IP for {} → {}", processed_hostname, new_ip);
            Ok(())
        } else {
            Err(format!("Failed PUT: HTTP {}", resp.status()).into())
        }
    }
    
    async fn delete_ip_for_hostname(
        &self,
        processed_hostname: &str,
        ips_to_delete: Vec<String>,
    ) -> Result<(), Box<dyn std::error::Error>> {
        for ip in ips_to_delete {
            let kv = form_urlencoded::byte_serialize(
                format!("{} {}", ip, processed_hostname).as_bytes()
            )
            .collect::<String>();

            let url = Url::parse(&self.api_path(&format!("config/dns/hosts/{}", kv)))?;

            let resp: reqwest::Response = self
                .request_with_sid(|sid| {
                    let url = url.clone();
                    async move {
                        self.client
                            .delete(url)
                            .header("sid", sid)
                            .send()
                            .await
                        }
                    })
                .await?;

            if !resp.status().is_success() {
                return Err(format!(
                    "Failed delete {} {}: HTTP {}",
                    ip,
                    processed_hostname,
                    resp.status()
                )
                .into());
            }

            info!("Deleted {} → {}", processed_hostname, ip);
        }

        Ok(())
    }

    async fn create_auth(&self) -> Result<AuthResponse, Box<dyn std::error::Error>> {
        let response: AuthResponse = self.client
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

    async fn ensure_sid(&self) -> Result<String, Box<dyn std::error::Error>> {
        if let Some(sid) = self.get_current_sid() {
            return Ok(sid);
        }

        let auth = self.create_auth().await?;

        let sid = auth
            .session
            .sid
            .ok_or("Authentication failed: missing SID")?;

        self.set_current_sid(sid.clone());
        Ok(sid)
    }

    async fn request_with_sid<F, Fut>(
        &self,
        mut req: F,
    ) -> Result<reqwest::Response, Box<dyn std::error::Error>>
    where
        F: FnMut(String) -> Fut,
        Fut: std::future::Future<Output = Result<reqwest::Response, reqwest::Error>>,
    {
        let sid = self.ensure_sid().await?;

        let resp = req(sid.clone()).await?;

        if resp.status() != reqwest::StatusCode::UNAUTHORIZED {
            return Ok(resp.error_for_status()?);
        }

        let auth = self.create_auth().await?;
        let sid = auth
            .session
            .sid
            .ok_or("Authentication failed after refresh")?;

        self.set_current_sid(sid.clone());

        let resp = req(sid).await?;
        Ok(resp)
    }

    fn api_path(&self, path: &str) -> String {
        format!("{}/{}", self.pihole_url, path)
    }

    fn get_current_sid(&self) -> Option<String> {
        self.current_sid.read().unwrap().clone()
    }

    fn set_current_sid(&self, sid: String) {
        *self.current_sid.write().unwrap() = Some(sid);
    }
}