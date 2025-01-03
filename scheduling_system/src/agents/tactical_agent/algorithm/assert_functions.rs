use anyhow::{bail, Result};
use colored::Colorize;
use std::{borrow::Cow, collections::HashMap};
use strum::IntoEnumIterator;
use tracing::{event, Level};

use shared_types::scheduling_environment::{
    time_environment::day::Day, work_order::operation::Work,
    worker_environment::resources::Resources,
};

use super::TacticalAlgorithm;

type TotalExcessHours = Work;

#[allow(dead_code)]
pub trait TacticalAssertions {
    fn asset_that_loading_matches_scheduled(&self) -> Result<()>;

    fn asset_that_capacity_is_not_exceeded(&self) -> Result<TotalExcessHours>;
}

impl TacticalAssertions for TacticalAlgorithm {
    fn asset_that_loading_matches_scheduled(&self) -> Result<()> {
        let mut aggregated_load: HashMap<Resources, HashMap<Day, Work>> = HashMap::new();

        for (_work_order_number, tactical_solution) in &self
            .tactical_solution
            .tactical_scheduled_work_orders
            .0
            .iter()
            .filter(|(_, whe_tac_sch)| whe_tac_sch.is_tactical())
            .collect::<Vec<_>>()
        {
            for operation_solution in tactical_solution.tactical_operations()?.0.values() {
                let resource = &operation_solution.resource;

                for (day, load) in &operation_solution.scheduled {
                    *aggregated_load
                        .entry(resource.clone())
                        .or_default()
                        .entry(day.clone())
                        .or_insert(Work::from(0.0)) += load;
                }
            }
        }

        for resource in Resources::iter() {
            for day in &self.tactical_days {
                let resource_map = match aggregated_load.get(&resource) {
                    Some(map) => Cow::Borrowed(map),
                    None => Cow::Owned(HashMap::new()),
                };

                let zero_work = Work::from(0.0);
                let agg_load = resource_map.get(day).unwrap_or(&zero_work);
                let sch_load = self
                    .tactical_solution
                    .tactical_loadings
                    .get_resource(&resource, day);

                if (agg_load - sch_load).0.round_dp(9) != Work::from(0.0).0 {
                    event!(Level::ERROR, agg_load = ?agg_load, sch_load = ?sch_load, resource = ?resource, day = ?day);
                    bail!("Loads does not match on: day {}\n\tresource: {}\n\tscheduled load: {}\n\taggregated_load: {}\n", day.to_string().bright_green(), resource.to_string().bright_blue(), sch_load.to_string().bright_yellow(), agg_load.to_string().bright_yellow());
                }
            }
        }

        Ok(())
    }

    fn asset_that_capacity_is_not_exceeded(&self) -> Result<TotalExcessHours> {
        let mut total_excess_hours = Work::from(0.0);
        for (resource, days) in &self.tactical_solution.tactical_loadings.resources {
            for (day, load) in &days.days {
                let capacity = self
                    .tactical_parameters
                    .tactical_capacity
                    .get_resource(resource, day);

                total_excess_hours += (load - capacity).max(Work::from(0.0));
                // ensure!(
                //     load <= capacity,
                //     format!(
                //         "Load exceeds Capacity for resource: {:?} on day: {:?} with load {:?} and capacity {:?}",
                //         resource, day, load, capacity
                //     )
                // );
            }
        }
        Ok(total_excess_hours)
    }
}

//         algorithm_state
//             .infeasible_cases_mut()
//             .unwrap()
//             .earliest_start_day = (|| {
//             for (work_order_number, optimized_work_order) in self.tactical_parameters().clone() {
//                 let start_date_from_period = match optimized_work_order.scheduled_period {
//                     Some(period) => period.start_date().date_naive(),
//                     None => optimized_work_order.earliest_allowed_start_date,
//                 };

//                 if let Some(operation_solutions) = optimized_work_order.operation_solutions {
//                     let start_days: Vec<_> =
//                         operation_solutions
//                             .values()
//                             .map(|operation_solution| {
//                                 operation_solution
//                             .scheduled
//                             .first()
//                             .expect("All scheduled operations should have a first scheduled day")
//                             .0
//                             .clone()
//                             })
//                             .collect();

//                     for start_day in start_days {
//                         if start_day.date().date_naive() < start_date_from_period {
//                             error!(start_day = ?start_day.date(), start_date_from_period = ?start_date_from_period);
//                             return ConstraintState::Infeasible(format!(
//                                 "{:?} is outside of its earliest start day: {}",
//                                 work_order_number, start_date_from_period
//                             ));
//                         }
//                     }
//                 }
//             }
//             ConstraintState::Feasible
//         })();

//         algorithm_state
//             .infeasible_cases_mut()
//             .unwrap()
//             .all_scheduled = (|| {
//             for (work_order_number, optimized_work_order) in self.tactical_parameters().clone() {
//                 if optimized_work_order.operation_solutions.is_none() {
//                     return ConstraintState::Infeasible(work_order_number.0.to_string());
//                 }
//             }
//             ConstraintState::Feasible
//         })();

//         algorithm_state
//             .infeasible_cases_mut()
//             .unwrap()
//             .respect_period_id = (|| {
//             for (_work_order_number, optimized_work_order) in self.tactical_parameters().clone() {
//                 let scheduled_period = match optimized_work_order.scheduled_period {
//                     Some(period) => period,
//                     None => return ConstraintState::Feasible,
//                 };
//                 if !self.tactical_periods.contains(&scheduled_period) {
//                     error!(work_order_number = ?_work_order_number, scheduled_period = ?scheduled_period, tactical_periods = ?self.tactical_periods, "Tactical period does not contain the scheduled period of the tactical work order");
//                     return ConstraintState::Infeasible(format!(
//                         "{:?} has a wrong scheduled period {}",
//                         _work_order_number, scheduled_period
//                     ));
//                 }
//             }
//             ConstraintState::Feasible
//         })();

//         let infeasible_cases = algorithm_state.infeasible_cases_mut().unwrap();

//         if infeasible_cases.aggregated_load == ConstraintState::Feasible
//             && infeasible_cases.earliest_start_day == ConstraintState::Feasible
//             && infeasible_cases.all_scheduled == ConstraintState::Feasible
//             && infeasible_cases.respect_period_id == ConstraintState::Feasible
//         {
//             AlgorithmState::Feasible
//         } else {
//             algorithm_state
//         }
//     }
// }
