pub mod optimized_work_orders;
pub mod assert_functions;

use crate::agents::traits::LargeNeighborHoodSearch;
use crate::agents::{SharedSolution, StrategicSolution, StrategicTacticalSolutionArcSwap};
use assert_functions::StrategicAlgorithmAssertions;
use optimized_work_orders::{StrategicParameterBuilder, StrategicParameters};
use priority_queue::PriorityQueue;
use rand::prelude::SliceRandom;
use shared_types::agent_error::AgentError;
use shared_types::scheduling_environment::WorkOrders;
use shared_types::scheduling_environment::time_environment::period::Period;
use shared_types::scheduling_environment::work_order::WorkOrderNumber;
use shared_types::scheduling_environment::work_order::operation::Work;use shared_types::scheduling_environment::worker_environment::resources::Resources;
use shared_types::strategic::StrategicResources;
use shared_types::strategic::strategic_request_periods_message::StrategicTimeRequest;
use shared_types::strategic::strategic_request_resources_message::StrategicResourceRequest;
use shared_types::strategic::strategic_request_scheduling_message::StrategicSchedulingRequest;
use shared_types::strategic::strategic_response_periods::StrategicResponsePeriods;use shared_types::strategic::strategic_response_resources::StrategicResponseResources;
use shared_types::strategic::strategic_response_scheduling::StrategicResponseScheduling;
use shared_types::{Asset, LoadOperation};
use std::collections::{HashMap, HashSet};
use std::fmt::Display;use std::hash::Hash;
use std::sync::Arc;use tracing::{error, event, info, instrument, trace, Level};

pub struct StrategicAlgorithm {
    pub objective_value: f64,
    pub resource_capacities: StrategicResources,
    pub resource_loadings: StrategicResources,
    priority_queues: PriorityQueues<WorkOrderNumber, u64>,
    pub strategic_parameters: StrategicParameters,
    pub strategic_solution: StrategicSolution,
    arc_swap_shared_solution: Arc<StrategicTacticalSolutionArcSwap>,
    loaded_shared_solution: arc_swap::Guard<Arc<SharedSolution>>,
    period_locks: HashSet<Period>,
    periods: Vec<Period>,
}

impl StrategicAlgorithm {
    pub fn load_shared_solution(&mut self) {
        self.loaded_shared_solution = self.arc_swap_shared_solution.0.load();
    }

    pub fn make_atomic_pointer_swap_for_with_the_better_strategic_solution(&self) {
        let mut shared_solution = (**self.loaded_shared_solution).clone();
        shared_solution.strategic = self.strategic_solution.clone();
        self.arc_swap_shared_solution.0.store(Arc::new(shared_solution));
    } 

    pub fn strategic_periods(&self) -> &HashMap<WorkOrderNumber, Option<Period>> {
        &self.strategic_solution.scheduled_periods
    }

    pub fn strategic_periods_mut(&mut self) -> &mut HashMap<WorkOrderNumber, Option<Period>> {
        &mut self.strategic_solution.scheduled_periods
    }

    pub fn set_periods(&mut self, periods: Vec<Period>) {
        self.periods = periods;
    }

    pub fn create_strategic_parameters(
        &mut self,
        work_orders: &WorkOrders,
        periods: &[Period],
        asset: &Asset,
    ) {
        for (work_order_number, work_order) in work_orders.inner.iter() {
            if &work_order.functional_location().asset != asset {
                continue;
            }

            let strategic_parameter = StrategicParameterBuilder::new().build_from_work_order(work_order, periods).build();
        
            // What should be coded here? 
            self.load_shared_solution();
            self.strategic_solution.scheduled_periods.insert(*work_order_number, None);
            
            self.strategic_parameters.insert_strategic_parameter(*work_order_number, strategic_parameter);
            self.make_atomic_pointer_swap_for_with_the_better_strategic_solution();
            let scheduled_period_option = self.strategic_solution.scheduled_periods.get(&work_order_number).unwrap().clone();

            if let Some(scheduled_period) = scheduled_period_option {
                self.update_loadings(*work_order_number, scheduled_period, LoadOperation::Add);
            }
        }
    }
}

impl StrategicAlgorithm {
    pub fn schedule_forced_work_orders(&mut self) {
        let mut work_order_numbers: Vec<WorkOrderNumber> = vec![];
        for (work_order_number, opt_work_order) in self.strategic_parameters.inner.iter() {
            let scheduled_period = self.strategic_solution.scheduled_periods.get(work_order_number).unwrap();
            if scheduled_period == &opt_work_order.locked_in_period {
                continue
            }
            if opt_work_order.locked_in_period.is_some() {
                work_order_numbers.push(*work_order_number);
            }
        }

        for work_order_number in work_order_numbers {
            self.schedule_forced_work_order(&work_order_number);
        }
    }

    pub fn schedule_normal_work_order(
        &mut self,
        work_order_number: WorkOrderNumber,
        period: &Period,
    ) -> Option<WorkOrderNumber> {
        let optimized_work_order = self
            .strategic_parameters
            .inner
            .get(&work_order_number)
            .unwrap()
            .clone();

        if period != self.periods().last().unwrap() {
            for (resource, resource_needed) in optimized_work_order.work_load.iter() {
                let resource_capacity: &Work = self
                    .resource_capacities
                    .inner
                    .get(&resource.clone())
                    .unwrap()
                    .0
                    .get(&period.clone())
                    .unwrap();

                let resource_loading: &Work = self
                    .resource_loadings
                    .inner
                    .get(&resource.clone())
                    .unwrap()
                    .0
                    .get(&period.clone())
                    .unwrap();

                if *resource_needed > resource_capacity - resource_loading {
                    return Some(work_order_number);
                }

                if optimized_work_order.excluded_periods().contains(period) {
                    return Some(work_order_number);
                }

                if self.period_locks.contains(period) {
                    return Some(work_order_number);
                }
            }
        }

        *self
            .strategic_periods_mut()
            .get_mut(&work_order_number)
            .expect("An entry in the Strategic part of the SharedSolution is missing. This should never occur")
            = Some(period.clone());
        
        self.update_loadings(work_order_number, period.clone(), LoadOperation::Add);
        None
    }

    pub fn schedule_forced_work_order(&mut self, work_order_number: &WorkOrderNumber) {
        if let Some(work_order_number) = self.is_scheduled(work_order_number) {
            self.unschedule(*work_order_number).unwrap();
        }

        let target_period = self
            .strategic_parameters
            .get_locked_in_period(work_order_number);

        *self.strategic_solution.scheduled_periods.get_mut(work_order_number).unwrap() = Some(target_period.clone());

        self.update_loadings(*work_order_number, target_period.clone(), LoadOperation::Add);
    }

    pub fn unschedule_random_work_orders(
        &mut self,
        number_of_work_orders: usize,
        rng: &mut impl rand::Rng,
    ) {
        event!(Level::WARN, "timing");
        let scheduled_periods = &self.strategic_solution.scheduled_periods;

        event!(Level::WARN, "timing");
        let strategic_parameter = &self.strategic_parameters.inner;

        event!(Level::WARN, "timing");
        let mut filtered_keys: Vec<_> = scheduled_periods
            .iter()
            .filter(|(key, _)| strategic_parameter.get(&key).unwrap().locked_in_period.is_none())
            .map(|(&key, _)| key)
            .collect();

        event!(Level::WARN, "timing");
        filtered_keys.sort();

        event!(Level::WARN, "timing");
        let sampled_work_order_keys = filtered_keys
            .choose_multiple(rng, number_of_work_orders)
            .collect::<Vec<_>>()
            .clone();

        event!(Level::WARN, "timing");
        // assert!(self.strategic_solution.scheduled_periods.values().all(|per| per.is_some()));
        for work_order_key in sampled_work_order_keys {
            self.unschedule(*work_order_key).unwrap();
            self.populate_priority_queues();
        }
        event!(Level::WARN, "timing");
    }

    fn is_scheduled<'a>(&'a self, work_order_number: &'a WorkOrderNumber) -> Option<&WorkOrderNumber> {
        self.strategic_periods()
            .get(work_order_number)
            .and_then(|scheduled_period| {
                scheduled_period
                    .as_ref()
                    .map(|_| work_order_number)
            })
    }

    fn update_loadings(&mut self, work_order_number: WorkOrderNumber, target_period: Period, load_operation: LoadOperation) {
        let work_load = self.strategic_parameters.inner.get(&work_order_number).unwrap().work_load.clone();
        for (resource, periods) in self.resource_loadings.inner.iter_mut() {
            for (period, loading) in &mut periods.0 {
                if *period == target_period {
                    match load_operation {
                        LoadOperation::Add => *loading += work_load.get(resource).unwrap_or(&Work::from(0.0)),
                        LoadOperation::Sub => *loading -= work_load.get(resource).unwrap_or(&Work::from(0.0)),
                    }
                }
            }
        }
    }
}

pub fn calculate_period_difference(scheduled_period: Period, latest_period: &Period) -> i64 {
    let scheduled_period_date = scheduled_period.end_date().to_owned();
    let latest_date = latest_period.end_date();
    let duration = scheduled_period_date.signed_duration_since(latest_date);
    let days = duration.num_days();
    days / 7
}

impl Display for StrategicAlgorithm {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "SchedulerAgentAlgorithm: \n
            objective_value: {}, \n
            manual_resources_capacity: {:?}, \n
            manual_resources_loading: {:?}, \n
            priority_queues: {:?}, \n
            optimized_work_orders: {:?}, \n
            periods: {:?}",
            self.objective_value,
            self.resource_capacities,
            self.resource_loadings,
            self.priority_queues,
            self.strategic_parameters,
            self.periods
        )
    }
}

impl LargeNeighborHoodSearch for StrategicAlgorithm {
    type BetterSolution = ();
    type SchedulingRequest = StrategicSchedulingRequest;
    type SchedulingResponse = StrategicResponseScheduling;
    type ResourceRequest = StrategicResourceRequest;
    type ResourceResponse = StrategicResponseResources;
    type TimeRequest = StrategicTimeRequest;
    type TimeResponse = StrategicResponsePeriods;
    type SchedulingUnit = WorkOrderNumber;
    type Error = AgentError;
    
    fn calculate_objective_value(&mut self) -> Self::BetterSolution {
        let mut period_penalty_contribution: f64 = 0.0;
        let mut excess_penalty_contribution: f64 = 0.0;

        for (work_order_number, scheduled_period) in &self.strategic_solution.scheduled_periods {
            let optimized_period = match scheduled_period {
                Some(optimized_period) => optimized_period.clone(),
                None => {
                    error!("{:?} does not have a scheduled period", work_order_number);
                    panic!("{:?} does not have a scheduled period", work_order_number);
                }
            };

            let work_order_latest_allowed_finish_period =
                &self.strategic_parameters.inner.get(work_order_number).expect("StrategicParameter should always be available for the StrategicSolution").latest_period;

            let period_difference = calculate_period_difference(
                optimized_period,
                work_order_latest_allowed_finish_period,
            );
            
            let period_penalty = std::cmp::max(period_difference, 0) as f64
                * self
                    .strategic_parameters
                    .inner
                    .get(work_order_number)
                    .unwrap()
                    .weight as f64;

            period_penalty_contribution += period_penalty;
        }
        
        for (resource, periods) in &self.resources_capacities().inner {
            for (period, capacity) in &periods.0 {
                let loading = self
                    .resources_loading(resource, period);

                if loading - capacity > Work::from(0.0) {
                    excess_penalty_contribution += (loading - capacity).to_f64()
                }
            }
        }

        self.objective_value = 
            period_penalty_contribution + 0.0 * excess_penalty_contribution;

    }

    #[instrument(level = "trace", skip_all)]
    fn schedule(&mut self) {
        while !self.priority_queues.normal.is_empty() {
            for period in self.periods.clone() {
                let (work_order_number, weight) = match self.priority_queues.normal.pop() {
                    Some((work_order_number, weight)) => (work_order_number, weight),
                    None => {
                        break;
                    }
                };

                let inf_work_order_number =
                    self.schedule_normal_work_order(work_order_number, &period);

                if let Some(work_order_number) = inf_work_order_number {
                    self.priority_queues.normal.push(work_order_number, weight);
                }
            }
        }
    }
    
    fn unschedule(&mut self, work_order_number: Self::SchedulingUnit) -> Result<(), AgentError> {
  
        // Why can this not unschedule?
        let scheduled_period= self
            .strategic_periods_mut()
            .get_mut(&work_order_number)
            .unwrap();

        match scheduled_period.take() {
            Some(unschedule_from_period) => {
                self.update_loadings(work_order_number, unschedule_from_period, LoadOperation::Sub);
                Ok(())
            }
            None => Err(AgentError::StrategicAgentCouldNotUnschedule),
        }
    }

    #[instrument(skip_all, level = "info")]
    fn update_resources_state(
        &mut self,
        strategic_resources_message: Self::ResourceRequest,
    ) -> Result<Self::ResourceResponse, AgentError> 
    {
    //tracing::info!("update_resources_state called");
        match strategic_resources_message {
            StrategicResourceRequest::SetResources(manual_resources) => {
                let mut count = 0;
                for (resource, periods) in manual_resources.inner {
                    for (period_imperium, capacity) in periods.0 {
                        let period = self.periods.iter().find(|period| **period == period_imperium).expect("The period was not found in the self.periods vector. Somehow a message was sent form the frontend without the period being initialized correctly.");
                        self.resource_capacities
                            .inner
                            .get_mut(&resource.clone())
                            .expect("The resource was not found in the self.resources_capacity vector. Somehow a message was sent form the frontend without the resource being initialized correctly.")
                            .0
                            .insert(period.clone(), capacity);
                        count += 1;
                    }
                }

                Ok(StrategicResponseResources::UpdatedResources(count))
            }
            StrategicResourceRequest::GetLoadings {
                periods_end: _,
                select_resources: _,
            } => {
                let loading = self.resources_loadings();

                let strategic_response_resources = StrategicResponseResources::LoadingAndCapacities(loading.clone());
                Ok(strategic_response_resources)
            }
            StrategicResourceRequest::GetCapacities { periods_end: _, select_resources: _ } => 
            {         
                let capacities = self.resources_capacities();

                let strategic_response_resources = StrategicResponseResources::LoadingAndCapacities(capacities.clone());
                Ok(strategic_response_resources)
            }
            StrategicResourceRequest::GetPercentageLoadings { periods_end:_, resources: _ } => {
                let capacities = self.resources_capacities();
                let loadings = self.resources_loadings();

                Self::assert_that_capacity_is_respected(loadings, capacities);
                Ok(StrategicResponseResources::Percentage(capacities.clone(), loadings.clone()))
            }
        }
    }

    #[allow(dead_code)]
    fn update_time_state(&mut self, _time_message: Self::TimeRequest) -> Result<Self::TimeResponse, AgentError> 
        { todo!() }

    #[instrument(level = "info", skip_all)]
    fn update_scheduling_state(
        &mut self,
        strategic_scheduling_message: StrategicSchedulingRequest,
    ) -> Result<Self::SchedulingResponse, AgentError>
{
        match strategic_scheduling_message {
            StrategicSchedulingRequest::Schedule(schedule_work_order) => {
                let work_order_number = schedule_work_order.work_order_number();
                let period = self
                    .periods
                    .iter()
                    .find(|period| {
                        period.period_string() == schedule_work_order.period_string()
                    })
                    .cloned();
    
                match period {
                    Some(period) => {
                        self.strategic_parameters
                            .set_locked_in_period(*work_order_number, period.clone());
             
                        Ok(StrategicResponseScheduling::new(vec![*work_order_number], vec![period]))
                    }
                    None => Err(AgentError::StateUpdateError("Could not update strategic scheduling state".to_string())),
                }
            }
            StrategicSchedulingRequest::ScheduleMultiple(schedule_work_orders) => {
                let _output_string = String::new();
                let mut work_orders: Vec<WorkOrderNumber> = vec![];
                let mut periods: Vec<Period> = vec![];
                for schedule_work_order in schedule_work_orders {
                    let work_order_number = schedule_work_order.work_order_number();
                    let period = self
                        .periods
                        .iter()
                        .find(|period| {
                            period.period_string() == schedule_work_order.period_string()
                        })
                        .cloned();

                    match period {
                        Some(period) => {
                            self.strategic_parameters
                                .set_locked_in_period(*work_order_number, period.clone());

                            work_orders.push(*work_order_number);
                            periods.push(period);
                        }
                        None => {
                   
                            return Err(AgentError::StateUpdateError("The period was not found in the self.periods vector. Somehow a message was sent form the frontend without the period being initialized correctly.".to_string()))
                        }
                    }
                }
        
                Ok(StrategicResponseScheduling::new(work_orders, periods))
                 
            }
            StrategicSchedulingRequest::ExcludeFromPeriod(exclude_from_period) => {
                let work_order_number = exclude_from_period.work_order_number();
                let period = self.periods.iter().find(|period| {
                    period.period_string() == exclude_from_period.period_string().clone()
                });

                match period {
                    Some(period) => {
                        let optimized_work_order = 
                            self
                                .strategic_parameters
                                .inner
                                .get_mut(work_order_number)
                                .expect("The work order number was not found in the optimized work orders. The work order should have been initialized at the outset.");
                        
                        optimized_work_order
                            .excluded_periods
                            .insert(period.clone());

                        let overwrite = if let Some(locked_in_period) = &optimized_work_order.locked_in_period {
                            if optimized_work_order.excluded_periods.contains(locked_in_period) {
                                optimized_work_order.locked_in_period = None;
                                let last_period = self.periods.iter().last().cloned();
                                *self.strategic_solution.scheduled_periods.get_mut(work_order_number).unwrap() = last_period;
                                true
                            } else {
                                false
                            }
                        } else {
                            false
                        };

                        if overwrite {
                            info!("{:?} has been excluded from period {} and the locked in period has been removed", work_order_number, period.period_string());
                        }
                            
                        Ok(StrategicResponseScheduling::new(vec![*work_order_number], vec![period.clone()]))
                    }
                    None => Err(AgentError::StateUpdateError("The period was not found in the self.periods vector. Somehow a message was sent form the frontend without the period being initialized correctly.".to_string())),
                }
            }
        }
    }
}

impl StrategicAlgorithm {
    pub fn new(
        objective_value: f64,
        resources_capacity: StrategicResources,
        resources_loading: StrategicResources,
        priority_queues: PriorityQueues<WorkOrderNumber, u64>,
        strategic_parameters: StrategicParameters,
        strategic_tactical_solution_arc_swap: Arc<StrategicTacticalSolutionArcSwap>,
        period_locks: HashSet<Period>,
        periods: Vec<Period>,
    ) -> Self {

        let loaded_shared_solution = strategic_tactical_solution_arc_swap.0.load();

        StrategicAlgorithm {
            objective_value,
            resource_capacities: resources_capacity,
            resource_loadings: resources_loading,
            priority_queues,
            strategic_parameters,
            strategic_solution: StrategicSolution::default() ,
            arc_swap_shared_solution: strategic_tactical_solution_arc_swap,
            loaded_shared_solution,
            periods,
            period_locks,
            
        }
    }

    pub fn resources_loadings(&self) -> &StrategicResources {
        &self.resource_loadings
    }

    pub fn resources_loading(&self, resource: &Resources, period: &Period) -> &Work {
        self.resource_loadings.inner.get(resource).unwrap().0.get(period).unwrap()
    }

    pub fn resources_capacities(&self) -> &StrategicResources {
        &self.resource_capacities
    }


    pub fn periods(&self) -> &Vec<Period> {
        &self.periods
    }

    pub fn populate_priority_queues(&mut self) {
        for work_order_number in self.strategic_solution.scheduled_periods.keys() {
            trace!("Work order {:?} has been added to the normal queue", work_order_number);
            let strategic_work_order_weight = self.strategic_parameters.inner.get(work_order_number).expect("The StrategicParameter should always be available for the StrategicSolution").weight;

            if self.strategic_periods().get(work_order_number).unwrap().is_none() {
                self.priority_queues
                    .normal
                    .push(*work_order_number, strategic_work_order_weight);
            }
        }
    }

}


#[derive(Debug, Clone)]
pub struct PriorityQueues<T, P>
where
    T: Hash + Eq,
    P: Ord,
{
    pub normal: PriorityQueue<T, P>,
}

impl PriorityQueues<WorkOrderNumber, u64> {
    pub fn new() -> Self {
        Self {
            normal: PriorityQueue::<WorkOrderNumber, u64>::new(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use optimized_work_orders::StrategicParameter;
    use shared_types::strategic::{strategic_request_scheduling_message::SingleWorkOrder, Periods};
    use chrono::{Duration, TimeZone, Utc};
    use rand::{rngs::StdRng, SeedableRng};

    use shared_types::scheduling_environment::worker_environment::resources::Resources;

    use crate::agents::strategic_agent::strategic_algorithm::{
            StrategicParameters, PriorityQueues, StrategicAlgorithm,
        };

    use std::{collections::HashMap, str::FromStr};

    use shared_types::scheduling_environment::{
        work_order::WorkOrder,
            WorkOrders,
    };

    impl StrategicAlgorithm {
        
    pub fn optimized_work_order(&self, work_order_number: &WorkOrderNumber) -> Option<&StrategicParameter> {
        self.strategic_parameters.inner.get(work_order_number)
    }
    }
    #[test]
    fn test_update_scheduler_algorithm_state() {
        let work_order_number = WorkOrderNumber(1212123212);
        let mut work_orders = WorkOrders::default();

        let work_order = WorkOrder::work_order_test(     );

        work_orders.insert(work_order.clone());

        let period = Period::from_str("2023-W47-48").unwrap();
        let periods = vec![period.clone()];

        let mut manual_resource_capacity: HashMap<Resources, Periods> = HashMap::new();

        let mut hash_map_periods_150 = Periods(HashMap::new());

        hash_map_periods_150.0.insert(period.clone(), Work::from(150.0));

        manual_resource_capacity.insert(
            Resources::MtnMech,
            hash_map_periods_150.clone()
        );

        manual_resource_capacity.insert(
            Resources::MtnElec,
            hash_map_periods_150.clone()
        );

        manual_resource_capacity.insert(
            Resources::Prodtech,
            hash_map_periods_150.clone()
        );

        let resource_capacity = StrategicResources {
            inner: manual_resource_capacity.clone(),
        };

        let mut scheduler_agent_algorithm = StrategicAlgorithm::new(
            0.0,
            resource_capacity,
            StrategicResources::default(),
            PriorityQueues::new(),
            StrategicParameters::new(HashMap::new()),
            StrategicTacticalSolutionArcSwap::default().into(),
            HashSet::new(),
            periods,
        );

        let optimized_work_order =
            StrategicParameter::new(
                Some(period.clone()), 
                HashSet::new(),
                period.clone(),
                1000,
                HashMap::new()
                );

        scheduler_agent_algorithm.set_optimized_work_order(work_order_number, optimized_work_order);

        let strategic_scheduling_message = StrategicSchedulingRequest::Schedule(
            SingleWorkOrder::new(work_order_number, "2023-W47-48".to_string()),
        );
        let strategic_resources_message = StrategicResourceRequest::new_test();

        assert_eq!(
            scheduler_agent_algorithm.resource_capacities.inner
                .get(&Resources::MtnMech)
                .unwrap()
                .0
                .get(&period)
            ,
            Some(&Work::from(150.0))
        );
        assert_eq!(
            scheduler_agent_algorithm
                .strategic_parameters
                .inner
                .get(&work_order_number),
            Some(&StrategicParameter::new(
                Some(period.clone()),
                HashSet::new(),
                period.clone(),
                1000,
                HashMap::new()
            ))
        );

        scheduler_agent_algorithm.update_scheduling_state(strategic_scheduling_message).unwrap();
        scheduler_agent_algorithm.update_resources_state(strategic_resources_message).unwrap();

        assert_eq!(
            scheduler_agent_algorithm.resource_capacities.inner
                .get(&Resources::MtnMech)
                .unwrap()
                .0
                .get(&period)
            ,
            Some(&Work::from(300.0))
        );
        assert_eq!(
            *scheduler_agent_algorithm
                .strategic_periods()
                .get(&work_order_number)
                .unwrap(),
            None
        );
        assert_eq!(
            scheduler_agent_algorithm
                .strategic_parameters
                .inner
                .get(&work_order_number)
                .unwrap()
                .locked_in_period,
            Some(period.clone())
        );
    }

    impl StrategicParameter {
        pub fn new(
            locked_in_period: Option<Period>,
            excluded_periods: HashSet<Period>,
            latest_period: Period,
            weight: u64,
            work_load: HashMap<Resources, Work>,
        ) -> Self {
            Self {
                locked_in_period,
                excluded_periods,
                latest_period,
                weight,
                work_load,
            }
        }
    }

    #[test]
    fn test_schedule_work_order() {
        let work_order_number = WorkOrderNumber(1923010293);
        let start_date = Utc.with_ymd_and_hms(2023, 11, 20, 7, 0, 0).unwrap();
        let end_date = start_date + chrono::Duration::days(13);

        let period = Period::new(1, start_date, end_date);

        let mut optimized_work_orders = StrategicParameters::new(HashMap::new());

        let optimized_work_order =
            StrategicParameter::new( None, HashSet::new(), period.clone(), 1000, HashMap::new());

        optimized_work_orders.insert_strategic_parameter(work_order_number, optimized_work_order);

        let mut resource_capacity = HashMap::new();
        let mut resource_loadings = HashMap::new();

        let mut period_hash_map_150 = Periods(HashMap::new());
        let mut period_hash_map_0 = Periods(HashMap::new());

        period_hash_map_150.insert(period.clone(), Work::from(150.0));
        period_hash_map_0.insert(period.clone(), Work::from(0.0));

        resource_capacity.insert(Resources::MtnMech, period_hash_map_150.clone());
        resource_capacity.insert(Resources::MtnElec, period_hash_map_150.clone());
        resource_capacity.insert(Resources::Prodtech, period_hash_map_150.clone());

        resource_loadings.insert(Resources::MtnMech, period_hash_map_0.clone());
        resource_loadings.insert(Resources::MtnElec, period_hash_map_0.clone());
        resource_loadings.insert(Resources::Prodtech, period_hash_map_0.clone());
        let mut scheduler_agent_algorithm = StrategicAlgorithm::new(
            0.0,
            StrategicResources::new(resource_capacity),
            StrategicResources::new(resource_loadings),
            PriorityQueues::new(),
            optimized_work_orders,
            StrategicTacticalSolutionArcSwap::default().into(),
            HashSet::new(),
            vec![period.clone()],
        );

        scheduler_agent_algorithm.schedule_normal_work_order(work_order_number, &period);

        assert_eq!(
            *scheduler_agent_algorithm
            .strategic_periods()
                .get(&work_order_number)
                .unwrap(),
            Some(period.clone())
        );
    }

    #[test]
    fn test_schedule_work_order_with_work_load() {
        let work_order_number = WorkOrderNumber(1923010293);
        let start_date = Utc.with_ymd_and_hms(2023, 11, 20, 7, 0, 0).unwrap();
        let end_date = start_date + chrono::Duration::days(13);

        let period = Period::new(1, start_date, end_date);

        let mut work_load = HashMap::new();

        work_load.insert(Resources::MtnMech,Work::from( 100.0));

        let mut optimized_work_orders = StrategicParameters::new(HashMap::new());

        let optimized_work_order =
            StrategicParameter::new( None, HashSet::new(), period.clone(), 1000, work_load);

        optimized_work_orders
            .inner
            .insert(work_order_number, optimized_work_order);

        let mut resource_capacity = HashMap::new();
        let mut resource_loadings = HashMap::new();

        let mut period_hash_map_0 = Periods(HashMap::new());

        period_hash_map_0.insert(period.clone(),Work::from( 0.0));

        resource_capacity.insert(Resources::MtnMech, period_hash_map_0.clone());
        resource_loadings.insert(Resources::MtnMech, period_hash_map_0.clone());

        let mut scheduler_agent_algorithm = StrategicAlgorithm::new(
            0.0,
            StrategicResources::new(resource_capacity),
            StrategicResources::new(resource_loadings),
            PriorityQueues::new(),
            optimized_work_orders,
            StrategicTacticalSolutionArcSwap::default().into(),
            HashSet::new(),
            vec![period.clone()],
        );
        scheduler_agent_algorithm.schedule_normal_work_order(work_order_number, &period);

        assert_eq!(
            *scheduler_agent_algorithm
            .strategic_periods()
                .get(&work_order_number)
                .unwrap(),
            scheduler_agent_algorithm.periods().last().cloned()
        );
    }

    #[test]
    fn test_update_loadings() {
        let mut work_load = HashMap::new();

        work_load.insert(Resources::MtnMech,Work::from( 20.0));
        work_load.insert(Resources::MtnElec,Work::from( 40.0));
        work_load.insert(Resources::Prodtech,Work::from( 60.0));

        let start_date = Utc.with_ymd_and_hms(2023, 11, 20, 7, 0, 0).unwrap();
        let end_date = start_date + chrono::Duration::days(13);

        let period = Period::new(1, start_date, end_date);

        let mut resource_capacity = HashMap::new();
        let mut resource_loadings = HashMap::new();

        let mut period_hash_map_150 = Periods(HashMap::new());
        let mut period_hash_map_0 = Periods(HashMap::new());

        period_hash_map_150.insert(period.clone(),Work::from( 150.0));
        period_hash_map_0.insert(period.clone(),Work::from( 0.0));

        resource_capacity.insert(Resources::MtnMech, period_hash_map_150.clone());
        resource_capacity.insert(Resources::MtnElec, period_hash_map_150.clone());
        resource_capacity.insert(Resources::Prodtech, period_hash_map_150.clone());

        resource_loadings.insert(Resources::MtnMech, period_hash_map_0.clone());
        resource_loadings.insert(Resources::MtnElec, period_hash_map_0.clone());
        resource_loadings.insert(Resources::Prodtech, period_hash_map_0.clone());

        let mut strategic_agent_algorithm = StrategicAlgorithm::new(
            0.0,
            StrategicResources::new(resource_capacity),
            StrategicResources::new(resource_loadings),
            PriorityQueues::new(),
            StrategicParameters::new(HashMap::new()),
            StrategicTacticalSolutionArcSwap::default().into(),
            HashSet::new(),
            vec![],
        );

        let work_order_number = WorkOrderNumber(2100000001);
        
        let work_order = StrategicParameter::new(
            Some(period.clone()),
            HashSet::new(),
            period.clone(),
            1000,
            work_load,
        );

        strategic_agent_algorithm.strategic_parameters.inner.insert(work_order_number, work_order);
        strategic_agent_algorithm.update_loadings(work_order_number, period.clone(), LoadOperation::Add);

        assert_eq!(
            *strategic_agent_algorithm
                .resources_loading(&Resources::MtnMech, &period),
           Work::from( 20.0)
        );
        assert_eq!(
            *strategic_agent_algorithm
                .resources_loading(&Resources::MtnElec, &period),
           Work::from( 40.0)
        );
        assert_eq!(
            *strategic_agent_algorithm
                .resources_loading(&Resources::Prodtech, &period),
           Work::from( 60.0)
        );

        assert!(
            !strategic_agent_algorithm
                .resource_loadings
                .inner
                .contains_key(&Resources::MtnScaf),
                
        );
    }

    #[test]
    fn test_unschedule_work_order() {
        let work_order_number = WorkOrderNumber(2200002020);
        let mut work_load = HashMap::new();

        work_load.insert(Resources::MtnMech,Work::from( 20.0));
        work_load.insert(Resources::MtnElec,Work::from( 40.0));
        work_load.insert(Resources::Prodtech,Work::from( 60.0));

        let start_date = Utc.with_ymd_and_hms(2023, 11, 20, 0, 0, 0).unwrap();
        let end_date = start_date
            + chrono::Duration::days(13)
            + chrono::Duration::hours(23)
            + chrono::Duration::minutes(59)
            + chrono::Duration::seconds(59);
        let period_1 = Period::new(0, start_date, end_date);
        let period_2 = Period::new(0, start_date, end_date) + Duration::weeks(2);
        let period_3 = Period::new(0, start_date, end_date) + Duration::weeks(4);

        let periods: Vec<Period> = vec![period_1.clone(), period_2.clone(), period_3.clone()];
        let resources = [Resources::MtnMech, Resources::MtnElec, Resources::Prodtech];
        // Again, this is not completely correct. There is an invariant here that is not being
        // upheld correctly. What should we do about that?
        let mut resource_capacity: HashMap<Resources, Periods> = HashMap::new();
        let mut resource_loadings: HashMap<Resources, Periods> = HashMap::new();

        for resource in resources.iter() {
            let capacity_map = resource_capacity.entry(resource.clone()).or_default();
            let loading_map = resource_loadings.entry(resource.clone()).or_default();

            for period in periods.iter() {
                capacity_map.insert(period.clone(),Work::from( 150.0));
                loading_map.insert(period.clone(),Work::from( 0.0));
            }
        }

        let mut optimized_work_orders = StrategicParameters::new(HashMap::new());

        let optimized_work_order = StrategicParameter::new(
            Some(period_1.clone()),
            HashSet::new(),
            period_1.clone(),
            1000,
            work_load,
        );

        optimized_work_orders
            .inner
            .insert(work_order_number, optimized_work_order);

        let mut strategic_agent_algorithm = StrategicAlgorithm::new(
            0.0,
            StrategicResources::new(resource_capacity),
            StrategicResources::new(resource_loadings),
            PriorityQueues::new(),
            optimized_work_orders,
            StrategicTacticalSolutionArcSwap::default().into(),
            HashSet::new(),
            periods,
        );

        assert_eq!(
            *strategic_agent_algorithm
                .resources_loading(&Resources::MtnMech, &period_1),
           Work::from( 0.0)
        );
        strategic_agent_algorithm.schedule_normal_work_order(work_order_number, &period_1);

        assert_eq!(
            *strategic_agent_algorithm
                .resources_loading(&Resources::MtnMech, &period_1),
           Work::from( 20.0)
        );

        assert_eq!(
            *strategic_agent_algorithm
                .resources_loading(&Resources::MtnElec, &period_1),
           Work::from( 40.0)
        );
        assert_eq!(
            *strategic_agent_algorithm
                .resources_loading(&Resources::Prodtech, &period_1),
           Work::from( 60.0)
        );

        strategic_agent_algorithm.unschedule(work_order_number).unwrap();
        assert_eq!(
            *strategic_agent_algorithm
                .resources_loading(&Resources::MtnMech, &period_1),
           Work::from( 0.0)
        );
        assert_eq!(
            *strategic_agent_algorithm
                .resources_loading(&Resources::MtnElec, &period_1),
           Work::from( 0.0)
        );
        assert_eq!(
            *strategic_agent_algorithm
                .resources_loading(&Resources::Prodtech, &period_1),
           Work::from( 0.0)
        );

        strategic_agent_algorithm.schedule_forced_work_order(&work_order_number);

        assert_eq!(
            *strategic_agent_algorithm
                .resources_loading(&Resources::MtnMech, &period_1),
           Work::from( 20.0)
        );
        assert_eq!(
            *strategic_agent_algorithm
                .resources_loading(&Resources::MtnElec, &period_1),
           Work::from( 40.0)
        );
        assert_eq!(
            *strategic_agent_algorithm
                .resources_loading(&Resources::Prodtech, &period_1),
           Work::from( 60.0)
        );

        strategic_agent_algorithm
            .strategic_parameters
            .set_locked_in_period(work_order_number, period_2.clone());
        strategic_agent_algorithm.schedule_forced_work_order(&work_order_number);

        assert_eq!(
            *strategic_agent_algorithm
                .resources_loading(&Resources::MtnMech, &period_1),
           Work::from( 0.0)
        );
        assert_eq!(
            *strategic_agent_algorithm
                .resources_loading(&Resources::MtnElec, &period_1),
           Work::from( 0.0)
        );
        assert_eq!(
            *strategic_agent_algorithm
                .resources_loading(&Resources::Prodtech, &period_1),
           Work::from( 0.0)
        );

        assert_eq!(
            *strategic_agent_algorithm
                .resources_loading(&Resources::MtnMech, &period_2),
           Work::from( 20.0)
        );
        assert_eq!(
            *strategic_agent_algorithm
                .resources_loading(&Resources::MtnElec, &period_2),
           Work::from( 40.0)
        );
        assert_eq!(
            *strategic_agent_algorithm
                .resources_loading(&Resources::Prodtech, &period_2),
           Work::from( 60.0)
        );

        strategic_agent_algorithm.unschedule(work_order_number);
        assert_eq!(
            *strategic_agent_algorithm
                .resources_loading(&Resources::MtnMech, &period_1),
           Work::from( 0.0)
        );
        assert_eq!(
            *strategic_agent_algorithm
                .resources_loading(&Resources::MtnElec, &period_1),
           Work::from( 0.0)
        );
        assert_eq!(
            *strategic_agent_algorithm
                .resources_loading(&Resources::Prodtech, &period_1),
           Work::from( 0.0)
        );

        assert_eq!(
            *strategic_agent_algorithm
                .resources_loading(&Resources::MtnMech, &period_1),
           Work::from( 0.0)
        );
        assert_eq!(
            *strategic_agent_algorithm
                .resources_loading(&Resources::MtnElec, &period_1),
           Work::from( 0.0)
        );
        assert_eq!(
            *strategic_agent_algorithm
                .resources_loading(&Resources::Prodtech, &period_1),
           Work::from( 0.0)
        );
    }

    #[test]
    fn test_unschedule_random_work_orders() {
        let mut work_load_1 = HashMap::new();
        let mut work_load_2 = HashMap::new();
        let mut work_load_3 = HashMap::new();

        work_load_1.insert(Resources::MtnMech,Work::from( 10.0));
        work_load_1.insert(Resources::MtnElec,Work::from( 10.0));
        work_load_1.insert(Resources::Prodtech,Work::from( 10.0));

        work_load_2.insert(Resources::MtnMech,Work::from( 20.0));
        work_load_2.insert(Resources::MtnElec,Work::from( 20.0));
        work_load_2.insert(Resources::Prodtech,Work::from( 20.0));

        work_load_3.insert(Resources::MtnMech,Work::from( 30.0));
        work_load_3.insert(Resources::MtnElec,Work::from( 30.0));
        work_load_3.insert(Resources::Prodtech,Work::from( 30.0));

        let mut optimized_work_orders = StrategicParameters::new(HashMap::new());

        let optimized_work_order_1 = StrategicParameter::new(
            None,
            HashSet::new(),
Period::from_str("2023-W49-50").unwrap(),
            1000,
            work_load_1,
        );

        let optimized_work_order_2 = StrategicParameter::new(
            None,
            HashSet::new(),
Period::from_str("2023-W49-50").unwrap(),
            1000,
            work_load_2,
        );

        let optimized_work_order_3 = StrategicParameter::new(
            None,
            HashSet::new(),
Period::from_str("2023-W49-50").unwrap(),
            1000,
            work_load_3,
        );

        optimized_work_orders
            .inner
            .insert(WorkOrderNumber(2200000001), optimized_work_order_1);
        optimized_work_orders
            .inner
            .insert(WorkOrderNumber(2200000002), optimized_work_order_2);
        optimized_work_orders
            .inner
            .insert(WorkOrderNumber(2200000003), optimized_work_order_3);

        let periods: Vec<Period> = vec![
            Period::from_str("2023-W47-48").unwrap(),
            Period::from_str("2023-W49-50").unwrap(),
        ];

        let mut scheduler_agent_algorithm = StrategicAlgorithm::new(
            0.0,
            StrategicResources::default(),
            StrategicResources::default(),
            PriorityQueues::new(),
            optimized_work_orders,
            StrategicTacticalSolutionArcSwap::default().into(),
            HashSet::new(),
            periods,
        );

        let seed: [u8; 32] = [
            1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16, 17, 18, 19, 20, 21, 22, 23, 24,
            25, 26, 27, 28, 29, 30, 31, 32,
        ];

        let mut rng = StdRng::from_seed(seed);

        scheduler_agent_algorithm.unschedule_random_work_orders(2, &mut rng);

        assert_eq!(
            *scheduler_agent_algorithm
                .strategic_periods()
                .get(&WorkOrderNumber(2200000001))
                .unwrap(),
            Some(Period::from_str("2023-W47-48").unwrap())
        );

        assert_eq!(
            *scheduler_agent_algorithm
                .strategic_periods()
                .get(&WorkOrderNumber(2200000002))
                .unwrap(),
            None
        );

        assert_eq!(

            *scheduler_agent_algorithm
                .strategic_periods()
                .get(&WorkOrderNumber(2200000003))
                .unwrap(),
            None
        );
    }

    #[test]
    fn test_calculate_period_difference() {
        let period_1 = Period::from_str("2023-W47-48");
        let period_2 = Period::from_str("2023-W49-50");

        let difference = calculate_period_difference(period_1.unwrap(), &period_2.unwrap());

        assert_eq!(difference, -2);
    }

    #[test]
    fn test_choose_multiple() {
        for _ in 0..19 {
            let seed: [u8; 32] = [
                1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16, 17, 18, 19, 20, 21, 22, 23,
                24, 25, 26, 27, 28, 29, 30, 31, 32,
            ];

            let mut rng = StdRng::from_seed(seed);

            assert_eq!(
                [1, 2, 3].choose_multiple(&mut rng, 2).collect::<Vec<_>>(),
                [&3, &2]
            );
        }
    }

    #[test]
    fn test_unschedule_work_order_none_in_scheduled_period() {
        let work_order_number = WorkOrderNumber(2100000001);
        let mut optimized_work_orders = StrategicParameters::new(HashMap::new());

        let optimized_work_order = StrategicParameter::new(
            None,
            HashSet::new(),
            Period::from_str("2023-W47-48").unwrap(),
            1000,
            HashMap::new(),
        );

        optimized_work_orders
            .inner
            .insert(work_order_number, optimized_work_order);

        let mut strategic_agent_algorithm = StrategicAlgorithm::new(
            0.0,
            StrategicResources::default(),
            StrategicResources::default(),
            PriorityQueues::new(),
            optimized_work_orders,
            StrategicTacticalSolutionArcSwap::default().into(),
            HashSet::new(),
            vec![Period::from_str("2023-W47-48").unwrap()],
        );

        strategic_agent_algorithm.unschedule(work_order_number);
        assert_eq!(
            *strategic_agent_algorithm
                .strategic_periods()
                .get(&work_order_number)
                .unwrap(),
            None
        );
    }

    #[test]
    fn test_period_clone_equality() {
        let period_1 = Period::from_str("2023-W47-48").unwrap();
        let period_2 = Period::from_str("2023-W47-48").unwrap();

        assert_eq!(period_1, period_2);
        assert_eq!(period_1, period_1.clone());
    }

    impl StrategicAlgorithm {
        pub fn set_optimized_work_order(
            &mut self,
            work_order_number: WorkOrderNumber,
            optimized_work_order: StrategicParameter,
        ) {
            self.strategic_parameters
                .inner
                .insert(work_order_number, optimized_work_order);
        }
    }
}
