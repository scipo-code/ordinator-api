use std::sync::atomic::Ordering;

use atomic_enum::atomic_enum;

#[derive(Hash, Eq, PartialEq, PartialOrd, Ord)]
#[atomic_enum]
pub enum Delegate {
    Assess,
    Assign,
    Unassign,
    Drop,
    Done,
    Fixed,
}

impl AtomicDelegate {
    pub fn state_change_to_drop(&self) {
        let mut delegate_state = self.load(Ordering::SeqCst);

        delegate_state.state_change_to_drop();

        self.store(delegate_state, Ordering::SeqCst);
    }

    pub fn state_change_to_unassign(&self) {
        let mut delegate_state = self.load(Ordering::SeqCst);

        delegate_state.state_change_to_unassign();

        self.store(delegate_state, Ordering::SeqCst);
    }

    pub fn state_change_to_assign(&self) {
        let mut delegate_state = self.load(Ordering::SeqCst);

        delegate_state.state_change_to_assign();

        self.store(delegate_state, Ordering::SeqCst);
    }
}

impl Delegate {
    pub fn new() -> Delegate {
        Delegate::Assess
    }

    pub fn is_assess(&self) -> bool {
        matches!(self, Self::Assess)
    }

    pub fn is_done(&self) -> bool {
        matches!(self, Self::Done)
    }

    pub fn is_assign(&self) -> bool {
        matches!(self, Self::Assign)
    }

    pub fn is_drop(&self) -> bool {
        matches!(self, Self::Drop)
    }

    pub fn state_change_to_drop(&mut self) {
        match self {
            Delegate::Assign => {
                *self = Delegate::Drop;
            }
            Delegate::Assess => {
                let delegate = Delegate::Drop;
                *self = delegate;
            }
            Delegate::Done => {
                // Specify specific logic
                *self = Delegate::Drop
            }
            Delegate::Unassign => {
                // Specify specific logic
                *self = Delegate::Drop
            }
            val => {
                panic!("Only Delegate::Assess and Delegate::Assign and Delegate::Drop can be converted to a Delegate::Drop. Got {:?} ", val);
            }
        }
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
