use thiserror::Error;
#[derive(Debug, Error)]

pub enum AgentError {
    #[error("Request to update agent state failed {0}")]
    StateUpdateError(String),
}
