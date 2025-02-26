use std::collections::HashMap;

use serde::{Deserialize, Serialize};
use status_codes::{SystemStatusCodes, UserStatusCodes};

use super::operation::{ActivityNumber, Operation};

pub mod status_codes;
#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct WorkOrderAnalytic {
    pub system_status_codes: SystemStatusCodes,
    pub user_status_codes: UserStatusCodes,
}

pub struct WorkOrderAnalyticBuilder {
    // TODO [ ]
    // You should make a builder for these if needed
    pub system_status_codes: Option<SystemStatusCodes>,
    // TODO [ ]
    // You should make a builder for these if needed
    pub user_status_codes: Option<UserStatusCodes>,
}

impl WorkOrderAnalyticBuilder {
    pub fn build(self, operations: HashMap<ActivityNumber, Operation>) -> WorkOrderAnalytic {
        WorkOrderAnalytic {
            // QUESTION
            // How should the fixed be calculated? I am not sure that it ever should.
            // Things that are fixed for one model may not be for all the other ones.
            // You should design the system so that these things are calculated and
            // should be part of the parameters of the applications.
            system_status_codes: todo!(),
            user_status_codes: todo!(),
        }
    }

    pub fn system_status_codes<F>(&mut self, f: F) -> &mut Self
    where
        F: FnOnce(&mut SystemStatusCodes) -> &mut SystemStatusCodes,
    {
        let mut system_status_codes_builder = SystemStatusCodes::builder();

        f(&mut system_status_codes_builder);
        self.system_status_codes = system_status_codes_builder.build();
        self
    }

    pub fn user_status_codes<F>(&mut self, f: F) -> &mut Self
    where
        F: FnOnce(&mut UserStatusCodes) -> &mut UserStatusCodes,
    {
        let mut user_status_codes_builder = UserStatusCodes::builder();

        f(&mut user_status_codes_builder);
        self.user_status_codes = user_status_codes_builder.build();
        self
    }

    pub fn from_data()
}

// You are doing something very shitty at the moment. Do you actually need any builders
// anymore when you are simply reading data from the... Yes you are doing it to ease testing
// that is the purpose.
impl WorkOrderAnalytic {
    // 
    pub fn builder() -> WorkOrderAnalyticBuilder {
        WorkOrderAnalyticBuilder {
            system_status_codes: ,
            user_status_codes: (),
        }
    }

    // TODO [ ]
    pub fn fixed(&self) -> bool {
        todo!("This could be a very important function to implement. Status codes should ideally express this together with material Statuses.")
    }
}
