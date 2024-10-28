use super::TacticalAgent;

trait TacticalAssertions {}

impl TacticalAssertions for TacticalAgent {}
// impl TestAlgorithm for TacticalAlgorithm {
//     type InfeasibleCases = TacticalInfeasibleCases;

//     #[instrument(level = "info", skip(self))]
//     fn determine_algorithm_state(&self) -> AlgorithmState<Self::InfeasibleCases> {
//         let mut algorithm_state = AlgorithmState::Infeasible(TacticalInfeasibleCases::default());

//         let mut aggregated_load: HashMap<Resources, HashMap<Day, Work>> = HashMap::new();
//         for (_work_order_number, optimized_work_order) in self.tactical_parameters().clone() {
//             for (_activity, operation_solution) in
//                 optimized_work_order.operation_solutions.unwrap_or_default()
//             {
//                 let resource = operation_solution.resource;

//                 for (day, load) in operation_solution.scheduled {
//                     *aggregated_load
//                         .entry(resource.clone())
//                         .or_default()
//                         .entry(day)
//                         .or_insert(Work::from(0.0)) += load;
//                 }
//             }
//         }

//         algorithm_state
//             .infeasible_cases_mut()
//             .unwrap()
//             .aggregated_load = (|| {
//             for resource in Resources::iter() {
//                 for day in &self.tactical_days {
//                     let resource_map = match aggregated_load.get(&resource) {
//                         Some(map) => Cow::Borrowed(map),
//                         None => Cow::Owned(HashMap::new()),
//                     };

//                     let zero_work = Work::from(0.0);
//                     let agg_load = resource_map.get(day).unwrap_or(&zero_work);
//                     let sch_load = self.loading(&resource, day);
//                     if (agg_load - sch_load) > Work::from(0.0)
//                         || (agg_load - sch_load) < Work::from(0.0)
//                     {
//                         error!(agg_load = ?agg_load, sch_load = ?sch_load, resource = ?resource, day = ?day);
//                         return ConstraintState::Infeasible(format!("Loads does not match on: day {}\nresource: {}\nscheduled load: {}\naggregated_load: {}", day, resource, sch_load, agg_load));
//                     }
//                 }
//             }
//             ConstraintState::Feasible
//         })();

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
