use std::sync::Arc;
use std::time::Duration;

use tokio::task::JoinHandle;
use core_watch::logging::{warn, debug};

use crate::domain::db::DbStore;

pub fn start_agent_monitor(db: Arc<DbStore>) -> JoinHandle<()> {
    tokio::spawn(async move {
        loop {
            let agents = match db.list_agents_with_last_seen().await {
                Ok(agents) => agents,
                Err(e) => {
                    warn!("Failed to list agents for monitor: {}", e);
                    tokio::time::sleep(Duration::from_secs(60)).await;
                    continue;
                }
            };
            
            let now = std::time::SystemTime::now()
                .duration_since(std::time::SystemTime::UNIX_EPOCH)
                .unwrap()
                .as_secs();
            
            for (agent_id, last_seen_secs) in agents {
                if now.saturating_sub(last_seen_secs) > 300 {
                    debug!("Agent {} has been offline for >5m. Deactivating...", agent_id);
                    match db.set_agent_deactivated(&agent_id).await {
                        Ok(()) => {},
                        Err(e) => warn!("Failed to deactivate agent {}: {}", agent_id, e),
                    }
                }
            }

            tokio::time::sleep(Duration::from_secs(60)).await;
        }
    })
}
