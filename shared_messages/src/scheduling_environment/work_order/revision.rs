use serde::{Deserialize, Serialize};
#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct Revision {
    pub string: String,
    pub shutdown: bool,
}

impl Revision {
    pub fn new(string: String) -> Self {
        Revision {
            string: string.clone(),
            shutdown: !string.contains("NOSD"),
        }
    }

    pub fn new_with_shutdown(string: String, shutdown: bool) -> Self {
        Revision { string, shutdown }
    }
}

impl Default for Revision {
    fn default() -> Self {
        Revision {
            string: String::from(""),
            shutdown: false,
        }
    }
}
