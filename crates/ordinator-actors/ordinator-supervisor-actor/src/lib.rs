mod algorithm;
mod assert_functions;
pub mod messages;

use std::ops::Deref;
use std::ops::DerefMut;
use std::sync::Arc;

use algorithm::SupervisorAlgorithm;
use algorithm::supervisor_solution::SupervisorSolution;
use arc_swap::Guard;
#[allow(unused_imports)]
use assert_functions::SupervisorAssertions;
use messages::SupervisorRequestMessage;
use messages::SupervisorResponseMessage;
use ordinator_actor_core::Actor;
use ordinator_configuration::SystemConfigurations;
use ordinator_orchestrator_actor_traits::MessageHandler;
use ordinator_orchestrator_actor_traits::SharedSolutionTrait;
use ordinator_scheduling_environment::worker_environment::resources::Id;
use rand::SeedableRng;
use rand::rngs::StdRng;

pub struct SupervisorActor<Ss>(
    Actor<SupervisorRequestMessage, SupervisorResponseMessage, SupervisorAlgorithm<Ss>>,
)
where
    Ss: SharedSolutionTrait<Supervisor = SupervisorSolution>,
    Self: MessageHandler<Req = SupervisorRequestMessage, Res = SupervisorResponseMessage>;

impl<Ss> Deref for SupervisorActor<Ss>
where
    Ss: SharedSolutionTrait<Supervisor = SupervisorSolution>,
{
    type Target =
        Actor<SupervisorRequestMessage, SupervisorResponseMessage, SupervisorAlgorithm<Ss>>;

    fn deref(&self) -> &Self::Target
    {
        &self.0
    }
}

impl<Ss> DerefMut for SupervisorActor<Ss>
where
    Ss: SharedSolutionTrait<Supervisor = SupervisorSolution>,
{
    fn deref_mut(&mut self) -> &mut Self::Target
    {
        &mut self.0
    }
}
pub struct SupervisorOptions
{
    pub number_of_unassigned_work_orders: usize,
    pub rng: StdRng,
}

impl<'a> From<(&Guard<Arc<SystemConfigurations>>, &Id)> for SupervisorOptions
{
    fn from(value: (&Guard<Arc<SystemConfigurations>>, &Id)) -> Self
    {
        let number_of_unassigned_work_orders = value
            .0
            .actor_specification
            .get(value.1.asset())
            .unwrap()
            .supervisors
            .iter()
            .find(|s| s.id == value.1.0)
            .unwrap()
            .supervisor_options
            .number_of_removed_work_orders;
        SupervisorOptions {
            rng: StdRng::from_os_rng(),
            number_of_unassigned_work_orders,
        }
    }
}
