mod algorithm;
use std::fmt::Write;

use std::collections::{HashMap, HashSet};
use std::fmt::Display;
use std::hash::{Hash, Hasher};
use serde::Serialize;
use tracing::{trace, info, instrument};
use colored::*;

use priority_queue::PriorityQueue;
use shared_messages::agent_error::AgentError;
use shared_messages::strategic::strategic_periods_message::StrategicTimeMessage;
use shared_messages::strategic::strategic_resources_message::StrategicResourceMessage;
use shared_messages::strategic::strategic_scheduling_message::StrategicSchedulingMessage;

use crate::agents::traits::LargeNeighborHoodSearch;
use crate::models::time_environment::period::Period;
use shared_messages::resources::Resources;

use self::algorithm::calculate_period_difference;

#[derive(Debug, Clone)]
pub struct StrategicAlgorithm {
    objective_value: f64,
    resources_capacity: AlgorithmResources,
    resources_loading: AlgorithmResources,
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

    pub fn tactical_work_orders(&self) -> Vec<(u32, Period)> {
        let periods = &self.periods.clone()[0..4];
        let mut tactical_work_orders: Vec<(u32, Period)> = vec![];

        for (work_order_number, optimized_work_order) in &self.optimized_work_orders.inner {
            match optimized_work_order.get_scheduled_period() {
                Some(period) => {
                    if periods.contains(&period) {
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
}

#[derive(Debug, Clone)]
pub struct AlgorithmResources {
    pub inner: HashMap<Resources, HashMap<Period, f64>>
}

impl AlgorithmResources {
    pub fn new(resources: HashMap<Resources, HashMap<Period, f64>>) -> Self {
        Self {
            inner: resources
        }
    }

    fn to_string(&self, number_of_periods: u32) -> String{
        let mut string = String::new();
        let mut periods = self.inner.values()
            .flat_map(|inner_map| inner_map.keys())
            .collect::<Vec<_>>();
        periods.sort();
        periods.dedup();

        write!(string, "{:<12}", "Resource").ok();
        for (nr_period, period) in periods.iter().enumerate().take(number_of_periods as usize) {
            if nr_period == 0 {
                write!(string, "{:>12}", period.period_string().red()).ok();
            } else if nr_period == 1 || nr_period == 2 {
                write!(string, "{:>12}", period.period_string().green()).ok();
            } else {
                write!(string, "{:>12}", period.period_string()).ok();
            }
        }
        writeln!(string).ok();
        
        for (resource, inner_map) in self.inner.iter() {
            write!(string, "{:<12}", resource.variant_name()).unwrap();
            for (nr_period, period) in periods.iter().enumerate().take(number_of_periods as usize) {
                let value = inner_map.get(period).unwrap_or(&0.0);
                if nr_period == 0 {
                    write!(string, "{:>12}", value.round().to_string().red()).ok();
                } else if nr_period == 1 || nr_period == 2 {
                    write!(string, "{:>12}", value.round().to_string().green()).ok();
                } else {
                    write!(string, "{:>12}", value.round()).ok();
                }
            }
            writeln!(string).ok();
        }
        string
    }
}

#[derive(Debug, PartialEq, Clone)]
pub struct OptimizedWorkOrders {
    inner: HashMap<u32, OptimizedWorkOrder>,
}

impl Hash for OptimizedWorkOrders {
    fn hash<H: Hasher>(&self, state: &mut H) {
        // Hash the length of the HashMap to ensure different lengths produce different hashes
        self.inner.len().hash(state);

        // Iterate over the HashMap and hash each key-value pair
        for (key, value) in &self.inner {
            key.hash(state);
            value.hash(state);
        }
    }
}

impl Hash for OptimizedWorkOrder {
    fn hash<H: Hasher>(&self, state: &mut H) {
        // Hash the length of the HashMap to ensure different lengths produce different hashes
        self.scheduled_period.hash(state);
        self.locked_in_period.hash(state);
        for period in &self.excluded_periods {
            period.hash(state);
        }
    }
}

impl OptimizedWorkOrders {
    pub fn new(inner: HashMap<u32, OptimizedWorkOrder>) -> Self {
        Self { inner }
    }

    #[instrument(level = "trace", skip_all)]
    pub fn set_scheduled_period(&mut self, work_order_number: u32, period: Period) {
        let optimized_work_order = match self.inner.get_mut(&work_order_number) {
            Some(optimized_work_order) => optimized_work_order,
            None => panic!(
                "Work order number {} not found in optimized work orders",
                work_order_number
            ),
        };
        optimized_work_order.scheduled_period = Some(period);
    }
    #[instrument(level = "trace", skip_all)]
    pub fn get_locked_in_period(&self, work_order_number: u32) -> Period {
        let option_period = match self.inner.get(&work_order_number) {
            Some(optimized_work_order) => optimized_work_order.locked_in_period.clone(),
            None => panic!(
                "Work order number {} not found in optimized work orders",
                work_order_number
            ),
        };
        match option_period {
            Some(period) => period,
            None => panic!("Work order number {} does not have a locked in period, but it is being called by the optimized_work_orders.schedule_forced_work_order", work_order_number)
        }
    }

    #[instrument(level = "trace", skip_all)]
    pub fn set_locked_in_period(&mut self, work_order_number: u32, period: Period) {

        let optimized_work_order = match self.inner.get_mut(&work_order_number) {
            Some(optimized_work_order) => optimized_work_order,
            None => panic!(
                "Work order number {} not found in optimized work orders",
                work_order_number
            ),
        };
        optimized_work_order.locked_in_period = Some(period);
    }


}

#[derive(Debug, PartialEq, Clone, Default, Serialize)]
pub struct OptimizedWorkOrder {
    scheduled_period: Option<Period>,
    locked_in_period: Option<Period>,
    excluded_periods: HashSet<Period>,
    latest_period: Option<Period>,
    weight: u32,
    work_load: HashMap<Resources, f64>,
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

    pub fn get_scheduled_period(&self) -> Option<Period> {
        self.scheduled_period.clone()
    }

    pub fn get_locked_in_period(&self) -> Option<Period> {
        self.locked_in_period.clone()
    }

    pub fn get_excluded_periods(&self) -> &HashSet<Period> {
        &self.excluded_periods
    }

    pub fn get_latest_period(&self) -> Option<Period> {
        self.latest_period.clone()
    }

    pub fn get_work_load(&self) -> &HashMap<Resources, f64> {
        &self.work_load
    }

    pub fn get_weight(&self) -> u32 {
        self.weight
    }

    /// This is a huge no-no! I think that this will lets us violate the invariant that we have
    /// created between scheduled work and the loadings. We should test for this
    pub fn set_scheduled_period(&mut self, period: Option<Period>) {
        self.scheduled_period = period;
    }
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
                    panic!("There are no periods in the system")
                }
            };

            let work_order_latest_allowed_finish_period =
                optimized_work_order.get_latest_period().clone();

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
                    .get_weight() as f64;

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
            period_penalty_contribution + 10000000000.0 * excess_penalty_contribution;
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

        match optimized_work_order.get_scheduled_period() {
            Some(_) => {
                for (resource, periods_loading) in self.resources_loading.inner.iter_mut() {
                    let period = optimized_work_order.get_scheduled_period().unwrap();
                    let loading = periods_loading.get_mut(&period).unwrap();
                    let work_load_for_resource = optimized_work_order.get_work_load().get(resource);
                    if let Some(work_load_for_resource) = work_load_for_resource {
                        *loading -= work_load_for_resource;
                    }
                }
                optimized_work_order.set_scheduled_period(None);
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
    tracing::info!("update_resources_state called");
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

                let algorithm_resources = AlgorithmResources::new(percentage_loading );
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
    pub fn populate_priority_queues(&mut self) {
        for (key, work_order) in self.optimized_work_orders.inner.iter() {
            trace!("Work order {} has been added to the normal queue", key);
            if work_order.scheduled_period.is_none() {
                self.priority_queues
                    .normal
                    .push(*key, work_order.get_weight());
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

// The backlog should not be in the algorithm, it should be in the agent under the 
// SchedulingEnvironment. I should remove it immediately 
impl StrategicAlgorithm {
    pub fn new(
        objective_value: f64,
        resources_capacity: AlgorithmResources,
        resources_loading: AlgorithmResources,
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

    pub fn resources_loadings(&self) -> &AlgorithmResources {
        &self.resources_loading
    }

    pub fn resources_capacities(&self) -> &AlgorithmResources {
        &self.resources_capacity
    }


    pub fn periods(&self) -> &Vec<Period> {
        &self.periods
    }



}


#[cfg(test)]
mod tests {
    use shared_messages::strategic::strategic_scheduling_message::SingleWorkOrder;

    use super::*;

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

        let resource_capacity = AlgorithmResources {
            inner: manual_resource_capacity.clone(),
        };

        let mut scheduler_agent_algorithm = StrategicAlgorithm::new(
            0.0,
            resource_capacity,
            AlgorithmResources::default(),
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



    impl AlgorithmResources {
        pub fn default() -> Self {
            Self {
                inner: HashMap::new()
            }
        }
    }

    impl OptimizedWorkOrders {
            pub fn insert_optimized_work_order(
        &mut self,
        work_order_number: u32,
        optimized_work_order: OptimizedWorkOrder,
    ) {
        self.inner.insert(work_order_number, optimized_work_order);
    }
    }
}
