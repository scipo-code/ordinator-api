use anyhow::Result;
use anyhow::anyhow;
use serde::Deserialize;
use serde::Serialize;

#[derive(Serialize, Deserialize, Debug)]
pub struct Throttling
{
    strategic_throttling: u64,
    tactical_throttling: u64,
    supervisor_throttling: u64,
    operational_throttling: u64,
}

impl Throttling
{
    pub fn get_throttling(&self, actor: &str) -> Result<u64>
    {
        match actor {
            s if s.to_lowercase().starts_with("strategic") => Ok(self.strategic_throttling),
            s if s.to_lowercase().starts_with("tactical") => Ok(self.tactical_throttling),
            s if s.to_lowercase().starts_with("main") => Ok(self.supervisor_throttling),
            s if s.to_lowercase().starts_with("supervisor") => Ok(self.supervisor_throttling),
            s if s.to_lowercase().starts_with("operational") => Ok(self.operational_throttling),
            s if s.starts_with("OP") => Ok(self.operational_throttling),
            _ => Err(anyhow!("wrong key to access actor throttling logic")),
        }
    }
}
