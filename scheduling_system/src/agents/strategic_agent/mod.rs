pub mod display;
pub mod strategic_algorithm;

use crate::agents::strategic_agent::strategic_algorithm::StrategicAlgorithm;
use crate::agents::traits::LargeNeighborHoodSearch;
use crate::models::SchedulingEnvironment;

use actix::prelude::*;
use shared_messages::agent_error::AgentError;
use shared_messages::resources::Resources;
use shared_messages::strategic::strategic_request_status_message::StrategicStatusMessage;
use shared_messages::strategic::strategic_response_status;
use shared_messages::strategic::strategic_response_status::StrategicResponseStatus;
use shared_messages::strategic::StrategicRequestMessage;
use shared_messages::Asset;
use shared_messages::SolutionExportMessage;
use shared_messages::StatusMessage;
use strum::IntoEnumIterator;
use tracing::info;

use std::collections::HashMap;
use std::sync::Arc;
use std::sync::Mutex;
use tracing::error;
use tracing::instrument;
use tracing::warn;

use crate::agents::tactical_agent::TacticalAgent;

use self::strategic_algorithm::optimized_work_orders::StrategicResources;

use super::traits::AlgorithmState;
use super::traits::ConstraintState;
use super::traits::TestAlgorithm;
use super::LoadOperation;
use super::SetAddr;
use super::StateLink;

pub struct StrategicAgent {
    asset: Asset,
    scheduling_environment: Arc<Mutex<SchedulingEnvironment>>,
    strategic_agent_algorithm: StrategicAlgorithm,
    tactical_agent_addr: Option<Addr<TacticalAgent>>,
}

impl Actor for StrategicAgent {
    type Context = Context<Self>;

    fn started(&mut self, ctx: &mut Context<Self>) {
        self.strategic_agent_algorithm.populate_priority_queues();
        info!("StrategicAgent has started for asset: {}", self.asset);
        ctx.notify(ScheduleIteration {})
    }

    fn stopped(&mut self, _ctx: &mut Context<Self>) {
        println!("SchedulerAgent is stopped");
    }
}

impl StrategicAgent {
    pub fn new(
        asset: Asset,
        scheduling_environment: Arc<Mutex<SchedulingEnvironment>>,
        strategic_agent_algorithm: StrategicAlgorithm,
        tactical_agent_addr: Option<Addr<TacticalAgent>>,
    ) -> Self {
        Self {
            asset,
            scheduling_environment,
            strategic_agent_algorithm,
            tactical_agent_addr,
        }
    }

    pub fn update_tactical_agent(&self) {
        let locked_scheduling_environment = self.scheduling_environment.lock().unwrap();

        let tactical_periods = locked_scheduling_environment.tactical_periods();
        let tactical_work_orders = self
            .strategic_agent_algorithm
            .tactical_work_orders(tactical_periods.to_vec());

        match &self.tactical_agent_addr {
            Some(tactical_agent_addr) => {
                tactical_agent_addr.do_send(StateLink::Strategic(tactical_work_orders));
            }
            None => {
                error!(
                    "The StrategicAgent cannot update the TacticalAgent as its address is not set"
                );
            }
        }
    }
}

#[derive(Message)]
#[rtype(result = "()")]
pub struct ScheduleIteration {}

impl Handler<ScheduleIteration> for StrategicAgent {
    type Result = ();

    #[instrument(level = "trace", skip_all)]
    fn handle(&mut self, _msg: ScheduleIteration, ctx: &mut Self::Context) -> Self::Result {
        self.strategic_agent_algorithm.schedule_forced_work_orders();

        let rng: &mut rand::rngs::ThreadRng = &mut rand::thread_rng();

        let mut temporary_schedule = self.strategic_agent_algorithm.clone();

        temporary_schedule.unschedule_random_work_orders(50, rng);

        temporary_schedule.schedule();

        temporary_schedule.calculate_objective_value();

        if temporary_schedule.objective_value() < self.strategic_agent_algorithm.objective_value() {
            self.strategic_agent_algorithm = temporary_schedule;

            info!(strategic_objective_value = %self.strategic_agent_algorithm.objective_value());

            self.update_tactical_agent();
        }
        ctx.notify(ScheduleIteration {});
    }
}

impl Handler<StrategicRequestMessage> for StrategicAgent {
    type Result = Result<String, AgentError>;

    #[instrument(level = "info", skip_all)]
    fn handle(
        &mut self,
        strategic_request_message: StrategicRequestMessage,
        _ctx: &mut Self::Context,
    ) -> Self::Result {
        match strategic_request_message {
            StrategicRequestMessage::Status(strategic_status_message) => {
                match strategic_status_message {
                    StrategicStatusMessage::General => {
                        let scheduling_status =
                            self.scheduling_environment.lock().unwrap().to_string();
                        let strategic_objective =
                            self.strategic_agent_algorithm.objective_value();

                        let optimized_work_orders =
                            self.strategic_agent_algorithm.optimized_work_orders();

                        let number_of_strategic_work_orders = optimized_work_orders.len();
                        let mut scheduled_count = 0;
                        for optimized_work_order in optimized_work_orders.values() {
                            if optimized_work_order.scheduled_period.is_some() {
                                scheduled_count += 1;
                            }
                        }

                        let asset = &self.asset;

                        let number_of_periods = self.strategic_agent_algorithm.periods().len();

                        let strategic_response_status = StrategicResponseStatus::new(asset.clone(), strategic_objective, number_of_strategic_work_orders, number_of_periods);

                        Ok(serde_json::to_string(&strategic_response_status).unwrap())
                    }
                    StrategicStatusMessage::Period(period) => {
                        let work_orders = self.strategic_agent_algorithm.optimized_work_orders();

                        if !self
                            .strategic_agent_algorithm
                            .periods()
                            .iter()
                            .map(|period| period.period_string())
                            .collect::<Vec<_>>()
                            .contains(&period)
                        {
                            return Err(AgentError::StateUpdateError(
                                "Period not found in the the scheduling environment".to_string(),
                            ));
                        }

                        let work_orders_by_period: Vec<u32> = work_orders
                            .iter()
                            .filter(|(_, opt_wo)| match opt_wo.scheduled_period.clone() {
                                Some(scheduled_period) => {
                                    scheduled_period.period_string() == period
                                }
                                None => false,
                            })
                            .map(|(work_order_number, _)| *work_order_number)
                            .collect();
                        let message =
                            self.format_selected_work_orders(work_orders_by_period, Some(period));

                        Ok(message)
                    }
                }
            }
            StrategicRequestMessage::Scheduling(scheduling_message) => {
                let scheduling_output = self
                    .strategic_agent_algorithm
                    .update_scheduling_state(scheduling_message);

                self.strategic_agent_algorithm.calculate_objective_value();
                info!(strategic_objective_value = %self.strategic_agent_algorithm.objective_value());
                scheduling_output
            }
            StrategicRequestMessage::Resources(resources_message) => {
                let resources_output = self
                .strategic_agent_algorithm
                .update_resources_state(resources_message);

                self.strategic_agent_algorithm.calculate_objective_value();
                info!(strategic_objective_value = %self.strategic_agent_algorithm.objective_value());
                resources_output
                }
            StrategicRequestMessage::Periods(periods_message) => {
                let mut scheduling_env_lock = self.scheduling_environment.lock().unwrap();

                let periods = scheduling_env_lock.periods_mut();

                for period_id in periods_message.periods.iter() {
                    if periods.last().unwrap().id() + 1 == *period_id {
                        let new_period =
                            periods.last().unwrap().clone() + chrono::Duration::weeks(2);
                        periods.push(new_period);
                    } else {
                        error!("periods not handled correctly");
                    }
                }
                self.strategic_agent_algorithm.set_periods(periods.to_vec());
                Ok("Periods updated".to_string())
            }
            StrategicRequestMessage::Test => {
                let algorithm_state = self.determine_algorithm_state();

                match algorithm_state {
                    AlgorithmState::Feasible => Ok(
                        "Strategic Schedule is Feasible (Additional tests may be needed)"
                            .to_string(),
                    ),
                    AlgorithmState::Infeasible(infeasible_cases) => Ok(format!(
                        "Strategic Schedule is Infesible: \n\
                            \t{:30}{:>20}\n\
                            \t{:30}{:>20}\n\
                            \t{:30}{:>20}\n\
                            \t{:30}{:>20}\n",
                        "respect_awsc: ",
                        &infeasible_cases.respect_awsc,
                        "respect_unloading: ",
                        &infeasible_cases.respect_unloading,
                        "respect_sch: ",
                        &infeasible_cases.respect_sch,
                        "respect_aggregated_load: ",
                        &infeasible_cases.respect_aggregated_load
                    )),
                }
            }
        }
    }
}

impl Handler<StatusMessage> for StrategicAgent {
    type Result = String;

    fn handle(&mut self, _msg: StatusMessage, _ctx: &mut Self::Context) -> Self::Result {
        format!(
            "Objective: {}",
            self.strategic_agent_algorithm.objective_value()
        )
    }
}

impl Handler<SetAddr> for StrategicAgent {
    type Result = ();

    fn handle(&mut self, msg: SetAddr, _ctx: &mut Context<Self>) {
        match msg {
            SetAddr::Tactical(addr) => {
                self.tactical_agent_addr = Some(addr);
            }
            _ => {
                println!("The strategic agent received an Addr<T>, where T is not a valid Actor");
                todo!()
            }
        }
    }
}

impl Handler<SolutionExportMessage> for StrategicAgent {
    type Result = String;

    fn handle(&mut self, _msg: SolutionExportMessage, _ctx: &mut Self::Context) -> Self::Result {
        let mut strategic_solution = HashMap::new();
        for (work_order_number, optimized_work_order) in self
            .strategic_agent_algorithm
            .optimized_work_orders()
            .iter()
        {
            strategic_solution.insert(
                *work_order_number,
                optimized_work_order
                    .scheduled_period
                    .as_ref()
                    .unwrap()
                    .period_string(),
            );
        }

        serde_json::to_string(&strategic_solution).unwrap()
    }
}

impl TestAlgorithm for StrategicAgent {
    type InfeasibleCases = StrategicInfeasibleCases;

    fn determine_algorithm_state(&self) -> AlgorithmState<Self::InfeasibleCases> {
        let scheduling_environment = self.scheduling_environment.lock().unwrap();

        let mut strategic_state = AlgorithmState::Infeasible(Self::InfeasibleCases::default());

        for (work_order_number, optimized_work_order) in
            self.strategic_agent_algorithm.optimized_work_orders()
        {
            let scheduled_period = optimized_work_order.scheduled_period.clone();
            let work_order = scheduling_environment
                .work_orders()
                .inner
                .get(work_order_number)
                .unwrap();
            let first_period = self.strategic_agent_algorithm.periods().first().unwrap();

            let basic_start_of_first_activity = work_order.order_dates().basic_start_date;

            let awsc = work_order.status_codes().awsc;

            match scheduled_period {
                Some(scheduled_period) => {
                    if awsc
                        && !scheduled_period.contains_date(basic_start_of_first_activity)
                        && &basic_start_of_first_activity > first_period.start_date()
                    {
                        strategic_state.infeasible_cases_mut().unwrap().respect_awsc =
                            ConstraintState::Infeasible(format!(
                                "Work order {} does not respect AWSC. Period: {}, basic start date: {}, status codes: {}, unloading_point: {:?}, vendor: {}",
                                work_order_number,
                                scheduled_period,
                                basic_start_of_first_activity,
                                work_order.status_codes(),
                                work_order.unloading_point().period,
                                if work_order.is_vendor() { "VEN" } else { "   " },
                            ));
                        break;
                    }
                }
                None => {
                    strategic_state.infeasible_cases_mut().unwrap().respect_awsc =
                        ConstraintState::Infeasible(format!(
                            "Work order {} does not have a period",
                            work_order_number,
                        ));
                    break;
                }
            }
            strategic_state.infeasible_cases_mut().unwrap().respect_awsc =
                ConstraintState::Feasible;
        }

        for (work_order_number, optimized_work_order) in
            self.strategic_agent_algorithm.optimized_work_orders()
        {
            let work_order = scheduling_environment
                .work_orders()
                .inner
                .get(work_order_number)
                .unwrap();
            let periods = scheduling_environment.periods();

            if work_order.unloading_point().period.is_some()
                && work_order.unloading_point().period != optimized_work_order.scheduled_period
                && !periods[0..=1].contains(work_order.unloading_point().period.as_ref().unwrap())
                && !work_order.status_codes().awsc
                && !work_order.status_codes().sch
            {
                error!(
                    work_order_number = ?work_order_number,
                    work_order_unloading_point = ?work_order.unloading_point(),
                    work_order_status_codes = ?work_order.status_codes(),
                    work_order_dates = ?work_order.order_dates().basic_start_date,
                    periods = ?periods[0..=1],
                    optimized_work_order_scheduled_period = ?optimized_work_order.scheduled_period,
                    optimized_work_order_locked_in_period = ?optimized_work_order.locked_in_period,
                );
                strategic_state
                    .infeasible_cases_mut()
                    .unwrap()
                    .respect_unloading = ConstraintState::Infeasible(format!(
                    "\t\t\nWork order number: {}\t\t\nwith unloading period: {}\t\t\nwith scheduled period: {}\t\t\nwith locked period: {}",
                    work_order_number,
                    work_order.unloading_point().period.as_ref().unwrap(),
                    optimized_work_order.scheduled_period.clone().unwrap(),
                    optimized_work_order.locked_in_period.clone().unwrap(),
                ));
                break;
            }
            strategic_state
                .infeasible_cases_mut()
                .unwrap()
                .respect_unloading = ConstraintState::Feasible;
        }

        for (work_order_number, optimized_work_order) in
            self.strategic_agent_algorithm.optimized_work_orders()
        {
            let work_order = scheduling_environment
                .work_orders()
                .inner
                .get(work_order_number)
                .unwrap();
            let periods = scheduling_environment.periods();

            if work_order.status_codes().sch
                && !periods[0..=1]
                    .contains(&optimized_work_order.scheduled_period.as_ref().unwrap())
            {
                error!(
                    work_order_number = ?work_order_number,
                    work_order_unloading_point = ?work_order.unloading_point(),
                    work_order_status_codes = ?work_order.status_codes(),
                    work_order_dates = ?work_order.order_dates().basic_start_date,
                    periods = ?periods[0..=1],
                    optimized_work_order_scheduled_period = ?optimized_work_order.scheduled_period,
                    optimized_work_order_locked_in_period = ?optimized_work_order.locked_in_period,
                );
                strategic_state
                    .infeasible_cases_mut()
                    .unwrap()
                    .respect_sch = ConstraintState::Infeasible(format!(
                    "\t\t\nWork order number: {}\t\t\nwith scheduled period: {}\t\t\nwith locked period: {:?}\t\t\n work order status codes: {}\t\t\n work order unloading point: {:?}",
                    work_order_number,
                    optimized_work_order.scheduled_period.as_ref().unwrap(),
                    optimized_work_order.locked_in_period.as_ref(),
                    work_order.status_codes(),
                    work_order.unloading_point().period.as_ref(),
                ));
                break;
            }
            strategic_state.infeasible_cases_mut().unwrap().respect_sch = ConstraintState::Feasible;
        }

        let mut aggregated_strategic_load = StrategicResources::new(HashMap::new());
        for period in self.strategic_agent_algorithm.periods() {
            for (_work_order_number, optimized_work_order) in
                self.strategic_agent_algorithm.optimized_work_orders()
            {
                if optimized_work_order.scheduled_period.as_ref().unwrap() == &period.clone() {
                    let work_load = &optimized_work_order.work_load;
                    for resource in Resources::iter() {
                        let load = work_load.get(&resource).unwrap_or(&0.0);
                        aggregated_strategic_load.update_load(
                            &resource,
                            period,
                            *load,
                            LoadOperation::Add,
                        );
                    }
                }
            }
        }

        let mut feasible: bool = true;
        for (resource, periods) in aggregated_strategic_load.inner {
            for (period, load) in periods {
                match self
                    .strategic_agent_algorithm
                    .resources_loadings()
                    .inner
                    .get(&resource)
                    .unwrap()
                    .get(&period)
                {
                    Some(resource_load) if (*resource_load - load).abs() < 0.005 => continue,
                    Some(resource_load) => {
                        strategic_state.infeasible_cases_mut().unwrap().respect_aggregated_load = ConstraintState::Infeasible(format!("resource = {}, period = {}, aggregated_load = {:.3e}, resource_load = {:.3e}", resource, period, load, resource_load));
                        error!(resource = %resource, period = %period, aggregated_load = %load, resource_load = %resource_load);
                        feasible = false
                    }
                    None => {
                        panic!("aggregated load and resource loading are not identically shaped")
                    }
                }
            }
        }
        if feasible {
            strategic_state
                .infeasible_cases_mut()
                .unwrap()
                .respect_aggregated_load = ConstraintState::Feasible
        }

        strategic_state
    }
}

pub struct StrategicInfeasibleCases {
    respect_awsc: ConstraintState<String>,
    respect_unloading: ConstraintState<String>,
    respect_sch: ConstraintState<String>,
    respect_aggregated_load: ConstraintState<String>,
}

impl Default for StrategicInfeasibleCases {
    fn default() -> Self {
        StrategicInfeasibleCases {
            respect_awsc: ConstraintState::Infeasible("Infeasible".to_string()),
            respect_unloading: ConstraintState::Infeasible("Infeasible".to_string()),
            respect_sch: ConstraintState::Infeasible("Infeasible".to_string()),
            respect_aggregated_load: ConstraintState::Infeasible("Infeasible".to_string()),
        }
    }
}

#[cfg(test)]
mod tests {

    use chrono::{TimeZone, Utc};
    use shared_messages::strategic::strategic_request_scheduling_message::SingleWorkOrder;
    use shared_messages::strategic::strategic_request_scheduling_message::StrategicSchedulingMessage;
    use tests::strategic_algorithm::optimized_work_orders::OptimizedWorkOrder;
    use tests::strategic_algorithm::optimized_work_orders::OptimizedWorkOrders;

    use std::collections::HashMap;
    use std::collections::HashSet;

    use super::{strategic_algorithm::PriorityQueues, *};
    use shared_messages::resources::Resources;

    use crate::agents::strategic_agent::strategic_algorithm::optimized_work_orders::StrategicResources;
    use crate::models::time_environment::period::Period;
    use crate::models::work_order::operation::Operation;
    use crate::models::{work_order::*, WorkOrders};

    #[actix_rt::test]
    async fn test_scheduler_agent_handle() {
        let mut work_orders = WorkOrders::new();
        let mut work_load = HashMap::new();

        work_load.insert(Resources::MtnMech, 20.0);
        work_load.insert(Resources::MtnElec, 40.0);
        work_load.insert(Resources::Prodtech, 60.0);

        let work_order = WorkOrder::default();

        work_orders.insert(work_order.clone());

        let start_date = Utc.with_ymd_and_hms(2023, 11, 20, 0, 0, 0).unwrap();
        let end_date = start_date
            + chrono::Duration::days(13)
            + chrono::Duration::hours(23)
            + chrono::Duration::minutes(59)
            + chrono::Duration::seconds(59);
        let period = Period::new(1, start_date, end_date);

        let mut resource_capacity: HashMap<Resources, HashMap<Period, f64>> = HashMap::new();
        let mut resource_loadings: HashMap<Resources, HashMap<Period, f64>> = HashMap::new();

        let mut period_hash_map_150 = HashMap::new();
        let mut period_hash_map_0 = HashMap::new();
        period_hash_map_150.insert(period.clone(), 150.0);
        period_hash_map_0.insert(period.clone(), 0.0);

        resource_capacity.insert(Resources::MtnMech, period_hash_map_150.clone());
        resource_capacity.insert(Resources::MtnElec, period_hash_map_150.clone());
        resource_capacity.insert(Resources::Prodtech, period_hash_map_150.clone());

        resource_loadings.insert(Resources::MtnMech, period_hash_map_0.clone());
        resource_loadings.insert(Resources::MtnElec, period_hash_map_0.clone());
        resource_loadings.insert(Resources::Prodtech, period_hash_map_0.clone());

        let periods: Vec<Period> = vec![Period::new_from_string("2023-W47-48").unwrap()];

        let scheduler_agent_algorithm = StrategicAlgorithm::new(
            0.0,
            StrategicResources::new(resource_capacity),
            StrategicResources::new(resource_loadings),
            PriorityQueues::new(),
            OptimizedWorkOrders::new(HashMap::new()),
            HashSet::new(),
            periods,
        );

        let mut manual_resources = HashMap::new();

        let mut period_hash_map = HashMap::new();
        period_hash_map.insert(period.period_string(), 300.0);

        manual_resources.insert(Resources::MtnMech, period_hash_map.clone());
        manual_resources.insert(Resources::MtnElec, period_hash_map.clone());
        manual_resources.insert(Resources::Prodtech, period_hash_map.clone());

        let scheduler_agent = StrategicAgent::new(
            Asset::DF,
            Arc::new(Mutex::new(SchedulingEnvironment::default())),
            scheduler_agent_algorithm,
            None,
        );

        let live_scheduler_agent = scheduler_agent.start();

        let test_response: TestResponse = live_scheduler_agent
            .send(TestRequest {})
            .await
            .unwrap()
            .unwrap();

        assert_eq!(
            *test_response
                .manual_resources_capacity
                .get(&Resources::MtnMech)
                .unwrap()
                .get(&period)
                .unwrap(),
            150.0
        );
        assert_eq!(
            *test_response
                .manual_resources_capacity
                .get(&Resources::MtnElec)
                .unwrap()
                .get(&period)
                .unwrap(),
            150.0
        );
        assert_eq!(
            *test_response
                .manual_resources_capacity
                .get(&Resources::Prodtech)
                .unwrap()
                .get(&period)
                .unwrap(),
            150.0
        );
    }

    #[test]
    fn test_extract_state_to_scheduler_overview() {
        let mut operations: HashMap<u32, Operation> = HashMap::new();

        let operation_1 = Operation::new_test(10, Resources::MtnMech, 1.0);

        let operation_2 = Operation::new_test(20, Resources::MtnMech, 1.0);

        let operation_3 = Operation::new_test(30, Resources::MtnMech, 1.0);

        operations.insert(10, operation_1);
        operations.insert(20, operation_2);
        operations.insert(30, operation_3);

        let work_order_1 = WorkOrder::default();

        let mut work_orders = WorkOrders::new();

        work_orders.insert(work_order_1);
    }

    #[test]
    fn test_update_scheduler_state() {
        let period_string: String = "2023-W47-48".to_string();

        let schedule_work_order = SingleWorkOrder::new(2200002020, period_string);

        let strategic_scheduling_internal =
            StrategicSchedulingMessage::Schedule(schedule_work_order);

        let periods: Vec<Period> = vec![Period::new_from_string("2023-W47-48").unwrap()];

        let optimized_work_orders = OptimizedWorkOrders::new(HashMap::new());
        let mut scheduler_agent_algorithm = StrategicAlgorithm::new(
            0.0,
            StrategicResources::default(),
            StrategicResources::default(),
            PriorityQueues::new(),
            optimized_work_orders,
            HashSet::new(),
            periods.clone(),
        );

        let optimized_work_order = OptimizedWorkOrder::new(
            None,
            Some(periods[0].clone()),
            HashSet::new(),
            None,
            1000,
            HashMap::new(),
        );

        scheduler_agent_algorithm.set_optimized_work_order(2200002020, optimized_work_order);

        scheduler_agent_algorithm
            .update_scheduling_state(strategic_scheduling_internal)
            .unwrap();

        assert_eq!(
            scheduler_agent_algorithm
                .optimized_work_orders()
                .get(&2200002020)
                .as_ref()
                .unwrap()
                .locked_in_period
                .as_ref()
                .unwrap()
                .period_string(),
            "2023-W47-48"
        );
    }

    #[test]
    fn test_input_scheduler_message_from() {
        let schedule_single_work_order =
            SingleWorkOrder::new(2100023841, "2023-W49-50".to_string());

        let strategic_scheduling_message =
            StrategicSchedulingMessage::Schedule(schedule_single_work_order);

        assert_eq!(
            match strategic_scheduling_message {
                StrategicSchedulingMessage::Schedule(ref schedule_single_work_order) => {
                    schedule_single_work_order.get_work_order_number()
                }
                _ => panic!("wrong message type"),
            },
            2100023841
        );

        assert_eq!(
            match strategic_scheduling_message {
                StrategicSchedulingMessage::Schedule(ref schedule_single_work_order) => {
                    schedule_single_work_order.get_period_string()
                }
                _ => panic!("wrong message type"),
            },
            "2023-W49-50".to_string()
        );

        let mut work_load = HashMap::new();

        work_load.insert(Resources::VenMech, 16.0);

        let periods: Vec<Period> = vec![Period::new_from_string("2023-W49-50").unwrap()];

        let mut capacities = HashMap::new();
        let mut loadings = HashMap::new();

        let mut periods_hash_map_0 = HashMap::new();
        let mut periods_hash_map_16 = HashMap::new();

        periods_hash_map_0.insert(Period::new_from_string("2023-W49-50").unwrap(), 0.0);

        periods_hash_map_16.insert(Period::new_from_string("2023-W49-50").unwrap(), 16.0);

        capacities.insert(Resources::VenMech, periods_hash_map_16);

        loadings.insert(Resources::VenMech, periods_hash_map_0);

        let mut scheduler_agent_algorithm = StrategicAlgorithm::new(
            0.0,
            StrategicResources::new(capacities),
            StrategicResources::new(loadings),
            PriorityQueues::new(),
            OptimizedWorkOrders::new(HashMap::new()),
            HashSet::new(),
            periods.clone(),
        );

        let optimized_work_order = OptimizedWorkOrder::new(
            None,
            Some(periods[0].clone()),
            HashSet::new(),
            None,
            1000,
            work_load,
        );

        scheduler_agent_algorithm.set_optimized_work_order(2100023841, optimized_work_order);

        scheduler_agent_algorithm
            .update_scheduling_state(strategic_scheduling_message)
            .unwrap();

        assert_eq!(
            scheduler_agent_algorithm
                .optimized_work_order(&2100023841)
                .unwrap()
                .locked_in_period,
            Some(Period::new_from_string("2023-W49-50").unwrap())
        );
        assert_eq!(
            scheduler_agent_algorithm
                .optimized_work_order(&2100023841)
                .unwrap()
                .scheduled_period,
            None
        );
        // assert_eq!(scheduler_agent_algorithm.get_or_initialize_manual_resources_loading("VEN_MECH".to_string(), "2023-W49-50".to_string()), 16.0);
    }

    //
    #[test]
    fn test_calculate_objective_value() {
        let mut optimized_work_orders = OptimizedWorkOrders::new(HashMap::new());

        let optimized_work_order = OptimizedWorkOrder::new(
            Some(Period::new_from_string("2023-W49-50").unwrap()),
            Some(Period::new_from_string("2023-W49-50").unwrap()),
            HashSet::new(),
            Some(Period::new_from_string("2023-W47-48").unwrap()),
            1000,
            HashMap::new(),
        );

        optimized_work_orders.insert_optimized_work_order(2100023841, optimized_work_order);

        let mut scheduler_agent_algorithm = StrategicAlgorithm::new(
            0.0,
            StrategicResources::default(),
            StrategicResources::default(),
            PriorityQueues::new(),
            optimized_work_orders,
            HashSet::new(),
            vec![],
        );

        scheduler_agent_algorithm.calculate_objective_value();

        // This test fails because the objective value in not initialized
        assert_eq!(scheduler_agent_algorithm.objective_value(), 2000.0);
    }

    pub struct TestRequest {}

    impl Message for TestRequest {
        type Result = Option<TestResponse>;
    }

    pub struct TestResponse {
        pub objective_value: f64,
        pub manual_resources_capacity: HashMap<Resources, HashMap<Period, f64>>,
        pub manual_resources_loading: HashMap<Resources, HashMap<Period, f64>>,
        pub priority_queues: PriorityQueues<u32, u32>,
        pub optimized_work_orders: OptimizedWorkOrders,
        pub periods: Vec<Period>,
    }

    impl Handler<TestRequest> for StrategicAgent {
        type Result = Option<TestResponse>; // Or relevant part of the state

        fn handle(&mut self, _msg: TestRequest, _: &mut Context<Self>) -> Self::Result {
            // Return the state or part of it
            Some(TestResponse {
                objective_value: self.strategic_agent_algorithm.objective_value(),
                manual_resources_capacity: self
                    .strategic_agent_algorithm
                    .resources_capacities()
                    .inner
                    .clone(),
                manual_resources_loading: self
                    .strategic_agent_algorithm
                    .resources_loadings()
                    .inner
                    .clone(),
                priority_queues: PriorityQueues::new(),
                optimized_work_orders: OptimizedWorkOrders::new(
                    self.strategic_agent_algorithm
                        .optimized_work_orders()
                        .clone(),
                ),
                periods: self.strategic_agent_algorithm.periods().clone(),
            })
        }
    }
}
