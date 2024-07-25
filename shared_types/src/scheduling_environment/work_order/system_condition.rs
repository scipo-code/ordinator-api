use serde::{Deserialize, Serialize};

#[derive(Default, Serialize, Deserialize, Debug, Clone)]
pub enum SystemCondition {
    A,
    B,
    C,
    D,
    E,
    F,
    G,
    #[default]
    Unknown,
}
