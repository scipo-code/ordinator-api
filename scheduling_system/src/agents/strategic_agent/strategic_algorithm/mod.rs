pub mod optimized_work_orders;

use std::collections::{HashMap, HashSet};
use std::fmt::Display;
use std::hash::Hash;
use shared_messages::Asset;
use tracing::{error, info, instrument, trace};
use rand::prelude::SliceRandom;

use priority_queue::PriorityQueue;
use shared_messages::agent_error::AgentError;
use shared_messages::strategic::strategic_periods_message::StrategicTimeMessage;
use shared_messages::strategic::strategic_resources_message::StrategicResourceMessage;
use shared_messages::strategic::strategic_scheduling_message::StrategicSchedulingMessage;

use crate::agents::LoadOperation;
use crate::agents::traits::LargeNeighborHoodSearch;
use crate::models::WorkOrders;
use crate::models::time_environment::period::Period;
use shared_messages::resources::Resources;

use self::optimized_work_orders::{OptimizedWorkOrder, OptimizedWorkOrders, StrategicResources};



#[derive(Debug, Clone)]
pub struct StrategicAlgorithm {
    objective_value: f64,
    resources_capacity: StrategicResources,
    resources_loading: StrategicResources,
    priority_queues: PriorityQueues<u32, u32>,
    optimized_work_orders: OptimizedWorkOrders,
    period_locks: HashSet<Period>,
    periods: Vec<Period>,
}

impl StrategicAlgorithm {
    #[allow(dead_code)]
    pub fn optimized_work_order(&self, work_order_number: &u32) -> Option<&OptimizedWorkOrder> {
        self.optimized_work_orders.inner.get(work_order_number)
    }

    pub fn objective_value(&self) -> f64 {
        self.objective_value
    }

    pub fn set_periods(&mut self, periods: Vec<Period>) {
        self.periods = periods;
    }

    pub fn tactical_work_orders(&self, tactical_periods: Vec<Period>) -> Vec<(u32, Period)> {
        
        
        let mut tactical_work_orders: Vec<(u32, Period)> = vec![];

        for (work_order_number, optimized_work_order) in &self.optimized_work_orders.inner {
            match optimized_work_order.scheduled_period.clone() {
                Some(period) => {
                    if tactical_periods.contains(&period) {
                        tactical_work_orders.push((*work_order_number, period));
                    }
                }
                None => {
                    panic!("Work order number {} does not have a scheduled period", work_order_number)
                }
            }
        }
        tactical_work_orders
    }

pub fn create_optimized_work_orders(
    &mut self,
    work_orders: &mut WorkOrders,
    periods: &[Period],
    asset: &Asset,
) {
    for (work_order_number, work_order) in work_orders.inner.iter() {
        if &work_order.functional_location().asset != asset {
            continue;
        }

        let optimized_work_order = OptimizedWorkOrder::builder().build_from_work_order(work_order, periods).build();
        
        let scheduled_period = optimized_work_order.scheduled_period.clone().unwrap();
        self.optimized_work_orders.insert_optimized_work_order(*work_order_number, optimized_work_order);

        self.update_loadings(*work_order_number, &scheduled_period, LoadOperation::Add);
    }
}
}

impl StrategicAlgorithm {
    #[instrument(level = "trace", skip_all)]
    pub fn schedule_forced_work_orders(&mut self) {
        let mut work_order_numbers: Vec<u32> = vec![];
        for (work_order_number, opt_work_order) in self.optimized_work_orders().iter() {
            if opt_work_order.locked_in_period.is_some() {
                work_order_numbers.push(*work_order_number);
            }
        }

        for work_order_number in work_order_numbers {
            self.schedule_forced_work_order(work_order_number);
        }
        self.calculate_objective_value();
    }

    #[instrument(level = "trace", skip_all)]
    pub fn schedule_normal_work_order(
        &mut self,
        work_order_number: u32,
        period: &Period,
    ) -> Option<u32> {
        let optimized_work_order = self
            .optimized_work_orders
            .inner
            .get(&work_order_number)
            .unwrap()
            .clone();

        if period != self.periods().last().unwrap() {
            for (resource, resource_needed) in optimized_work_order.work_load.iter() {
                let resource_capacity: &f64 = self
                    .resources_capacity
                    .inner
                    .get(&resource.clone())
                    .unwrap()
                    .get(&period.clone())
                    .unwrap();

                let resource_loading: &f64 = self
                    .resources_loading
                    .inner
                    .get(&resource.clone())
                    .unwrap()
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
        match self.optimized_work_orders.inner.get_mut(&work_order_number) {
            Some(optimized_work_order) => {
                optimized_work_order.set_scheduled_period(Some(period.clone()));
            }
            None => {
                panic!(
                    "The work order is not found in the optimized work orders. Should have been
                initialized"
                )
            }
        }
        self.update_loadings(work_order_number, period, LoadOperation::Add);
        None
    }

    #[instrument(level = "trace", skip_all)]
    pub fn schedule_forced_work_order(&mut self, work_order_number: u32) {
        if let Some(work_order_number) = self.is_scheduled(work_order_number) {
            self.unschedule(work_order_number);
        }

        let target_period = self
            .optimized_work_orders
            .get_locked_in_period(work_order_number);

        self.optimized_work_orders
            .set_scheduled_period(work_order_number, target_period.clone());

        self.update_loadings(work_order_number, &target_period, LoadOperation::Add);
    }

    #[instrument(level = "trace", skip_all)]
    pub fn unschedule_random_work_orders(
        &mut self,
        number_of_work_orders: usize,
        rng: &mut impl rand::Rng,
    ) {
        let optimized_work_orders = self.optimized_work_orders();

        let mut filtered_keys: Vec<_> = optimized_work_orders
            .iter()
            .filter(|(&_key, value)| value.locked_in_period.is_none())
            .map(|(&key, _)| key)
            .collect();

        filtered_keys.sort();

        let sampled_work_order_keys = filtered_keys
            .choose_multiple(rng, number_of_work_orders)
            .collect::<Vec<_>>()
            .clone();

        for work_order_key in sampled_work_order_keys {
            self.unschedule(*work_order_key);
            self.populate_priority_queues();
        }
    }

    fn is_scheduled(&self, work_order_number: u32) -> Option<u32> {
        self.optimized_work_orders
            .inner
            .get(&work_order_number)
            .and_then(|optimized_work_order| {
                optimized_work_order
                    .scheduled_period
                    .as_ref()
                    .map(|_| work_order_number)
            })
    }

    fn update_loadings(&mut self, work_order_number: u32, target_period: &Period, load_operation: LoadOperation) {
        let work_load = self.optimized_work_orders.inner.get(&work_order_number).unwrap().work_load.clone();
        for (resource, periods) in self.resources_loading.inner.iter_mut() {
            for (period, loading) in periods {
                if period == target_period {
                    match load_operation {
                        LoadOperation::Add => *loading += work_load.get(resource).unwrap_or(&0.0),
                        LoadOperation::Sub => *loading -= work_load.get(resource).unwrap_or(&0.0),
                    }
                }
            }
        }
    }
}

pub fn calculate_period_difference(scheduled_period: Period, latest_period: Option<Period>) -> i64 {
    let scheduled_period_date = scheduled_period.end_date().to_owned();
    let latest_period_date = match latest_period.clone() {
        Some(period) => period.end_date().to_owned(),
        None => scheduled_period_date,
    };

    let duration = scheduled_period_date.signed_duration_since(latest_period_date);
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
            self.resources_capacity,
            self.resources_loading,
            self.priority_queues,
            self.optimized_work_orders,
            self.periods
        )
    }
}

impl LargeNeighborHoodSearch for StrategicAlgorithm {

    type SchedulingMessage = StrategicSchedulingMessage;
    type ResourceMessage = StrategicResourceMessage;
    type TimeMessage = StrategicTimeMessage;

    type Error = AgentError;
    
    #[instrument(level = "trace", skip_all)]
    fn calculate_objective_value(&mut self) {
        let mut period_penalty_contribution: f64 = 0.0;
        let mut excess_penalty_contribution: f64 = 0.0;

        for (work_order_number, optimized_work_order) in &self.optimized_work_orders.inner {
            let optimized_period = match &optimized_work_order.scheduled_period {
                Some(optimized_period) => optimized_period.clone(),
                None => {
                    error!("Work order number {} does not have a scheduled period", work_order_number);
                    panic!("Work order number {} does not have a scheduled period", work_order_number);
                }
            };

            let work_order_latest_allowed_finish_period =
                optimized_work_order.latest_period().clone();

            let period_difference = calculate_period_difference(
                optimized_period,
                work_order_latest_allowed_finish_period,
            );
            let period_penalty = std::cmp::max(period_difference, 0) as f64
                * self
                    .optimized_work_orders
                    .inner
                    .get(work_order_number)
                    .unwrap()
                    .weight() as f64;

            period_penalty_contribution += period_penalty;
        }

        for (resource, periods) in &self.resources_capacities().inner {
            for (period, capacity) in periods {
                let loading = self
                    .resources_loadings()
                    .inner
                    .get(resource)
                    .unwrap()
                    .get(period)
                    .unwrap();
                if *loading > *capacity {
                    excess_penalty_contribution += loading - capacity;
                }
            }
        }

        self.objective_value =
            period_penalty_contribution + 10000000.0 * excess_penalty_contribution;
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
    
    #[instrument(level = "trace", skip_all)]
    fn unschedule(&mut self, work_order_number: u32) {
  
        let optimized_work_order: &mut OptimizedWorkOrder = self
            .optimized_work_orders
            .inner
            .get_mut(&work_order_number)
            .unwrap();

        match optimized_work_order.scheduled_period_mut().take() {
            Some(target_period) => {
                self.update_loadings(work_order_number, &target_period, LoadOperation::Sub);
            }
            None => panic!(),
        }
    }

    #[instrument(skip_all, level = "info")]
    fn update_resources_state(
        &mut self,
        strategic_resources_message: StrategicResourceMessage,
    ) -> Result<String, AgentError> 
    {
    //tracing::info!("update_resources_state called");
        match strategic_resources_message {
            StrategicResourceMessage::SetResources(manual_resources) => {
                let mut count = 0;
                for (resource, periods) in manual_resources {
                    for (period_string, capacity) in periods {
                        let period = self.periods.iter().find(|period| period.period_string() == period_string).expect("The period was not found in the self.periods vector. Somehow a message was sent form the frontend without the period being initialized correctly.");
                        self.resources_capacity
                            .inner
                            .get_mut(&resource.clone())
                            .expect("The resource was not found in the self.resources_capacity vector. Somehow a message was sent form the frontend without the resource being initialized correctly.")
                            .insert(period.clone(), capacity);
                        count += 1;
                    }
                }
                let response_message = format!("{} resources-period pairs updated correctly", count);
                Ok(response_message)
            }
            StrategicResourceMessage::GetLoadings {
                periods_end,
                select_resources: _,
            } => {
                let loading = self.resources_loadings();

                let periods_end: u32 = periods_end.parse().unwrap();

                Ok(loading.to_string(periods_end))
            }
            StrategicResourceMessage::GetCapacities { periods_end, select_resources: _ } => 
            {         
                let capacities = self.resources_capacities();

                let periods_end: u32 = periods_end.parse().unwrap();

                Ok(capacities.to_string(periods_end))
            }
            StrategicResourceMessage::GetPercentageLoadings { periods_end, resources: _ } => {
                let periods_end: u32 = periods_end.parse().unwrap();
                let capacities = self.resources_capacities();
                let loadings = self.resources_loadings();

                let mut percentage_loading = HashMap::<Resources, HashMap<Period, f64>>::new();

                for (resource, periods) in &capacities.inner {
                    if percentage_loading.get(resource).is_none() {
                        percentage_loading.insert(resource.clone(), HashMap::<Period, f64>::new());
                    }
                    for (period, capacity) in periods {
                        let percentage: f64 = (loadings.inner.get(resource).unwrap().get(period).unwrap() / capacity * 100.0).round();
                        percentage_loading.get_mut(resource).unwrap().insert(period.clone(), percentage);
                    }
                }

                let algorithm_resources = StrategicResources::new(percentage_loading );
                Ok(algorithm_resources.to_string(periods_end))
            }
        }
    }



    #[allow(dead_code)]
    fn update_time_state(&mut self, _time_message: StrategicTimeMessage) -> Result<String, AgentError> 
        { todo!() }

    #[instrument(level = "info", skip_all)]
    fn update_scheduling_state(
        &mut self,
        strategic_scheduling_message: StrategicSchedulingMessage,
    ) -> Result<String, AgentError>
{
        match strategic_scheduling_message {
            StrategicSchedulingMessage::Schedule(schedule_work_order) => {
                let work_order_number = schedule_work_order.get_work_order_number();
                let period = self
                    .periods
                    .iter()
                    .find(|period| {
                        period.period_string() == schedule_work_order.get_period_string()
                    })
                    .cloned();
    
                match period {
                    Some(period) => {
                        self.optimized_work_orders
                            .set_locked_in_period(work_order_number, period.clone());
             
                        Ok(format!(
                            "Work order {} has been scheduled for period {}",
                            work_order_number,
                            period.period_string()
                        ))
                    }
                    None => Err(AgentError::StateUpdateError("Could not update strategic scheduling state".to_string())),
                }
            }
            StrategicSchedulingMessage::ScheduleMultiple(schedule_work_orders) => {
                let mut output_string = String::new();
                for schedule_work_order in schedule_work_orders {
                    let work_order_number = schedule_work_order.get_work_order_number();
                    let period = self
                        .periods
                        .iter()
                        .find(|period| {
                            period.period_string() == schedule_work_order.get_period_string()
                        })
                        .cloned();

                    match period {
                        Some(period) => {
                            self.optimized_work_orders
                                .set_locked_in_period(work_order_number, period.clone());

                            output_string += &format!(
                                "Work order {} has been scheduled for period {}",
                                work_order_number,
                                period.clone().period_string()
                            )
                            .to_string();
                        }
                        None => {
                   
                            return Err(AgentError::StateUpdateError("The period was not found in the self.periods vector. Somehow a message was sent form the frontend without the period being initialized correctly.".to_string()))
                        }
                    }
                }
        
                Ok(output_string.to_string())
                 
            }
            StrategicSchedulingMessage::ExcludeFromPeriod(exclude_from_period) => {
                let work_order_number = exclude_from_period.get_work_order_number();
                let period = self.periods.iter().find(|period| {
                    period.period_string() == exclude_from_period.get_period_string().clone()
                });

                match period {
                    Some(period) => {
                        let optimized_work_order = 
                            self
                                .optimized_work_orders
                                .inner
                                .get_mut(&work_order_number)
                                .expect("The work order number was not found in the optimized work orders. The work order should have been initialized at the outset.");
                        
                        optimized_work_order
                            .excluded_periods
                            .insert(period.clone());

                        let overwrite = if let Some(locked_in_period) = &optimized_work_order.locked_in_period {
                            if optimized_work_order.excluded_periods.contains(locked_in_period) {
                                optimized_work_order.scheduled_period = None;
                                optimized_work_order.locked_in_period = None;
                                true
                            } else {
                                false
                            }
                        } else {
                            false
                        };

                        if overwrite {
                            info!("Work order {} has been excluded from period {} and the locked in period has been removed", work_order_number, period.period_string());
                        }
                            
                        Ok(format!(
                            "Work order {} has been excluded from period {}",
                            work_order_number,
                            period.period_string()
                        ))
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
        priority_queues: PriorityQueues<u32, u32>,
        optimized_work_orders: OptimizedWorkOrders,
        period_locks: HashSet<Period>,
        periods: Vec<Period>,
    ) -> Self {
        StrategicAlgorithm {
            objective_value,
            resources_capacity,
            resources_loading,
            priority_queues,
            optimized_work_orders,
            periods,
            period_locks,
            
        }
    }

    pub fn optimized_work_orders(&self) -> &HashMap<u32, OptimizedWorkOrder> {
        &self.optimized_work_orders.inner
    }

    pub fn resources_loadings(&self) -> &StrategicResources {
        &self.resources_loading
    }

    pub fn resources_capacities(&self) -> &StrategicResources {
        &self.resources_capacity
    }


    pub fn periods(&self) -> &Vec<Period> {
        &self.periods
    }

    pub fn populate_priority_queues(&mut self) {
        for (key, work_order) in self.optimized_work_orders.inner.iter() {
            trace!("Work order {} has been added to the normal queue", key);
            if work_order.scheduled_period.is_none() {
                self.priority_queues
                    .normal
                    .push(*key, work_order.weight());
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
    pub unloading: PriorityQueue<T, P>,
    pub shutdown_vendor: PriorityQueue<T, P>,
    pub normal: PriorityQueue<T, P>,
}

impl PriorityQueues<u32, u32> {
    pub fn new() -> Self {
        Self {
            unloading: PriorityQueue::<u32, u32>::new(),
            shutdown_vendor: PriorityQueue::<u32, u32>::new(),
            normal: PriorityQueue::<u32, u32>::new(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use shared_messages::strategic::strategic_scheduling_message::SingleWorkOrder;
    use chrono::{Duration, TimeZone, Utc};
    use rand::{rngs::StdRng, SeedableRng};

    use shared_messages::resources::Resources;


    use std::collections::HashMap;

    use crate::{
        agents::strategic_agent::strategic_algorithm::{
            OptimizedWorkOrders, PriorityQueues, StrategicAlgorithm,
        },
        models::{
            work_order::WorkOrder,
            
            WorkOrders,
        },
    };

    #[test]
    fn test_update_scheduler_algorithm_state() {
        let mut work_orders = WorkOrders::new();

        let work_order = WorkOrder::default(     );

        work_orders.insert(work_order.clone());

        let period = Period::new_from_string("2023-W47-48").unwrap();
        let periods = vec![period.clone()];

        let mut manual_resource_capacity: HashMap<Resources, HashMap<Period, f64>> = HashMap::new();

        let mut hash_map_periods_150 = HashMap::new();

        hash_map_periods_150.insert(period.clone(), 150.0);


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
            OptimizedWorkOrders::new(HashMap::new()),
            HashSet::new(),
            periods,
        );

        let optimized_work_order =
            OptimizedWorkOrder::new(
                None, 
                Some(period.clone()), 
                HashSet::new(),
                None,
                1000,
                HashMap::new()
                );

        scheduler_agent_algorithm.set_optimized_work_order(2200002020, optimized_work_order);

        let strategic_scheduling_message = StrategicSchedulingMessage::Schedule(
            SingleWorkOrder::new(2200002020, "2023-W47-48".to_string()),
        );
        let strategic_resources_message = StrategicResourceMessage::new_test();

        assert_eq!(
            scheduler_agent_algorithm.resources_capacity.inner
                .get(&Resources::MtnMech)
                .unwrap()
                .get(&period)
            ,
            Some(&150.0)
        );
        assert_eq!(
            scheduler_agent_algorithm
                .optimized_work_orders
                .inner
                .get(&2200002020),
            Some(&OptimizedWorkOrder::new(
                None,
                Some(period.clone()),
                HashSet::new(),
                None,
                1000,
                HashMap::new()
            ))
        );

        scheduler_agent_algorithm.update_scheduling_state(strategic_scheduling_message).unwrap();
        scheduler_agent_algorithm.update_resources_state(strategic_resources_message).unwrap();

        assert_eq!(
            scheduler_agent_algorithm.resources_capacity.inner
                .get(&Resources::MtnMech)
                .unwrap()
                .get(&period)
            ,
            Some(&300.0)
        );
        assert_eq!(
            scheduler_agent_algorithm
                .optimized_work_orders
                .inner
                .get(&2200002020)
                .unwrap()
                .scheduled_period,
            None
        );
        assert_eq!(
            scheduler_agent_algorithm
                .optimized_work_orders
                .inner
                .get(&2200002020)
                .unwrap()
                .locked_in_period,
            Some(period.clone())
        );
    }

    #[test]
    fn test_invariant_of_scheduled_period() {
        //todo!()
    }



    impl StrategicResources {
        pub fn default() -> Self {
            Self {
                inner: HashMap::new()
            }
        }
    }

    
    impl OptimizedWorkOrder {

        pub fn new(
            scheduled_period: Option<Period>,
            locked_in_period: Option<Period>,
            excluded_periods: HashSet<Period>,
            latest_period: Option<Period>,
            weight: u32,
            work_load: HashMap<Resources, f64>,
        ) -> Self {
            Self {
                scheduled_period,
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
        let start_date = Utc.with_ymd_and_hms(2023, 11, 20, 7, 0, 0).unwrap();
        let end_date = start_date + chrono::Duration::days(13);

        let period = Period::new(1, start_date, end_date);

        let mut optimized_work_orders = OptimizedWorkOrders::new(HashMap::new());

        let optimized_work_order =
            OptimizedWorkOrder::new(None, None, HashSet::new(), None, 1000, HashMap::new());

        optimized_work_orders.insert_optimized_work_order(2200002020, optimized_work_order);

        let mut resource_capacity = HashMap::new();
        let mut resource_loadings = HashMap::new();

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
        let mut scheduler_agent_algorithm = StrategicAlgorithm::new(
            0.0,
            StrategicResources::new(resource_capacity),
            StrategicResources::new(resource_loadings),
            PriorityQueues::new(),
            optimized_work_orders,
            HashSet::new(),
            vec![period.clone()],
        );

        scheduler_agent_algorithm.schedule_normal_work_order(2200002020, &period);

        assert_eq!(
            scheduler_agent_algorithm
                .optimized_work_orders
                .inner
                .get(&2200002020)
                .unwrap()
                .scheduled_period,
            Some(period.clone())
        );
    }

    #[test]
    fn test_schedule_work_order_with_work_load() {
        let start_date = Utc.with_ymd_and_hms(2023, 11, 20, 7, 0, 0).unwrap();
        let end_date = start_date + chrono::Duration::days(13);

        let period = Period::new(1, start_date, end_date);

        let mut work_load = HashMap::new();

        work_load.insert(Resources::MtnMech, 100.0);

        let mut optimized_work_orders = OptimizedWorkOrders::new(HashMap::new());

        let optimized_work_order =
            OptimizedWorkOrder::new(None, None, HashSet::new(), None, 1000, work_load);

        optimized_work_orders
            .inner
            .insert(2200002020, optimized_work_order);

        let mut resource_capacity = HashMap::new();
        let mut resource_loadings = HashMap::new();

        let mut period_hash_map_0 = HashMap::new();

        period_hash_map_0.insert(period.clone(), 0.0);

        resource_capacity.insert(Resources::MtnMech, period_hash_map_0.clone());
        resource_loadings.insert(Resources::MtnMech, period_hash_map_0.clone());

        let mut scheduler_agent_algorithm = StrategicAlgorithm::new(
            0.0,
            StrategicResources::new(resource_capacity),
            StrategicResources::new(resource_loadings),
            PriorityQueues::new(),
            optimized_work_orders,
            HashSet::new(),
            vec![period.clone()],
        );
        scheduler_agent_algorithm.schedule_normal_work_order(2200002020, &period);

        assert_eq!(
            scheduler_agent_algorithm
                .optimized_work_orders
                .inner
                .get(&2200002020)
                .as_ref()
                .unwrap()
                .scheduled_period,
            scheduler_agent_algorithm.periods().last().cloned()
        );
    }

    #[test]
    fn test_update_loadings() {
        let mut work_load = HashMap::new();

        work_load.insert(Resources::MtnMech, 20.0);
        work_load.insert(Resources::MtnElec, 40.0);
        work_load.insert(Resources::Prodtech, 60.0);

        let start_date = Utc.with_ymd_and_hms(2023, 11, 20, 7, 0, 0).unwrap();
        let end_date = start_date + chrono::Duration::days(13);

        let period = Period::new(1, start_date, end_date);

        let mut resource_capacity = HashMap::new();
        let mut resource_loadings = HashMap::new();

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

        let mut strategic_agent_algorithm = StrategicAlgorithm::new(
            0.0,
            StrategicResources::new(resource_capacity),
            StrategicResources::new(resource_loadings),
            PriorityQueues::new(),
            OptimizedWorkOrders::new(HashMap::new()),
            HashSet::new(),
            vec![],
        );

        let work_order_number = 2100000001;
        
        let work_order = OptimizedWorkOrder::new(
            Some(period.clone()),
            Some(period.clone()),
            HashSet::new(),
            None,
            1000,
            work_load,
        );

        strategic_agent_algorithm.optimized_work_orders.inner.insert(work_order_number, work_order);
        strategic_agent_algorithm.update_loadings(work_order_number, &period, LoadOperation::Add);

        assert_eq!(
            strategic_agent_algorithm
                .resources_loading
                .inner
                .get(&Resources::MtnMech)
                .unwrap()
                .get(&period.clone()),
            Some(20.0).as_ref()
        );
        assert_eq!(
            strategic_agent_algorithm
                .resources_loading
                .inner
                .get(&Resources::MtnElec)
                .unwrap()
                .get(&period.clone()),
            Some(40.0).as_ref()
        );
        assert_eq!(
            strategic_agent_algorithm
                .resources_loading
                .inner
                .get(&Resources::Prodtech)
                .unwrap()
                .get(&period.clone()),
            Some(60.0).as_ref()
        );

        assert_eq!(
            strategic_agent_algorithm
                .resources_loading
                .inner
                .get(&Resources::MtnScaf),
            None
        );
    }

    #[test]
    fn test_unschedule_work_order() {
        let mut work_load = HashMap::new();

        work_load.insert(Resources::MtnMech, 20.0);
        work_load.insert(Resources::MtnElec, 40.0);
        work_load.insert(Resources::Prodtech, 60.0);

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
        let resources = vec![Resources::MtnMech, Resources::MtnElec, Resources::Prodtech];
        // Again, this is not completely correct. There is an invariant here that is not being
        // upheld correctly. What should we do about that?
        let mut resource_capacity: HashMap<Resources, HashMap<Period, f64>> = HashMap::new();
        let mut resource_loadings: HashMap<Resources, HashMap<Period, f64>> = HashMap::new();

        for resource in resources.iter() {
            let capacity_map = resource_capacity.entry(resource.clone()).or_default();
            let loading_map = resource_loadings.entry(resource.clone()).or_default();

            for period in periods.iter() {
                capacity_map.insert(period.clone(), 150.0);
                loading_map.insert(period.clone(), 0.0);
            }
        }

        let mut optimized_work_orders = OptimizedWorkOrders::new(HashMap::new());

        let optimized_work_order = OptimizedWorkOrder::new(
            None,
            Some(period_1.clone()),
            HashSet::new(),
            None,
            1000,
            work_load,
        );

        optimized_work_orders
            .inner
            .insert(2200002020, optimized_work_order);

        let mut scheduler_agent_algorithm = StrategicAlgorithm::new(
            0.0,
            StrategicResources::new(resource_capacity),
            StrategicResources::new(resource_loadings),
            PriorityQueues::new(),
            optimized_work_orders,
            HashSet::new(),
            periods,
        );

        assert_eq!(
            *scheduler_agent_algorithm
                .resources_loadings()
                .inner
                .get(&Resources::MtnMech)
                .unwrap()
                .get(&period_1)
                .unwrap(),
            0.0
        );
        scheduler_agent_algorithm.schedule_normal_work_order(2200002020, &period_1);

        assert_eq!(
            *scheduler_agent_algorithm
                .resources_loadings()
                .inner
                .get(&Resources::MtnMech)
                .unwrap()
                .get(&period_1)
                .unwrap(),
            20.0
        );

        assert_eq!(
            *scheduler_agent_algorithm
                .resources_loadings()
                .inner
                .get(&Resources::MtnElec)
                .unwrap()
                .get(&period_1)
                .unwrap(),
            40.0
        );
        assert_eq!(
            *scheduler_agent_algorithm
                .resources_loadings()
                .inner
                .get(&Resources::Prodtech)
                .unwrap()
                .get(&period_1)
                .unwrap(),
            60.0
        );

        scheduler_agent_algorithm.unschedule(2200002020);
        assert_eq!(
            *scheduler_agent_algorithm
                .resources_loadings()
                .inner
                .get(&Resources::MtnMech)
                .unwrap()
                .get(&period_1)
                .unwrap(),
            0.0
        );
        assert_eq!(
            *scheduler_agent_algorithm
                .resources_loadings()
                .inner
                .get(&Resources::MtnElec)
                .unwrap()
                .get(&period_1)
                .unwrap(),
            0.0
        );
        assert_eq!(
            *scheduler_agent_algorithm
                .resources_loadings()
                .inner
                .get(&Resources::Prodtech)
                .unwrap()
                .get(&period_1)
                .unwrap(),
            0.0
        );

        scheduler_agent_algorithm.schedule_forced_work_order(2200002020);

        assert_eq!(
            *scheduler_agent_algorithm
                .resources_loadings()
                .inner
                .get(&Resources::MtnMech)
                .unwrap()
                .get(&period_1)
                .unwrap(),
            20.0
        );
        assert_eq!(
            *scheduler_agent_algorithm
                .resources_loadings()
                .inner
                .get(&Resources::MtnElec)
                .unwrap()
                .get(&period_1)
                .unwrap(),
            40.0
        );
        assert_eq!(
            *scheduler_agent_algorithm
                .resources_loadings()
                .inner
                .get(&Resources::Prodtech)
                .unwrap()
                .get(&period_1)
                .unwrap(),
            60.0
        );

        scheduler_agent_algorithm
            .optimized_work_orders
            .set_locked_in_period(2200002020, period_2.clone());
        scheduler_agent_algorithm.schedule_forced_work_order(2200002020);

        assert_eq!(
            *scheduler_agent_algorithm
                .resources_loadings()
                .inner
                .get(&Resources::MtnMech)
                .unwrap()
                .get(&period_1)
                .unwrap(),
            0.0
        );
        assert_eq!(
            *scheduler_agent_algorithm
                .resources_loadings()
                .inner
                .get(&Resources::MtnElec)
                .unwrap()
                .get(&period_1)
                .unwrap(),
            0.0
        );
        assert_eq!(
            *scheduler_agent_algorithm
                .resources_loadings()
                .inner
                .get(&Resources::Prodtech)
                .unwrap()
                .get(&period_1)
                .unwrap(),
            0.0
        );

        assert_eq!(
            *scheduler_agent_algorithm
                .resources_loadings()
                .inner
                .get(&Resources::MtnMech)
                .unwrap()
                .get(&period_2)
                .unwrap(),
            20.0
        );
        assert_eq!(
            *scheduler_agent_algorithm
                .resources_loadings()
                .inner
                .get(&Resources::MtnElec)
                .unwrap()
                .get(&period_2)
                .unwrap(),
            40.0
        );
        assert_eq!(
            *scheduler_agent_algorithm
                .resources_loadings()
                .inner
                .get(&Resources::Prodtech)
                .unwrap()
                .get(&period_2)
                .unwrap(),
            60.0
        );

        scheduler_agent_algorithm.unschedule(2200002020);
        assert_eq!(
            *scheduler_agent_algorithm
                .resources_loadings()
                .inner
                .get(&Resources::MtnMech)
                .unwrap()
                .get(&period_1)
                .unwrap(),
            0.0
        );
        assert_eq!(
            *scheduler_agent_algorithm
                .resources_loadings()
                .inner
                .get(&Resources::MtnElec)
                .unwrap()
                .get(&period_1)
                .unwrap(),
            0.0
        );
        assert_eq!(
            *scheduler_agent_algorithm
                .resources_loadings()
                .inner
                .get(&Resources::Prodtech)
                .unwrap()
                .get(&period_1)
                .unwrap(),
            0.0
        );

        assert_eq!(
            *scheduler_agent_algorithm
                .resources_loadings()
                .inner
                .get(&Resources::MtnMech)
                .unwrap()
                .get(&period_1)
                .unwrap(),
            0.0
        );
        assert_eq!(
            *scheduler_agent_algorithm
                .resources_loadings()
                .inner
                .get(&Resources::MtnElec)
                .unwrap()
                .get(&period_1)
                .unwrap(),
            0.0
        );
        assert_eq!(
            *scheduler_agent_algorithm
                .resources_loadings()
                .inner
                .get(&Resources::Prodtech)
                .unwrap()
                .get(&period_1)
                .unwrap(),
            0.0
        );
    }

    #[test]
    fn test_unschedule_random_work_orders() {
        let mut work_load_1 = HashMap::new();
        let mut work_load_2 = HashMap::new();
        let mut work_load_3 = HashMap::new();

        work_load_1.insert(Resources::MtnMech, 10.0);
        work_load_1.insert(Resources::MtnElec, 10.0);
        work_load_1.insert(Resources::Prodtech, 10.0);

        work_load_2.insert(Resources::MtnMech, 20.0);
        work_load_2.insert(Resources::MtnElec, 20.0);
        work_load_2.insert(Resources::Prodtech, 20.0);

        work_load_3.insert(Resources::MtnMech, 30.0);
        work_load_3.insert(Resources::MtnElec, 30.0);
        work_load_3.insert(Resources::Prodtech, 30.0);

        let mut optimized_work_orders = OptimizedWorkOrders::new(HashMap::new());

        let optimized_work_order_1 = OptimizedWorkOrder::new(
            Some(Period::new_from_string("2023-W47-48").unwrap()),
            None,
            HashSet::new(),
            None,
            1000,
            work_load_1,
        );

        let optimized_work_order_2 = OptimizedWorkOrder::new(
            Some(Period::new_from_string("2023-W47-48").unwrap()),
            None,
            HashSet::new(),
            None,
            1000,
            work_load_2,
        );

        let optimized_work_order_3 = OptimizedWorkOrder::new(
            Some(Period::new_from_string("2023-W49-50").unwrap()),
            None,
            HashSet::new(),
            None,
            1000,
            work_load_3,
        );

        optimized_work_orders
            .inner
            .insert(2200000001, optimized_work_order_1);
        optimized_work_orders
            .inner
            .insert(2200000002, optimized_work_order_2);
        optimized_work_orders
            .inner
            .insert(2200000003, optimized_work_order_3);

        let periods: Vec<Period> = vec![
            Period::new_from_string("2023-W47-48").unwrap(),
            Period::new_from_string("2023-W49-50").unwrap(),
        ];

        let mut scheduler_agent_algorithm = StrategicAlgorithm::new(
            0.0,
            StrategicResources::default(),
            StrategicResources::default(),
            PriorityQueues::new(),
            optimized_work_orders,
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
            scheduler_agent_algorithm
                .optimized_work_orders
                .inner
                .get(&2200000001)
                .unwrap()
                .scheduled_period,
            Some(Period::new_from_string("2023-W47-48").unwrap())
        );

        assert_eq!(
            scheduler_agent_algorithm
                .optimized_work_orders
                .inner
                .get(&2200000002)
                .unwrap()
                .scheduled_period,
            None
        );

        assert_eq!(
            scheduler_agent_algorithm
                .optimized_work_orders
                .inner
                .get(&2200000003)
                .unwrap()
                .scheduled_period,
            None
        );
    }

    #[test]
    fn test_calculate_period_difference() {
        let period_1 = Period::new_from_string("2023-W47-48");
        let period_2 = Period::new_from_string("2023-W49-50");

        let difference = calculate_period_difference(period_1.unwrap(), Some(period_2.unwrap()));

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
        let mut optimized_work_orders = OptimizedWorkOrders::new(HashMap::new());

        let optimized_work_order = OptimizedWorkOrder::new(
            Period::new_from_string("2023-W47-48").ok(),
            None,
            HashSet::new(),
            None,
            1000,
            HashMap::new(),
        );

        optimized_work_orders
            .inner
            .insert(2100000001, optimized_work_order);

        let mut scheduler_agent_algorithm = StrategicAlgorithm::new(
            0.0,
            StrategicResources::default(),
            StrategicResources::default(),
            PriorityQueues::new(),
            optimized_work_orders,
            HashSet::new(),
            vec![Period::new_from_string("2023-W47-48").unwrap()],
        );

        scheduler_agent_algorithm.unschedule(2100000001);
        assert_eq!(
            scheduler_agent_algorithm
                .optimized_work_orders
                .inner
                .get(&2100000001)
                .unwrap()
                .scheduled_period,
            None
        );
    }

    #[test]
    fn test_period_clone_equality() {
        let period_1 = Period::new_from_string("2023-W47-48").unwrap();
        let period_2 = Period::new_from_string("2023-W47-48").unwrap();

        assert_eq!(period_1, period_2);
        assert_eq!(period_1, period_1.clone());
    }

    impl StrategicAlgorithm {
        pub fn set_optimized_work_order(
            &mut self,
            work_order_number: u32,
            optimized_work_order: OptimizedWorkOrder,
        ) {
            self.optimized_work_orders
                .inner
                .insert(work_order_number, optimized_work_order);
        }
    }

    #[test]
    fn test_strategic_optimized_work_order_builder() {
        // let OptimizedWorkOrderBuilder::new();


        
    }

}
