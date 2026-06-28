use std::sync::Arc;

use crate::service::AgentService;

#[derive(Clone)]
pub(crate) struct AppState {
    pub agent_service: Arc<AgentService>,
}