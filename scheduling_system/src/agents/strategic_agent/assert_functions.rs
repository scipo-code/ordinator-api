use std::collections::HashMap;

use super::StrategicAgent;
use anyhow::{bail, Result};
use shared_types::scheduling_environment::work_order::operation::Work;
use shared_types::scheduling_environment::worker_environment::resources::Resources;
use shared_types::strategic::StrategicResources;
use shared_types::LoadOperation;
use strum::IntoEnumIterator;
use tracing::{event, Level};

#[allow(dead_code)]
pub trait StrategicAssertions {
    fn assert_aggregated_load(&self) -> Result<()>;
}

impl StrategicAssertions for StrategicAgent {
    fn assert_aggregated_load(&self) -> Result<()> {
        let mut aggregated_strategic_load = StrategicResources::new(HashMap::new());
        for period in self.strategic_algorithm.periods() {
            for (work_order_number, strategic_solution) in self
                .strategic_algorithm
                .strategic_solution
                .strategic_periods
                .iter()
            {
                let strategic_parameter = self
                    .strategic_algorithm
                    .strategic_parameters
                    .strategic_work_order_parameters
                    .get(work_order_number)
                    .unwrap();
                if strategic_solution.as_ref().unwrap() == &period.clone() {
                    let work_load = &strategic_parameter.work_load;
                    for resource in Resources::iter() {
                        let load: Work =
                            work_load.get(&resource).cloned().unwrap_or(Work::from(0.0));
                        aggregated_strategic_load.update_load(
                            &resource,
                            period,
                            load,
                            LoadOperation::Add,
                        );
                    }
                }
            }
        }

        for (resource, periods) in aggregated_strategic_load.inner {
            for (period, load) in periods.0 {
                match self
                    .strategic_algorithm
                    .resources_loadings()
                    .inner
                    .get(&resource)
                    .unwrap()
                    .0
                    .get(&period)
                {
                    // Some(resource_load) if (*resource_load - load).abs() < 0.005 => continue,
                    Some(resource_load) => {
                        if resource_load.0.round_dp(6) != load.0.round_dp(6) {
                            event!(Level::ERROR, resource = %resource, period = %period, aggregated_load = %load, resource_load = %resource_load);
                            bail!("aggregated load and loading are not the same");
                        }
                    }
                    None => {
                        bail!("aggregated load and resource loading are not identically shaped")
                    }
                }
            }
        }
        Ok(())
    }
}

// impl TestAlgorithm for StrategicAgent {
//     type InfeasibleCases = StrategicInfeasibleCases;

//     fn determine_algorithm_state(&self) -> AlgorithmState<Self::InfeasibleCases> {
//         let scheduling_environment = self.scheduling_environment.lock().unwrap();

//         let mut strategic_state = AlgorithmState::Infeasible(Self::InfeasibleCases::default());

//         for (work_order_number, scheduled_period) in self
//             .strategic_agent_algorithm
//             .strategic_tactical_solution_arc_swap
//             .0
//             .load()
//             .strategic
//             .scheduled_periods
//         {
//             let scheduled_period = scheduled_period.clone();
//             let work_order = scheduling_environment
//                 .work_orders()
//                 .inner
//                 .get(&work_order_number)
//                 .unwrap();

//             let first_period = self.strategic_agent_algorithm.periods().first().unwrap();

//             let basic_start_of_first_activity = work_order.order_dates().basic_start_date;

//             let awsc = work_order.work_order_analytic.user_status_codes.awsc;

//             match scheduled_period {
//                 Some(scheduled_period) => {
//                     if awsc
//                         && !(scheduled_period.contains_date(basic_start_of_first_activity)
//                             || work_order.unloading_point_contains_period(scheduled_period.clone()))
//                         && &basic_start_of_first_activity > &first_period.start_date().date_naive()
//                     {
//                         strategic_state.infeasible_cases_mut().unwrap().respect_awsc =
//                             ConstraintState::Infeasible(format!(
//                                 "Work order {:?} does not respect AWSC. Period: {}, basic start date: {}, status codes: {:?}, unloading_point: {:?}, vendor: {}",
//                                 work_order_number,
//                                 scheduled_period,
//                                 basic_start_of_first_activity,
//                                 work_order.work_order_analytic.user_status_codes,
//                                 work_order.operations.values().map(|opr| opr.unloading_point.period.clone()),
//                                 if work_order.is_vendor() { "VEN" } else { "   " },
//                             ));
//                         break;
//                     }
//                 }
//                 None => {
//                     strategic_state.infeasible_cases_mut().unwrap().respect_awsc =
//                         ConstraintState::Infeasible(format!(
//                             "Work order {:?} does not have a period",
//                             work_order_number,
//                         ));
//                     break;
//                 }
//             }
//             strategic_state.infeasible_cases_mut().unwrap().respect_awsc =
//                 ConstraintState::Feasible;
//         }

//         for (work_order_number, optimized_work_order) in
//             self.strategic_agent_algorithm.optimized_work_orders()
//         {
//             let work_order = scheduling_environment
//                 .work_orders()
//                 .inner
//                 .get(work_order_number)
//                 .unwrap();

//             let periods = scheduling_environment.periods();

//             if work_order.unloading_point().is_some()
//                 && work_order.unloading_point() != optimized_work_order.scheduled_period
//                 && !periods[0..=1].contains(work_order.unloading_point().as_ref().unwrap())
//                 && !work_order.work_order_analytic.user_status_codes.awsc
//                 && !work_order.work_order_analytic.user_status_codes.sch
//             {
//                 error!(
//                     work_order_number = ?work_order_number,
//                     work_order_unloading_point = ?work_order.unloading_point(),
//                     work_order_status_codes = ?work_order.work_order_analytic.user_status_codes,
//                     work_order_dates = ?work_order.order_dates().basic_start_date,
//                     periods = ?periods[0..=1],
//                     optimized_work_order_scheduled_period = ?optimized_work_order.scheduled_period,
//                     optimized_work_order_locked_in_period = ?optimized_work_order.locked_in_period,
//                 );
//                 strategic_state
//                     .infeasible_cases_mut()
//                     .unwrap()
//                     .respect_unloading = ConstraintState::Infeasible(format!(
//                     "\t\t\nWork order number: {:?}\t\t\nwith unloading period: {}\t\t\nwith scheduled period: {}\t\t\nwith locked period: {}",
//                     work_order_number,
//                     work_order.unloading_point().as_ref().unwrap(),
//                     optimized_work_order.scheduled_period.clone().unwrap(),
//                     optimized_work_order.locked_in_period.clone().unwrap(),
//                 ));
//                 break;
//             }
//             strategic_state
//                 .infeasible_cases_mut()
//                 .unwrap()
//                 .respect_unloading = ConstraintState::Feasible;
//         }

//         for (work_order_number, scheduled_period) in self
//             .strategic_agent_algorithm
//             .strategic_tactical_solution_arc_swap
//             .0
//             .load()
//             .strategic
//             .scheduled_periods
//         {
//             let work_order = scheduling_environment
//                 .work_orders()
//                 .inner
//                 .get(&work_order_number)
//                 .unwrap();

//             let periods = scheduling_environment.periods();

//             if work_order.work_order_analytic.user_status_codes.sch
//                 && work_order.unloading_point().is_some()
//                 && periods[0..=1].contains(work_order.unloading_point().as_ref().unwrap())
//                 && scheduled_period != work_order.unloading_point()
//             {
//                 error!(
//                     work_order_number = ?work_order_number,
//                     work_order_unloading_point = ?work_order.unloading_point(),
//                     work_order_status_codes = ?work_order.work_order_analytic.user_status_codes,
//                     work_order_dates = ?work_order.order_dates().basic_start_date,
//                     periods = ?periods[0..=1],
//                     optimized_work_order_scheduled_period = ?scheduled_period,
//                     optimized_work_order_locked_in_period = ?self.locked_in_period,
//                 );
//                 strategic_state
//                     .infeasible_cases_mut()
//                     .unwrap()
//                     .respect_sch = ConstraintState::Infeasible(format!(
//                     "\t\t\nWork order number: {:?}\t\t\nwith scheduled period: {}\t\t\nwith locked period: {:?}\t\t\n work order status codes: {:?}\t\t\n work order unloading point: {:?}",
//                     work_order_number,
//                     scheduled_period.scheduled_period.as_ref().unwrap(),
//                     scheduled_period.locked_in_period.as_ref(),
//                     work_order.work_order_analytic.user_status_codes,
//                     work_order.unloading_point().as_ref(),
//                 ));
//                 break;
//             }
//             strategic_state.infeasible_cases_mut().unwrap().respect_sch = ConstraintState::Feasible;
//         }

//         strategic_state
//     }
// }
