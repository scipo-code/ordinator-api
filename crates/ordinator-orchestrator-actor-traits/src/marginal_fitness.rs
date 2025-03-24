use std::write;

use anyhow::Result;
use ordinator_scheduling_environment::work_order::WorkOrderActivity;
use ordinator_scheduling_environment::worker_environment::resources::Id;

#[derive(Eq, PartialEq, PartialOrd, Ord, Clone, Default)]
pub enum MarginalFitness
{
    Scheduled(u64),
    #[default]
    None,
}
// WARN
// More complex logic will be needed here for later. Start with this kind
// of implementation and then continue to make the most of it. I think
// that it is a better choice to quickly make this interface and then
// change afterwards.
//
// This means that this should not have a `new` function, but instead
//

/// You should most likely remove this and insert something else instead. I
/// think

impl std::fmt::Debug for MarginalFitness
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result
    {
        match self {
            MarginalFitness::Scheduled(time) => write!(
                f,
                "{}::{:?}({}, {:?}, {:?})",
                std::any::type_name::<MarginalFitness>()
                    .split("::")
                    .last()
                    .unwrap(),
                "Some",
                time,
                time / 3600,
                time / 3600 / 24,
            ),
            MarginalFitness::None => write!(f, "MarginalFitness::None"),
        }
    }
}
