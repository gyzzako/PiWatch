use std::sync::Arc;
use std::time::{Duration, SystemTime};
use tokio::task::JoinHandle;

use core_watch::dto::http_payloads::{RegisterPayload, RegisterResponse};
use core_watch::logging::{warn, debug, info};

use crate::domain::model::Agent;
use crate::domain::repository::AgentRepository;
use crate::domain::security::CryptoService;
use crate::dto::agent_summary::AgentSummary;
use crate::error::{Error, Result};
use crate::pihole::client::PiholeClient;

pub(crate) struct AgentService {
    agent_repo: Arc<dyn AgentRepository>,
    pihole: Arc<PiholeClient>,
}

impl AgentService {
    pub fn new(agent_repo: Arc<dyn AgentRepository>, pihole: Arc<PiholeClient>) -> Self {
        Self { agent_repo, pihole }
    }

    pub async fn register(&self, payload: RegisterPayload) -> Result<RegisterResponse> {
        let valid_token = self.validate_install_token(&payload.install_token).await?;

        if !valid_token {
            return Err(Error::Unauthorized);
        }

        crate::version::check_compatible(&payload.agent_version, env!("CARGO_PKG_VERSION"))?;

        let existing = self.agent_repo.get_agent_by_hostname(&payload.hostname).await?;
        if let Some(a) = existing {
            return Err(Error::Conflict(format!(
                "Hostname {} already registered as agent_id={}",
                payload.hostname,
                a.agent_id
            )));
        }

        let agent_id = uuid::Uuid::new_v4().to_string();
        let agent_secret = CryptoService::generate_secret();
        let salt = CryptoService::generate_salt();
        let secret_hash = CryptoService::hash_with_salt(&agent_secret, &salt);
        let now = SystemTime::now()
            .duration_since(SystemTime::UNIX_EPOCH)
            .unwrap()
            .as_secs();

        let agent = Agent {
            agent_id: agent_id.clone(),
            secret_hash,
            salt,
            hostname: payload.hostname.clone(),
            agent_version: payload.agent_version,
            ipv4: None,
            last_seen_secs: now,
            created_at: SystemTime::now(),
            revoked: false,
            deactivated_at: None,
        };

        self.agent_repo.create_agent(&agent).await?;

        debug!("Agent {} registered successfully", agent.name());

        Ok(RegisterResponse {
            agent_id,
            agent_secret,
        })
    }

    pub async fn reconcile_ip(&self, agent: &Agent, ipv4: Option<String>) -> Result<()> {
        let Some(ip) = ipv4 else {
            return Err(Error::BadRequest("Missing IPv4 address".to_string()));
        };

        info!("Received IP reconciliation for agent={} ip={}", agent.name(), ip);

        self.pihole
            .reconcile_ip_for_hostname(&agent.hostname, &ip)
            .await
            .map_err(|e| Error::Internal(format!("DNS reconciliation failed: {}", e)))?;

        self.agent_repo.update_agent_ip(&agent.agent_id, &ip).await?;

        info!("Reconciled IP for agent={} ip={}", agent.name(), ip);
        Ok(())
    }

    pub async fn update_agent_version(&self, agent_id: &str, version: &str) -> Result<()> {
        self.agent_repo.update_agent_version(agent_id, version).await
    }

    pub async fn heartbeat(&self, agent_id: &str) -> Result<()> {
        self.agent_repo.update_agent_last_seen(agent_id).await?;
        Ok(())
    }

    pub async fn list_agents(&self) -> Result<Vec<Agent>> {
        self.agent_repo.list_agents().await
    }

    pub async fn list_agent_summaries(&self) -> Result<Vec<AgentSummary>> {
        let agents = self.agent_repo.list_agents().await?;

        let now = SystemTime::now()
            .duration_since(SystemTime::UNIX_EPOCH)
            .unwrap()
            .as_secs();

        let summaries: Vec<AgentSummary> = agents
            .into_iter()
            .map(|agent| {
                let elapsed = now.saturating_sub(agent.last_seen_secs);

                AgentSummary {
                    hostname: agent.name(),
                    agent_version: agent.agent_version,
                    ipv4: agent.ipv4.unwrap_or_default(),
                    online: agent.deactivated_at.is_none(),
                    last_seen_sec: elapsed,
                    registered_at: agent.created_at,
                }
            })
            .collect();

        Ok(summaries)
    }

    pub async fn get_agent(&self, agent_id: &str) -> Result<Option<Agent>> {
        self.agent_repo.get_agent(agent_id).await
    }

    pub async fn validate_install_token(&self, plaintext: &str) -> Result<bool> {
        self.agent_repo.validate_install_token(plaintext).await
    }

    pub fn start_monitor(&self) -> JoinHandle<()> {
        let repo = self.agent_repo.clone();
        tokio::spawn(async move {
            loop {
                let agents = match repo.list_agents_with_last_seen().await {
                    Ok(agents) => agents,
                    Err(e) => {
                        warn!("Failed to list agents for monitor: {}", e);
                        tokio::time::sleep(Duration::from_secs(60)).await;
                        continue;
                    }
                };

                let now = SystemTime::now()
                    .duration_since(SystemTime::UNIX_EPOCH)
                    .unwrap()
                    .as_secs();

                for (agent_id, last_seen_secs) in agents {
                    if now.saturating_sub(last_seen_secs) > 300 {
                        debug!(
                            "Agent {} has been offline for >5m. Deactivating...",
                            agent_id
                        );
                        match repo.set_agent_deactivated(&agent_id).await {
                            Ok(()) => {}
                            Err(e) => warn!("Failed to deactivate agent {}: {}", agent_id, e),
                        }
                    }
                }

                tokio::time::sleep(Duration::from_secs(60)).await;
            }
        })
    }
}
