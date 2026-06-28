use crate::error::Result;
use async_trait::async_trait;
use crate::domain::{model::{Agent, InstallToken}};

#[async_trait]
pub(crate) trait AgentRepository: Send + Sync {
    async fn create_agent(&self, agent: &Agent) -> Result<()>;
    async fn get_agent(&self, agent_id: &str) -> Result<Option<Agent>>;
    async fn get_agent_by_hostname(&self, hostname: &str) -> Result<Option<Agent>>;
    async fn update_agent_last_seen(&self, agent_id: &str) -> Result<()>;
    async fn update_agent_ip(&self, agent_id: &str, ipv4: &str) -> Result<()>;
    #[allow(dead_code)]
    async fn revoke_agent(&self, agent_id: &str) -> Result<()>;
    #[allow(dead_code)]
    async fn delete_agent(&self, agent_id: &str) -> Result<()>;
    async fn set_agent_deactivated(&self, agent_id: &str) -> Result<()>;
    async fn list_agents(&self) -> Result<Vec<Agent>>;
    async fn list_agents_with_last_seen(&self) -> Result<Vec<(String, u64)>>;
    async fn create_install_token(&self, token: &InstallToken) -> Result<()>;
    async fn validate_install_token(&self, plaintext: &str) -> Result<bool>;
}
