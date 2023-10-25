use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

use crate::models::period::Period;

#[derive(Serialize, Deserialize)]
#[derive(Clone)]
pub struct OptimizedWorkOrder {
    period: Option<Period>,
    start_date: Option<DateTime<Utc>>,
    // TODO add operation data as well.
}


impl OptimizedWorkOrder {
    pub fn new(period: Period, start_date: DateTime<Utc>) -> OptimizedWorkOrder {
        OptimizedWorkOrder { 
            period: Some(period), 
            start_date: Some(start_date)
         }
    }

    pub fn empty() -> Self {
        OptimizedWorkOrder { 
            period: None, 
            start_date: None 
        }
    }
}