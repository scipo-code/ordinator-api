use std::collections::HashMap;

use serde::{Deserialize, Serialize};
use status_codes::{
    SystemStatusCodes, SystemStatusCodesBuilder, UserStatusCodes, UserStatusCodesBuilder,
};

use super::operation::{ActivityNumber, Operation};

pub mod status_codes;
#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct WorkOrderAnalytic {
    pub system_status_codes: SystemStatusCodes,
    pub user_status_codes: UserStatusCodes,
}

pub struct WorkOrderAnalyticBuilder {
    // TODO [x]
    // You should make a builder for these if needed
    system_status_codes: Option<SystemStatusCodes>,
    // TODO [x]
    // You should make a builder for these if needed
    user_status_codes: Option<UserStatusCodes>,
}

impl WorkOrderAnalyticBuilder {
    pub fn build(self) -> WorkOrderAnalytic {
        WorkOrderAnalytic {
            // FIX [ ]
            // This is wrong, you should design it so that the code
            // How should the fixed be calculated? I am not sure that it ever should.
            // Things that are fixed for one model may not be for all the other ones.
            // You should design the system so that these things are calculated and
            // should be part of the parameters of the applications.
            //
            system_status_codes: self
                .system_status_codes
                .expect("Check that all builder steps are followed"),
            user_status_codes: self
                .user_status_codes
                .expect("Check that the code is actually created correctly"),
        }
    }

    pub fn system_status_codes<F>(mut self, configure: F) -> Self
    where
        F: FnOnce(SystemStatusCodesBuilder) -> SystemStatusCodesBuilder,
    {
        let system_status_codes_builder = SystemStatusCodes::builder();

        let configured_systes_codes_builder = configure(system_status_codes_builder);

        self.system_status_codes = Some(configured_systes_codes_builder.build());
        self
    }

    pub fn user_status_codes<F>(mut self, configure: F) -> Self
    where
        F: FnOnce(UserStatusCodesBuilder) -> UserStatusCodesBuilder,
    {
        let user_status_codes_builder = UserStatusCodes::builder();

        let configured_user_status_codes_builder = configure(user_status_codes_builder);

        self.user_status_codes = Some(configured_user_status_codes_builder.build());
        self
    }
}

// You are doing something very shitty at the moment. Do you actually need any builders
// anymore when you are simply reading data from the... Yes you are doing it to ease testing
// that is the purpose.
impl WorkOrderAnalytic {
    //
    pub fn builder() -> WorkOrderAnalyticBuilder {
        // QUESTION
        // How should this be designed?
        let system_status_codes = Some(SystemStatusCodes::builder().build());
        let user_status_codes = Some(UserStatusCodes::builder().build());

        WorkOrderAnalyticBuilder {
            system_status_codes,
            user_status_codes,
        }
    }

    // TODO [ ]
    pub fn fixed(&self) -> bool {
        todo!("This could be a very important function to implement. Status codes should ideally express this together with material Statuses.")
    }
}
