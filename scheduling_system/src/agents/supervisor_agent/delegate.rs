use shared_types::scheduling_environment::work_order::WorkOrderActivity;

#[derive(Hash, Eq, PartialEq, PartialOrd, Ord, Debug)]
pub enum Delegate {
    Assess(WorkOrderActivity),
    Assign(WorkOrderActivity),
    Drop(WorkOrderActivity),
    Done(WorkOrderActivity),
    Fixed,
}
impl Delegate {
    pub fn new(work_order_activity: WorkOrderActivity) -> Delegate {
        Delegate::Assess(work_order_activity)
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
            Delegate::Assign(work_order_activity) => {
                panic!("The program is not ready to handle this yet");
                *self = Delegate::Drop(*work_order_activity);
            }
            Delegate::Assess(work_order_activity) => {
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
