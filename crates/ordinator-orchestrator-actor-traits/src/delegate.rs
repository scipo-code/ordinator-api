use ordinator_scheduling_environment::work_order::operation::Operation;

#[derive(Clone, Default, Hash, Eq, PartialEq, PartialOrd, Ord, Debug)]
pub enum Delegate {
    #[default]
    Assess,
    Assign,
    Unassign,
    Drop,
    Done,
    Fixed,
}

impl Delegate {
    pub fn build(operation: &Operation) -> Delegate {
        if operation.operation_info.work_remaining.0.is_zero() {
            return Delegate::Done;
        }
        Delegate::Assess
    }

    pub fn is_done(&self) -> bool {
        matches!(self, Self::Done)
    }

    pub fn is_assign(&self) -> bool {
        matches!(self, Self::Assign)
    }

    pub fn is_assess(&self) -> bool {
        matches!(self, Self::Assess)
    }

    pub fn is_drop(&self) -> bool {
        matches!(self, Self::Drop)
    }

    pub fn state_change_to_unassign(&mut self) {
        match self {
            Delegate::Assess => *self = Delegate::Unassign,
            Delegate::Assign => todo!(),
            Delegate::Unassign => todo!(),
            Delegate::Drop => todo!(),
            Delegate::Done => todo!(),
            Delegate::Fixed => todo!(),
        }
    }

    pub fn state_change_to_assign(&mut self) {
        match self {
            Delegate::Assess => *self = Delegate::Assign,
            Delegate::Assign => todo!(),
            Delegate::Unassign => todo!(),
            Delegate::Drop => todo!(),
            Delegate::Done => todo!(),
            Delegate::Fixed => todo!(),
        }
    }
}
