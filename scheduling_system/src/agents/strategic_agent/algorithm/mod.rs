pub mod strategic_parameters;
pub mod assert_functions;

use assert_functions::StrategicAssertions;
use shared_types::scheduling_environment::time_environment::TimeEnvironment;
use strum::IntoEnumIterator;
use crate::agents::traits::LargeNeighborhoodSearch;
use crate::agents::{SharedSolution, StrategicSolution, ArcSwapSharedSolution};
use anyhow::{anyhow, bail, Context, Result};
use strategic_parameters::{StrategicParameter, StrategicParameterBuilder, StrategicParameters};
use priority_queue::PriorityQueue;
use rand::prelude::SliceRandom;
use shared_types::scheduling_environment::WorkOrders;
use shared_types::scheduling_environment::time_environment::period::Period;
use shared_types::scheduling_environment::work_order::WorkOrderNumber;
use shared_types::scheduling_environment::work_order::operation::Work;
use shared_types::scheduling_environment::worker_environment::resources::Resources;
use shared_types::strategic::{StrategicObjectiveValue, StrategicResources};
use shared_types::strategic::strategic_request_periods_message::StrategicTimeRequest;
use shared_types::strategic::strategic_request_resources_message::StrategicResourceRequest;
use shared_types::strategic::strategic_request_scheduling_message::StrategicSchedulingRequest;
use shared_types::strategic::strategic_response_periods::StrategicResponsePeriods;use shared_types::strategic::strategic_response_resources::StrategicResponseResources;
use shared_types::strategic::strategic_response_scheduling::StrategicResponseScheduling;
use shared_types::{Asset, LoadOperation};
use std::any::type_name;
use std::collections::{HashMap, HashSet};
use std::hash::Hash;
use std::sync::Arc;

use tracing::{event, instrument, Level};

pub struct StrategicAlgorithm {
    priority_queues: PriorityQueues<WorkOrderNumber, u64>,
    pub strategic_parameters: StrategicParameters,
    pub strategic_solution: StrategicSolution,
    arc_swap_shared_solution: Arc<ArcSwapSharedSolution>,
    pub loaded_shared_solution: arc_swap::Guard<Arc<SharedSolution>>,
    period_locks: HashSet<Period>,
    pub strategic_periods: Vec<Period>,
}

impl StrategicAlgorithm {
    pub fn load_shared_solution(&mut self) {
        self.loaded_shared_solution = self.arc_swap_shared_solution.0.load();
    }

    pub fn make_atomic_pointer_swap(&self) {
        // Performance enhancements:
        // * COW:
        //      #[derive(Clone)]
        //      struct SharedSolution<'a> {
        //          tactical: Cow<'a, TacticalSolution>,
        //          // other fields...
        //      }
        //
        // * Reuse the old SharedSolution, cloning only the fields that are needed.
        //     let shared_solution = Arc::new(SharedSolution {
        //             tactical: self.tactical_solution.clone(),
        //             // Copy over other fields without cloning
        //             ..(**old).clone()
        //         });
        self.arc_swap_shared_solution.0.rcu(|old| {
            let mut shared_solution = (**old).clone();
            shared_solution.strategic = self.strategic_solution.clone();
            Arc::new(shared_solution)
        });
    
    } 

    pub fn strategic_periods(&self) -> &HashMap<WorkOrderNumber, Option<Period>> {
        &self.strategic_solution.strategic_periods
    }

    pub fn strategic_periods_mut(&mut self) -> &mut HashMap<WorkOrderNumber, Option<Period>> {
        &mut self.strategic_solution.strategic_periods
    }

    pub fn set_periods(&mut self, periods: Vec<Period>) {
        self.strategic_periods = periods;
    }

    pub fn update_the_locked_in_period(&mut self, work_order_number: &WorkOrderNumber, locked_in_period: &Period) -> Result<()> {
        self.strategic_solution.strategic_periods.insert(*work_order_number, Some(locked_in_period.clone()));

        let strategic_parameter = self.strategic_parameters.strategic_work_order_parameters.get_mut(work_order_number).with_context(|| format!("{:?} not found in {}", work_order_number, std::any::type_name::<StrategicParameters>()))?;

        strategic_parameter.excluded_periods.remove(locked_in_period);
        strategic_parameter.locked_in_period = Some(locked_in_period.clone());
        Ok(())
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
            self.strategic_solution.strategic_periods.insert(*work_order_number, None);
            
            self.strategic_parameters.insert_strategic_parameter(*work_order_number, strategic_parameter);
            self.make_atomic_pointer_swap();
            let scheduled_period_option = self.strategic_solution.strategic_periods.get(work_order_number).unwrap().clone();

            if let Some(scheduled_period) = scheduled_period_option {
                self.update_loadings(*work_order_number, scheduled_period, LoadOperation::Add);
            }
        }
    }

    fn strategic_capacity(&self, resource: &Resources, period: &Period) -> Result<&Work> {
        self
            .strategic_parameters
            .strategic_capacity
            .inner
            .get(&resource.clone())
            .with_context(|| format!("{} not found in {:?}", resource, std::any::type_name::<StrategicResources>()))?
            .0
            .get(&period.clone()).ok_or(anyhow!("{} not found is {:?}", period, std::any::type_name::<StrategicResources>()))
    }

    fn strategic_loading(&self, resource: &Resources, period: &Period) -> Result<&Work> {
        self
            .strategic_solution
            .strategic_loadings
            .inner
            .get(&resource.clone())
            .with_context(|| format!("{} not found in {:?}", resource, std::any::type_name::<StrategicResources>()))?            
            .0
            .get(&period.clone()).ok_or(anyhow!("{} not found is {:?}", period, std::any::type_name::<StrategicResources>()))
    }

    pub fn calculate_utilization(&self) -> Result<Vec<(i32, u64)>> {
        let mut utilization_by_period = Vec::new();

        for period in &self.strategic_periods {
            let mut intermediate_loading: f64 = 0.0;
            let mut intermediate_capacity: f64 = 0.0;
            for resource in Resources::iter() {
                let loading = self.strategic_loading(&resource, period)?;
                let capacity = self.strategic_capacity(&resource, period)?;

                intermediate_loading += loading.to_f64();
                intermediate_capacity += capacity.to_f64();
                
            }
            let percentage_loading = ((intermediate_loading / intermediate_capacity) * 100.0) as u64;
            utilization_by_period.push((*period.id(), percentage_loading));
        }
        Ok(utilization_by_period)
    }
}

#[derive(Debug)]
pub enum ForcedWorkOrder {
    Locked,
    FromTactical(Period),
}

impl StrategicAlgorithm {
    pub fn schedule_forced_work_orders(&mut self) -> Result<()> {
        let tactical_work_orders = &self.loaded_shared_solution.tactical.tactical_scheduled_work_orders;
        let mut work_order_numbers: Vec<(WorkOrderNumber, ForcedWorkOrder)> = vec![];
        for (work_order_number, strategic_parameter) in self.strategic_parameters.strategic_work_order_parameters.iter() {
            let scheduled_period = self
                .strategic_solution.strategic_periods
                .get(work_order_number)
                .unwrap();

            let tactical_work_order = tactical_work_orders
                .0
                .get(work_order_number)
                .expect("State should always be present except if the TacticalAgent has not had time to initialize yet");

            if scheduled_period == &strategic_parameter.locked_in_period {
                continue
            }

            if strategic_parameter.locked_in_period.is_some() {
                work_order_numbers.push((*work_order_number, ForcedWorkOrder::Locked));
            } else if tactical_work_order.is_tactical() {
                let first_day = tactical_work_order
                    .tactical_operations()?
                    .0
                    .iter()
                    .min_by(|ele1 , ele2| {
                        ele1.1.scheduled[0].0.date().date_naive().cmp(&ele2.1.scheduled[0].0.date().date_naive())
                    }).unwrap()
                    .1
                    .scheduled[0].0.date().date_naive();

                let tactical_period = self
                    .strategic_periods
                    .iter()
                    .find(|per| per.contains_date(first_day))
                    .expect("This result would come directly from the tactical agent. It should always find a Period in the Vec<Period>");

                work_order_numbers.push((*work_order_number, ForcedWorkOrder::FromTactical(tactical_period.clone())));
            } 
        }

        for forced_work_order_numbers in work_order_numbers {
            self.schedule_forced_work_order(&forced_work_order_numbers).with_context(|| format!("{:#?} could not be force scheduled", forced_work_order_numbers))?;
        }
        Ok(())
    }

    pub fn schedule_normal_work_order(
        &mut self,
        work_order_number: WorkOrderNumber,
        period: &Period,
    ) -> Result<Option<WorkOrderNumber>> {
        let strategic_parameter = self
            .strategic_parameters
            .strategic_work_order_parameters
            .get(&work_order_number)
            .unwrap()
            .clone();

        if period != self.periods().last().unwrap() {
            for (resource, resource_needed) in strategic_parameter.work_load.iter() {
                let resource_capacity: &Work = self.strategic_capacity(resource, period).with_context(|| format!("{} is missing state for {} in {}", std::any::type_name::<StrategicResources>(), resource, period))?;

                let loading_coming_from_the_tactical_agent = self.loaded_shared_solution.tactical.tactical_loadings.determine_period_load(resource, period).unwrap_or_default();

                let resource_loading: Work = self.strategic_loading(resource, period)? + &loading_coming_from_the_tactical_agent;

                if WorkOrderNumber(240350782) == work_order_number {
                    panic!();
                }
                if *resource_needed > resource_capacity - &resource_loading {
                    return Ok(Some(work_order_number));
                }

                if strategic_parameter.excluded_periods().contains(period) {
                    return Ok(Some(work_order_number));
                }

                if self.period_locks.contains(period) {
                    return Ok(Some(work_order_number));
                }
            }
        }

        *self
            .strategic_periods_mut()
            .get_mut(&work_order_number)
            .expect("An entry in the Strategic part of the SharedSolution is missing. This should never occur")
            = Some(period.clone());
        
        self.update_loadings(work_order_number, period.clone(), LoadOperation::Add);
        Ok(None)
    }

    pub fn schedule_forced_work_order(&mut self, force_schedule_work_order: &(WorkOrderNumber, ForcedWorkOrder)) -> Result<()> {
        
        if let Some(work_order_number) = self.is_scheduled(&force_schedule_work_order.0) {
            self.unschedule(*work_order_number).unwrap();
        }

        let locked_in_period = match &force_schedule_work_order.1 {
            ForcedWorkOrder::Locked => self
                .strategic_parameters
                .get_locked_in_period(&force_schedule_work_order.0).clone(),
            ForcedWorkOrder::FromTactical(period) => period.clone() ,
        };

        // Should the update loadings also be included here? I do not think that is a good idea.
        // What other things could we do?
        self.update_the_locked_in_period(&force_schedule_work_order.0, &locked_in_period)
            .with_context(|| format!("Could not fully update {:#?} in {}", force_schedule_work_order.0, &locked_in_period))?;

        self.update_loadings(force_schedule_work_order.0, locked_in_period.clone(), LoadOperation::Add);
        Ok(())
    }

    pub fn unschedule_random_work_orders(
        &mut self,
        number_of_work_orders: usize,
        rng: &mut impl rand::Rng,
    ) -> Result<()> {
        let strategic_periods = &self.strategic_solution.strategic_periods;

        let strategic_parameters = &self.strategic_parameters.strategic_work_order_parameters;

        let mut filtered_keys: Vec<_> = strategic_periods
            .iter()
            .filter(|(won, _)| strategic_parameters.get(won).unwrap().locked_in_period.is_none())
            .map(|(&won, _)| won)
            .collect();

        filtered_keys.sort();

        let sampled_work_order_keys = filtered_keys
            .choose_multiple(rng, number_of_work_orders)
            .collect::<Vec<_>>()
            .clone();

        // assert!(self.strategic_solution.scheduled_periods.values().all(|per| per.is_some()));
        for work_order_number in sampled_work_order_keys {
            self.unschedule(*work_order_number).with_context(|| format!("{:?} should always be present", work_order_number))?;
            self.populate_priority_queues();
        }
        Ok(())
    }

    fn is_scheduled<'a>(&'a self, work_order_number: &'a WorkOrderNumber) -> Option<&'a WorkOrderNumber> {
        self.strategic_periods()
            .get(work_order_number)
            .and_then(|scheduled_period| {
                scheduled_period
                    .as_ref()
                    .map(|_| work_order_number)
            })
    }

    fn update_loadings(&mut self, work_order_number: WorkOrderNumber, target_period: Period, load_operation: LoadOperation) {
        let work_load = self.strategic_parameters.strategic_work_order_parameters.get(&work_order_number).unwrap().work_load.clone();
        for (resource, periods) in self.strategic_solution.strategic_loadings.inner.iter_mut() {
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


impl LargeNeighborhoodSearch for StrategicAlgorithm {
    type BetterSolution = ();
    type SchedulingRequest = StrategicSchedulingRequest;
    type SchedulingResponse = StrategicResponseScheduling;
    type ResourceRequest = StrategicResourceRequest;
    type ResourceResponse = StrategicResponseResources;
    type TimeRequest = StrategicTimeRequest;
    type TimeResponse = StrategicResponsePeriods;
    type SchedulingUnit = WorkOrderNumber;
    
    fn calculate_objective_value(&mut self) -> Self::BetterSolution {

        let mut strategic_objective_value = StrategicObjectiveValue::new((1, 0), (10000000, 0)); 

        for (work_order_number, scheduled_period) in &self.strategic_solution.strategic_periods {
            let optimized_period = match scheduled_period {
                Some(optimized_period) => optimized_period.clone(),
                None => {
                    event!(Level::ERROR, "{:?} does not have a scheduled period", work_order_number);
                    panic!("{:?} does not have a scheduled period", work_order_number);
                }
            };

            let work_order_latest_allowed_finish_period =
                &self.strategic_parameters.strategic_work_order_parameters.get(work_order_number).expect("StrategicParameter should always be available for the StrategicSolution").latest_period;

            let period_difference = calculate_period_difference(
                optimized_period,
                work_order_latest_allowed_finish_period,
            );
            
            let period_penalty = std::cmp::max(period_difference, 0) as u64  
                * self
                    .strategic_parameters
                    .strategic_work_order_parameters
                    .get(work_order_number)
                    .unwrap()
                    .weight;

            strategic_objective_value.urgency.1 += period_penalty;
        }
        
        for (resource, periods) in &self.resources_capacities().inner {
            for (period, capacity) in &periods.0 {
                let loading = self
                    .resources_loading(resource, period);

                if loading - capacity > Work::from(0.0) {
                    strategic_objective_value.resource_penalty.1 += (loading - capacity).to_f64() as u64
                }
            }
        }

        strategic_objective_value.aggregate_objectives();

        self.strategic_solution.objective_value = strategic_objective_value;
    }

    #[instrument(level = "trace", skip_all)]
    fn schedule(&mut self) -> Result<()> {
        while !self.priority_queues.normal.is_empty() {
            for period in self.strategic_periods.clone() {
                let (work_order_number, weight) = match self.priority_queues.normal.pop() {
                    Some((work_order_number, weight)) => (work_order_number, weight),
                    None => {
                        break;
                    }
                };

                let inf_work_order_number =
                    self.schedule_normal_work_order(work_order_number, &period).with_context(|| format!("{:?} could not be scheduled normally", work_order_number))?;

                if let Some(work_order_number) = inf_work_order_number {
                    self.priority_queues.normal.push(work_order_number, weight);
                }
            }
        }
        Ok(())
    }
    
    fn unschedule(&mut self, work_order_number: Self::SchedulingUnit) -> Result<()> {
  
        // Why can this not unschedule?
        let strategic_period= self
            .strategic_periods_mut()
            .get_mut(&work_order_number)
            .with_context(|| format!("{:?}: was not present in the strategic periods", work_order_number))?;

        match strategic_period.take() {
            Some(unschedule_from_period) => {
                self.update_loadings(work_order_number, unschedule_from_period, LoadOperation::Sub);
                Ok(())
            }
            None => bail!("The strategic {:?} was not scheduled but StrategicAlgorithm.unschedule() was called on it.", work_order_number),
        }
    }

    fn update_resources_state(
        &mut self,
        strategic_resources_message: Self::ResourceRequest,
    ) -> Result<Self::ResourceResponse> 
    {
    //tracing::info!("update_resources_state called");
        match strategic_resources_message {
            StrategicResourceRequest::SetResources{resources, period_imperium, capacity} => {
                let mut count = 0;
                for resource in resources {
                    let period = self.strategic_periods.iter().find(|period| **period == period_imperium).expect("The period was not found in the self.periods vector. Somehow a message was sent form the frontend without the period being initialized correctly.");
                    self.strategic_parameters. 
                        strategic_capacity
                        .inner
                        .get_mut(&resource.clone())
                        .expect("The resource was not found in the self.resources_capacity vector. Somehow a message was sent form the frontend without the resource being initialized correctly.")
                        .0
                        .insert(period.clone(), Work::from(capacity));
                    count += 1;
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

                StrategicAlgorithm::assert_that_capacity_is_respected(loadings, capacities).context("Loadings exceed the capacities")?;
                Ok(StrategicResponseResources::Percentage(capacities.clone(), loadings.clone()))
            }
        }
    }

    #[allow(dead_code)]
    fn update_time_state(&mut self, _time_message: Self::TimeRequest) -> Result<Self::TimeResponse> 
        { todo!() }

    #[instrument(level = "info", skip_all)]
    fn update_scheduling_state(
        &mut self,
        strategic_scheduling_message: StrategicSchedulingRequest,
    ) -> Result<Self::SchedulingResponse>
{
        match strategic_scheduling_message {
            StrategicSchedulingRequest::Schedule(schedule_work_order) => {
                let period = self
                    .strategic_periods
                    .iter()
                    .find(|period| {
                        period.period_string() == schedule_work_order.period_string()
                    })
                    .cloned()
                    .with_context(|| format!("period: {:?} does not exist", schedule_work_order.period_string()))?;
    

                let mut number_of_work_orders = 0;
                for work_order_number in schedule_work_order.work_order_number {
                    let strategic_parameter = self.strategic_parameters.strategic_work_order_parameters.get_mut(&work_order_number).unwrap();
                    if strategic_parameter.excluded_periods.contains(&period) {
                        strategic_parameter.excluded_periods.remove(&period);
                    } 
                    self.strategic_parameters
                        .set_locked_in_period(work_order_number, period.clone()).context("could not set locked in period")?;
                    number_of_work_orders += 1;
                }

                Ok(StrategicResponseScheduling::new(number_of_work_orders, period))
                
            }
            StrategicSchedulingRequest::ExcludeFromPeriod(exclude_from_period) => {

                let period = self.strategic_periods.iter().find(|period| {
                    period.period_string() == exclude_from_period.period_string().clone()
                }).with_context(|| format!("{} was not found in the {}", exclude_from_period.period_string, type_name::<TimeEnvironment>()))?;
                
                let mut number_of_work_orders = 0;
                for work_order_number in exclude_from_period.work_order_number {
                    let strategic_parameter = 
                        self
                            .strategic_parameters
                            .strategic_work_order_parameters
                            .get_mut(&work_order_number)
                            .with_context(|| format!("The {:?} was not found in the {:#?}. The {:#?} should have been initialized at creation.", work_order_number, type_name::<StrategicParameter>(), type_name::<StrategicParameter>()))?;
            
                    assert!(!strategic_parameter.excluded_periods.contains(self.strategic_solution.strategic_periods.get(&work_order_number).as_ref().unwrap().as_ref().unwrap()));
                    dbg!(&strategic_parameter.excluded_periods);
                    strategic_parameter
                        .excluded_periods
                        .insert(period.clone());

                    dbg!(&strategic_parameter.excluded_periods);
                    // assert!(!strategic_parameter.excluded_periods.contains(self.strategic_solution.strategic_periods.get(&work_order_number).as_ref().unwrap().as_ref().unwrap()));

                    if let Some(locked_in_period) = &strategic_parameter.locked_in_period {
                        if strategic_parameter.excluded_periods.contains(locked_in_period) {
                            strategic_parameter.locked_in_period = None;
                            event!(Level::INFO, "{:?} has been excluded from period {} and the locked in period has been removed", work_order_number, period.period_string());
                        }
                    }

                    let last_period = self.strategic_periods.iter().last().cloned();
                    self.strategic_solution.strategic_periods.insert(work_order_number, last_period);

                    assert!(!strategic_parameter.excluded_periods.contains(self.strategic_solution.strategic_periods.get(&work_order_number).as_ref().unwrap().as_ref().unwrap()));
                    number_of_work_orders += 1;
                }

                Ok(StrategicResponseScheduling::new(number_of_work_orders, period.clone()))
            }
        }
    }
}

impl StrategicAlgorithm {
    pub fn new(
        priority_queues: PriorityQueues<WorkOrderNumber, u64>,
        strategic_parameters: StrategicParameters,
        strategic_tactical_solution_arc_swap: Arc<ArcSwapSharedSolution>,
        period_locks: HashSet<Period>,
        periods: Vec<Period>,
    ) -> Self {

        let loaded_shared_solution = strategic_tactical_solution_arc_swap.0.load();

        StrategicAlgorithm {
            priority_queues,
            strategic_parameters,
            strategic_solution: StrategicSolution::default() ,
            arc_swap_shared_solution: strategic_tactical_solution_arc_swap,
            loaded_shared_solution,
            strategic_periods: periods,
            period_locks,
            
        }
    }

    pub fn resources_loadings(&self) -> &StrategicResources {
        &self.strategic_solution.strategic_loadings
    }

    pub fn resources_loading(&self, resource: &Resources, period: &Period) -> &Work {
        self.strategic_solution.strategic_loadings.inner.get(resource).unwrap().0.get(period).unwrap()
    }

    pub fn resources_capacities(&self) -> &StrategicResources {
        &self.strategic_parameters.strategic_capacity
    }


    pub fn periods(&self) -> &Vec<Period> {
        &self.strategic_periods
    }

    pub fn populate_priority_queues(&mut self) {
        for work_order_number in self.strategic_solution.strategic_periods.keys() {
            event!(Level::TRACE, "Work order {:?} has been added to the normal queue", work_order_number);
            let strategic_work_order_weight = self.strategic_parameters.strategic_work_order_parameters.get(work_order_number).expect("The StrategicParameter should always be available for the StrategicSolution").weight;

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
    use arc_swap::ArcSwap;
    use strategic_parameters::StrategicParameter;
    use shared_types::strategic::{strategic_request_scheduling_message::ScheduleChange, Periods};
    use chrono::{Duration, TimeZone, Utc};
    use rand::{rngs::StdRng, SeedableRng};

    use shared_types::scheduling_environment::worker_environment::resources::Resources;

    use crate::agents::{strategic_agent::algorithm::{
            PriorityQueues, StrategicAlgorithm, StrategicParameters
        }, TacticalSolutionBuilder, WhereIsWorkOrder};

    use std::{collections::HashMap, str::FromStr};

    use shared_types::scheduling_environment::{
        work_order::WorkOrder,
            WorkOrders,
    };

    impl StrategicAlgorithm {
        
    pub fn strategic_parameter(&self, work_order_number: &WorkOrderNumber) -> Option<&StrategicParameter> {
        self.strategic_parameters.strategic_work_order_parameters.get(work_order_number)
    }
    }
    #[test]
    fn test_update_strategic_algorithm_state() {
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

        let mut strategic_algorithm = StrategicAlgorithm::new(
            
            PriorityQueues::new(),
            StrategicParameters::new(HashMap::new(), resource_capacity),
            ArcSwapSharedSolution::default().into(),
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

        strategic_algorithm.set_strategic_parameter(work_order_number, optimized_work_order);

        let strategic_scheduling_message = StrategicSchedulingRequest::Schedule(
            ScheduleChange::new(vec![work_order_number], "2023-W47-48".to_string()),
        );

        let strategic_resources_message = 
            StrategicResourceRequest::SetResources { resources: vec![Resources::MtnMech] , period_imperium: period.clone(), capacity:  300.0};

        assert_eq!(
            strategic_algorithm.strategic_parameters.strategic_capacity.inner
                .get(&Resources::MtnMech)
                .unwrap()
                .0
                .get(&period)
            ,
            Some(&Work::from(150.0))
        );
        assert_eq!(
            strategic_algorithm
                .strategic_parameters
                .strategic_work_order_parameters
                .get(&work_order_number),
            Some(&StrategicParameter::new(
                Some(period.clone()),
                HashSet::new(),
                period.clone(),
                1000,
                HashMap::new()
            ))
        );

        strategic_algorithm.update_scheduling_state(strategic_scheduling_message).unwrap();
        strategic_algorithm.update_resources_state(strategic_resources_message).unwrap();

        assert_eq!(
            strategic_algorithm.strategic_parameters.strategic_capacity.inner
                .get(&Resources::MtnMech)
                .unwrap()
                .0
                .get(&period)
            ,
            Some(&Work::from(300.0))
        );
        assert_eq!(
            strategic_algorithm
                .strategic_periods()
                .get(&work_order_number),
            None
        );
        assert_eq!(
            strategic_algorithm
                .strategic_parameters
                .strategic_work_order_parameters
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

        let mut strategic_parameters = StrategicParameters::new(HashMap::new(), StrategicResources::new(resource_capacity));

        let strategic_parameter =
            StrategicParameter::new( None, HashSet::new(), period.clone(), 1000, HashMap::new());



        strategic_parameters.insert_strategic_parameter(work_order_number, strategic_parameter);
        let mut strategic_algorithm = StrategicAlgorithm::new(
            PriorityQueues::new(),
            strategic_parameters,
            ArcSwapSharedSolution::default().into(),
            HashSet::new(),
            vec![period.clone()],
        );
        strategic_algorithm.strategic_solution.strategic_periods.insert(work_order_number, None);
        strategic_algorithm.strategic_solution.strategic_loadings = StrategicResources::new(resource_loadings);

        strategic_algorithm.schedule_normal_work_order(work_order_number, &period).unwrap();

        assert_eq!(
            *strategic_algorithm
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


        let optimized_work_order =
            StrategicParameter::new( None, HashSet::new(), period.clone(), 1000, work_load);


        let mut resource_capacity = HashMap::new();
        let mut resource_loadings = HashMap::new();

        let mut period_hash_map_0 = Periods(HashMap::new());

        period_hash_map_0.insert(period.clone(),Work::from( 0.0));

        resource_capacity.insert(Resources::MtnMech, period_hash_map_0.clone());
        resource_loadings.insert(Resources::MtnMech, period_hash_map_0.clone());

        let mut strategic_parameters = StrategicParameters::new(HashMap::new(), StrategicResources::new(resource_capacity));
        strategic_parameters
            .strategic_work_order_parameters
            .insert(work_order_number, optimized_work_order);

        let mut strategic_algorithm = StrategicAlgorithm::new(
            PriorityQueues::new(),
            strategic_parameters,
            ArcSwapSharedSolution::default().into(),
            HashSet::new(),
            vec![period.clone()],
        );

        strategic_algorithm.strategic_solution.strategic_periods.insert(work_order_number, None);
        strategic_algorithm.strategic_solution.strategic_loadings = StrategicResources::new(resource_loadings);
        strategic_algorithm.schedule_normal_work_order(work_order_number, &period).unwrap();

        assert_eq!(
            *strategic_algorithm
            .strategic_periods()
                .get(&work_order_number)
                .unwrap(),
            strategic_algorithm.periods().last().cloned()
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

        let mut strategic_algorithm = StrategicAlgorithm::new(
            PriorityQueues::new(),
            StrategicParameters::new(HashMap::new(), StrategicResources::new(resource_capacity)),
            ArcSwapSharedSolution::default().into(),
            HashSet::new(),
            vec![],
        );

        strategic_algorithm.strategic_solution.strategic_loadings = StrategicResources::new(resource_loadings);

        let work_order_number = WorkOrderNumber(2100000001);
        
        let work_order = StrategicParameter::new(
            Some(period.clone()),
            HashSet::new(),
            period.clone(),
            1000,
            work_load,
        );

        strategic_algorithm.strategic_parameters.strategic_work_order_parameters.insert(work_order_number, work_order);
        strategic_algorithm.update_loadings(work_order_number, period.clone(), LoadOperation::Add);

        assert_eq!(
            *strategic_algorithm
                .resources_loading(&Resources::MtnMech, &period),
           Work::from( 20.0)
        );
        assert_eq!(
            *strategic_algorithm
                .resources_loading(&Resources::MtnElec, &period),
           Work::from( 40.0)
        );
        assert_eq!(
            *strategic_algorithm
                .resources_loading(&Resources::Prodtech, &period),
           Work::from( 60.0)
        );

        assert!(
            !strategic_algorithm
                .strategic_solution
                .strategic_loadings
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

        let mut strategic_parameters = StrategicParameters::new(HashMap::new(), StrategicResources::new(resource_capacity));

        let strategic_parameter = StrategicParameter::new(
            Some(period_1.clone()),
            HashSet::new(),
            period_1.clone(),
            1000,
            work_load,
        );

        strategic_parameters
            .strategic_work_order_parameters
            .insert(work_order_number, strategic_parameter);

        let mut strategic_algorithm = StrategicAlgorithm::new(
            PriorityQueues::new(),
            strategic_parameters,
            ArcSwapSharedSolution::default().into(),
            HashSet::new(),
            periods,
        );

        strategic_algorithm.strategic_solution.strategic_loadings = StrategicResources::new(resource_loadings);
        strategic_algorithm.strategic_solution.strategic_periods.insert(work_order_number, None);
        
        assert_eq!(
            *strategic_algorithm
                .resources_loading(&Resources::MtnMech, &period_1),
           Work::from( 0.0)
        );
        strategic_algorithm.schedule_normal_work_order(work_order_number, &period_1).unwrap();

        assert_eq!(
            *strategic_algorithm
                .resources_loading(&Resources::MtnMech, &period_1),
           Work::from( 20.0)
        );

        assert_eq!(
            *strategic_algorithm
                .resources_loading(&Resources::MtnElec, &period_1),
           Work::from( 40.0)
        );
        assert_eq!(
            *strategic_algorithm
                .resources_loading(&Resources::Prodtech, &period_1),
           Work::from( 60.0)
        );

        strategic_algorithm.unschedule(work_order_number).unwrap();
        assert_eq!(
            *strategic_algorithm
                .resources_loading(&Resources::MtnMech, &period_1),
           Work::from( 0.0)
        );
        assert_eq!(
            *strategic_algorithm
                .resources_loading(&Resources::MtnElec, &period_1),
           Work::from( 0.0)
        );
        assert_eq!(
            *strategic_algorithm
                .resources_loading(&Resources::Prodtech, &period_1),
           Work::from( 0.0)
        );

        strategic_algorithm.schedule_forced_work_order(&(work_order_number, ForcedWorkOrder::Locked)).unwrap();

        assert_eq!(
            *strategic_algorithm
                .resources_loading(&Resources::MtnMech, &period_1),
           Work::from( 20.0)
        );
        assert_eq!(
            *strategic_algorithm
                .resources_loading(&Resources::MtnElec, &period_1),
           Work::from( 40.0)
        );
        assert_eq!(
            *strategic_algorithm
                .resources_loading(&Resources::Prodtech, &period_1),
           Work::from( 60.0)
        );

        strategic_algorithm
            .strategic_parameters
            .set_locked_in_period(work_order_number, period_2.clone()).context("could not set locked in period").expect("test failed");
        strategic_algorithm.schedule_forced_work_order(&(work_order_number, ForcedWorkOrder::Locked)).unwrap();

        assert_eq!(
            *strategic_algorithm
                .resources_loading(&Resources::MtnMech, &period_1),
           Work::from( 0.0)
        );
        assert_eq!(
            *strategic_algorithm
                .resources_loading(&Resources::MtnElec, &period_1),
           Work::from( 0.0)
        );
        assert_eq!(
            *strategic_algorithm
                .resources_loading(&Resources::Prodtech, &period_1),
           Work::from( 0.0)
        );

        assert_eq!(
            *strategic_algorithm
                .resources_loading(&Resources::MtnMech, &period_2),
           Work::from( 20.0)
        );
        assert_eq!(
            *strategic_algorithm
                .resources_loading(&Resources::MtnElec, &period_2),
           Work::from( 40.0)
        );
        assert_eq!(
            *strategic_algorithm
                .resources_loading(&Resources::Prodtech, &period_2),
           Work::from( 60.0)
        );

        strategic_algorithm.unschedule(work_order_number).unwrap();
        assert_eq!(
            *strategic_algorithm
                .resources_loading(&Resources::MtnMech, &period_1),
           Work::from( 0.0)
        );
        assert_eq!(
            *strategic_algorithm
                .resources_loading(&Resources::MtnElec, &period_1),
           Work::from( 0.0)
        );
        assert_eq!(
            *strategic_algorithm
                .resources_loading(&Resources::Prodtech, &period_1),
           Work::from( 0.0)
        );

        assert_eq!(
            *strategic_algorithm
                .resources_loading(&Resources::MtnMech, &period_1),
           Work::from( 0.0)
        );
        assert_eq!(
            *strategic_algorithm
                .resources_loading(&Resources::MtnElec, &period_1),
           Work::from( 0.0)
        );
        assert_eq!(
            *strategic_algorithm
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

        let mut strategic_parameters = StrategicParameters::new(HashMap::new(), StrategicResources::default());

        let strategic_parameter_1 = StrategicParameter::new(
            None,
            HashSet::new(),
Period::from_str("2023-W49-50").unwrap(),
            1000,
            work_load_1,
        );

        let strategic_parameter_2 = StrategicParameter::new(
            None,
            HashSet::new(),
Period::from_str("2023-W49-50").unwrap(),
            1000,
            work_load_2,
        );

        let strategic_parameter_3 = StrategicParameter::new(
            None,
            HashSet::new(),
Period::from_str("2023-W49-50").unwrap(),
            1000,
            work_load_3,
        );

        let work_order_number_1 = WorkOrderNumber(2200000001);
        let work_order_number_2 = WorkOrderNumber(2200000002);
        let work_order_number_3 = WorkOrderNumber(2200000003);

        strategic_parameters
            .strategic_work_order_parameters
            .insert(work_order_number_1, strategic_parameter_1);

        strategic_parameters
            .strategic_work_order_parameters
            .insert(work_order_number_2, strategic_parameter_2);

        strategic_parameters
            .strategic_work_order_parameters
            .insert(work_order_number_3, strategic_parameter_3);

        let periods: Vec<Period> = vec![
            Period::from_str("2023-W47-48").unwrap(),
            Period::from_str("2023-W49-50").unwrap(),
        ];

        let mut strategic_algorithm = StrategicAlgorithm::new(
            PriorityQueues::new(),
            strategic_parameters,
            ArcSwapSharedSolution::default().into(),
            HashSet::new(),
            periods.clone(),
        );

        strategic_algorithm.strategic_solution.strategic_periods.insert(work_order_number_1, Some(periods[0].clone()));
        strategic_algorithm.strategic_solution.strategic_periods.insert(work_order_number_2, Some(periods[1].clone()));
        strategic_algorithm.strategic_solution.strategic_periods.insert(work_order_number_3, Some(periods[1].clone()));

        let seed: [u8; 32] = [
            1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16, 17, 18, 19, 20, 21, 22, 23, 24,
            25, 26, 27, 28, 29, 30, 31, 32,
        ];

        let mut rng = StdRng::from_seed(seed);

        strategic_algorithm.unschedule_random_work_orders(2, &mut rng).expect("It should always be possible to unschedule random work orders in the strategic agent");

        assert_eq!(
            *strategic_algorithm
                .strategic_periods()
                .get(&WorkOrderNumber(2200000001))
                .unwrap(),
            Some(Period::from_str("2023-W47-48").unwrap())
        );

        assert_eq!(
            *strategic_algorithm
                .strategic_periods()
                .get(&WorkOrderNumber(2200000002))
                .unwrap(),
            None
        );

        assert_eq!(

            *strategic_algorithm
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
        let mut strategic_parameters = StrategicParameters::new(HashMap::new(), StrategicResources::default());

        let strategic_parameter = StrategicParameter::new(
            None,
            HashSet::new(),
            Period::from_str("2023-W47-48").unwrap(),
            1000,
            HashMap::new(),
        );

        strategic_parameters
            .strategic_work_order_parameters
            .insert(work_order_number, strategic_parameter);

        let tactical_solution_builder = TacticalSolutionBuilder::new();
       
        let mut tactical_days = HashMap::new();
        tactical_days.insert(work_order_number, WhereIsWorkOrder::NotScheduled);
        
        let tactical_solution = tactical_solution_builder.with_tactical_days(tactical_days).build();

        let shared_solution = SharedSolution {
            tactical: tactical_solution,
            ..SharedSolution::default()
        };

        let arc_swap_shared_solution = ArcSwapSharedSolution(ArcSwap::from_pointee(shared_solution));

        let mut strategic_algorithm = StrategicAlgorithm::new(
            PriorityQueues::new(),
            strategic_parameters,
            arc_swap_shared_solution.into(),
            HashSet::new(),
            vec![Period::from_str("2023-W47-48").unwrap()],
        );

        strategic_algorithm
            .strategic_solution
            .strategic_periods
            .insert(work_order_number, Some(Period::from_str("2024-W41-42").unwrap()));

        strategic_algorithm.schedule_forced_work_orders().unwrap();

        strategic_algorithm.unschedule(work_order_number).unwrap();
        assert_eq!(
            *strategic_algorithm
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
        pub fn set_strategic_parameter(
            &mut self,
            work_order_number: WorkOrderNumber,
            optimized_work_order: StrategicParameter,
        ) {
            self.strategic_parameters
                .strategic_work_order_parameters
                .insert(work_order_number, optimized_work_order);
        }
    }
}
            
