mod algorithm;
pub mod messages;

use std::ops::Deref;
use std::ops::DerefMut;
use std::sync::Arc;

use algorithm::TacticalAlgorithm;
use algorithm::tactical_solution::TacticalSolution;
use arc_swap::Guard;
use messages::TacticalRequestMessage;
use messages::TacticalResponseMessage;
use ordinator_actor_core::Actor;
use ordinator_configuration::SystemConfigurations;
use ordinator_orchestrator_actor_traits::MessageHandler;
use ordinator_orchestrator_actor_traits::SharedSolutionTrait;
use ordinator_scheduling_environment::worker_environment::resources::Id;
use rand::SeedableRng;
use rand::rngs::StdRng;

//TODO [ ]
// Make `TacticalAlgorithm` but... Eat first.
pub struct TacticalActor<Ss>(
    Actor<TacticalRequestMessage, TacticalResponseMessage, TacticalAlgorithm<Ss>>,
)
where
    Ss: SharedSolutionTrait<Tactical = TacticalSolution>,
    Self: MessageHandler<Req = TacticalRequestMessage, Res = TacticalResponseMessage>;

pub struct TacticalOptions
{
    pub number_of_removed_work_orders: usize,
    pub urgency: usize,
    pub resource_penalty: usize,
    pub rng: StdRng,
}
impl From<(&Guard<Arc<SystemConfigurations>>, &Id)> for TacticalOptions
{
    fn from(value: (&Guard<Arc<SystemConfigurations>>, &Id)) -> Self
    {
        let tactical_options_config = &value
            .0
            .actor_specification
            .get(value.1.asset())
            .unwrap()
            .tactical
            .tactical_options_config;
        TacticalOptions {
            number_of_removed_work_orders: tactical_options_config.number_of_removed_work_orders,
            rng: StdRng::from_os_rng(),
            urgency: tactical_options_config.urgency,
            resource_penalty: tactical_options_config.resource_penalty,
        }
    }
}

impl<Ss> Deref for TacticalActor<Ss>
where
    Ss: SharedSolutionTrait<Tactical = TacticalSolution>,
{
    type Target = Actor<TacticalRequestMessage, TacticalResponseMessage, TacticalAlgorithm<Ss>>;

    fn deref(&self) -> &Self::Target
    {
        &self.0
    }
}

impl<Ss> DerefMut for TacticalActor<Ss>
where
    Ss: SharedSolutionTrait<Tactical = TacticalSolution>,
{
    fn deref_mut(&mut self) -> &mut Self::Target
    {
        &mut self.0
    }
}
