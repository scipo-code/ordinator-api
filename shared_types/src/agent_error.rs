use serde::Serialize;
use thiserror::Error;

#[derive(Debug, Error, Serialize)]
pub enum AgentError {
    #[error("Request to update agent state failed {0}")]
    StateUpdateError(String),
}
