use std::{
    sync::Arc,
    time::Duration,
};

use dashmap::DashMap;
use tokio::task::JoinHandle;
use core_watch::logging::warn;

use crate::model::state::AgentState;

pub fn start_agent_monitor(agents: Arc<DashMap<String, AgentState>>) -> JoinHandle<()> {
    tokio::spawn(async move {
        loop {
            for entry in agents.iter() {
                if entry.last_seen.elapsed() > Duration::from_secs(300) {
                    warn!(
                        "Node {} has been offline for >5m",
                        entry.key()
                    );
                }
            }

            tokio::time::sleep(Duration::from_secs(60)).await;
        }
    })
}