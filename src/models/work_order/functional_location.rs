
<<<<<<< HEAD
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
#[derive(Clone)]
=======
use chrono::{DateTime, Utc, Duration};
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
>>>>>>> origin
pub struct FunctionalLocation {
    pub string: String
}
