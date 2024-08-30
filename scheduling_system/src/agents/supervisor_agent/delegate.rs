use std::sync::{Arc, RwLock};

use actix::Message;
use shared_types::scheduling_environment::{work_order::WorkOrderActivity, worker_environment::resources::Id};

use crate::agents::tactical_agent::tactical_algorithm::TacticalOperation;

#[derive(Hash, Eq, PartialEq, PartialOrd, Ord, Debug)]
pub enum Delegate {
    Assess((WorkOrderActivity, Arc<TacticalOperation>)),
    Assign((WorkOrderActivity, Arc<TacticalOperation>)),
    Drop(WorkOrderActivity),
    Done(WorkOrderActivity),
    Fixed,
}
impl Delegate {
    pub fn new(work_order_activity: WorkOrderActivity, tactical_operation: Arc<TacticalOperation>) -> Delegate {
        Delegate::Assess((work_order_activity, tactical_operation))
    }

    pub fn tactical_operation(&self) -> Arc<TacticalOperation> {
        match self {
            Delegate::Assess((_, tactical_operation)) => tactical_operation.clone(),
            Delegate::Assign((_, tactical_operation)) => tactical_operation.clone(),
            Delegate::Drop(_) => panic!(),
            Delegate::Done(_) => {
                panic!("The Operation is done. There should be no applicable business logic.")
            }
            Delegate::Fixed => panic!(),
        }
    }

    pub fn is_assess(&self) -> bool {
        matches!(self, Self::Assess(_))
    }

    pub fn is_done(&self) -> bool {
        matches!(self, Self::Done(_))
    }

    pub fn is_assign(&self) -> bool {
        matches!(self, Self::Assign(_))
    }

    pub fn is_drop(&self) -> bool {
        matches!(self, Self::Drop(_))
    }

    pub fn is_fixed(&self) -> bool {
        matches!(self, Self::Fixed)
    }

    pub fn state_change_to_drop(&mut self) {
        match self {
            Delegate::Assign((work_order_activity, _)) => {
                panic!("The program is not ready to handle this yet");
                *self = Delegate::Drop(*work_order_activity);
            }
            Delegate::Assess((work_order_activity, _)) => {
                let delegate = Delegate::Drop(*work_order_activity);
                *self = delegate;
            }
            Delegate::Done(work_order_activity) => {
                *self = Delegate::Drop(*work_order_activity)
            }
            _ => panic!("Only Delegate::Assess and Delegate::Assign and Delegate::Drop can be converted to a Delegate::Drop")
        }
    }
}

#[derive(Debug)]
pub struct DelegateAndId(pub Arc<RwLock<Delegate>>, pub Id);

impl Message for DelegateAndId {
    type Result = ();
}
