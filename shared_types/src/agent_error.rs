#[derive(Debug)]
pub enum AgentError {
    StateUpdateError(String),
}

impl std::fmt::Display for AgentError {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match *self {
            AgentError::StateUpdateError(ref err) => write!(f, "State update error: {}", err),
        }
    }
}

impl std::error::Error for AgentError {}
