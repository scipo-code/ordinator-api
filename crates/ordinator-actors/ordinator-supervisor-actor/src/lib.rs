mod algorithm;
mod assert_functions;
pub mod messages;

use std::sync::RwLockReadGuard;

use algorithm::supervisor_parameters::SupervisorParameters;
use algorithm::supervisor_solution::SupervisorSolution;
#[allow(unused_imports)]
use assert_functions::SupervisorAssertions;
use ordinator_actor_core::algorithm::Algorithm;
use ordinator_configuration::SystemConfigurations;
use ordinator_orchestrator_actor_traits::SharedSolutionTrait;
use rand::SeedableRng;
use rand::rngs::StdRng;

pub struct SupervisorOptions
{
    pub number_of_unassigned_work_orders: usize,
    pub rng: StdRng,
}

pub struct SupervisorAlgorithm<Ss>(Algorithm<SupervisorSolution, SupervisorParameters, (), Ss>)
where
    Ss: SharedSolutionTrait;

impl<'a> From<RwLockReadGuard<'a, SystemConfigurations>> for SupervisorOptions
{
    fn from(value: RwLockReadGuard<SystemConfigurations>) -> Self
    {
        let number_of_unassigned_work_orders = value
            .actor_configurations
            .supervisor_options
            .number_of_removed_work_orders;
        SupervisorOptions {
            rng: StdRng::from_os_rng(),
            number_of_unassigned_work_orders,
        }
    }
}
