use serde::{Deserialize, Serialize};

use crate::models::period::PeriodNone;

<<<<<<< HEAD
#[derive(Clone)]
=======
use serde::{Deserialize, Serialize};

>>>>>>> origin
#[derive(Serialize, Deserialize)]
pub struct UnloadingPoint {
    pub string: String,
    pub present: bool,
    pub period: PeriodNone,
}

