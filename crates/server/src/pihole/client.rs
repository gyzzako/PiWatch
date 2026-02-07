use tokio::sync::Mutex;

use url::{Url, form_urlencoded};

use crate::pihole::dto::AuthResponse;

pub(crate) struct PiholeClient {
    client: reqwest::Client,
    server_url: String,
    server_pass: &'static str,
    current_sid: Mutex<Option<String>>, // TODO: move to DB
}

impl PiholeClient {
    // TODO: move to config
    const SERVER_URL: &str = "";
    const SERVER_PASS: &str = "";
    
    pub(crate) fn new(client: reqwest::Client) -> Self {
        Self {
            client: client,
            server_url: format!("{}/{}", Self::SERVER_URL, "api"),
            server_pass: Self::SERVER_PASS,
            current_sid: Mutex::new(None),
        }
    }

    pub(crate) async fn put_ip(&self, hostname: &str, ip: &str) -> Result<(), Box<dyn std::error::Error>> {
        let is_auth_valid: bool = self.is_auth_valid().await?;

        if !is_auth_valid {
            let auth_response: AuthResponse = self.create_auth().await?;

            if !auth_response.session.valid || auth_response.session.sid.is_none() {
                return Err("Authentication failed".into());
            }

            self.set_current_sid(auth_response.session.sid.unwrap()).await;
        }

        let kv = form_urlencoded::byte_serialize(format!("{} {}", ip, hostname).as_bytes()).collect::<String>();
        let api_path = format!("{}/{}", self.api_path("config/dns/hosts"), kv);

        let url = Url::parse(&api_path)?;

        let sid: String = self.get_current_sid().await.ok_or("Unexpected authentication failure")?;

        // TODO: handle error when hostname-ip already exists
        match self.client
            .put(url)
            .header("sid",  sid)
            .send()
            .await {
                Ok(resp) => {
                    if resp.status().is_success() {
                        Ok(())
                    } else {
                        Err(format!("Failed to put IP: HTTP {}", resp.status()).into())
                    }
                },
                Err(e) => Err(format!("Failed to put IP: {}", e).into()),
            }
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
                "password": self.server_pass,
            }))
            .send()
            .await?
            .json::<AuthResponse>()
            .await?;

        Ok(response)
    }

    fn api_path(&self, path: &str) -> String {
        format!("{}/{}", self.server_url, path)
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