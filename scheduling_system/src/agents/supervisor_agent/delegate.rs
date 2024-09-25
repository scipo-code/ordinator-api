use std::sync::atomic::Ordering;

use atomic_enum::atomic_enum;

#[derive(Hash, Eq, PartialEq, PartialOrd, Ord)]
#[atomic_enum]
pub enum Delegate {
    Assess,
    Assign,
    Drop,
    Done,
    Fixed,
}

impl AtomicDelegate {
    pub fn state_change_to_drop(&self) {
        let mut delegate_state = self.load(Ordering::Acquire);

        delegate_state.state_change_to_drop();

        self.store(delegate_state, Ordering::Release);
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
                panic!("The program is not ready to handle this yet");
                // *self = Delegate::Drop;
            }
            Delegate::Assess => {
                let delegate = Delegate::Drop;
                *self = delegate;
            }
            Delegate::Done => {
                *self = Delegate::Drop
            }
            _ => panic!("Only Delegate::Assess and Delegate::Assign and Delegate::Drop can be converted to a Delegate::Drop")
        }
    }
}
