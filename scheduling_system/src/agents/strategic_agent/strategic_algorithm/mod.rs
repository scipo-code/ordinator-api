mod algorithm;
use std::fmt::Write;

use std::collections::{HashMap, HashSet};
use std::fmt::{ Display};
use std::hash::{Hash, Hasher};

use priority_queue::PriorityQueue;
use shared_messages::agent_error::AgentError;
use shared_messages::strategic::strategic_resources_message::{StrategicResourcesMessage};
use shared_messages::strategic::strategic_scheduling_message::StrategicSchedulingMessage;
use tracing::{debug, info, instrument};

use crate::models::time_environment::period::{Period};
use shared_messages::resources::Resources;

#[derive(Debug, Clone)]

pub struct StrategicAlgorithm {
    objective_value: f64,
    resources_capacity: AlgorithmResources,
    resources_loading: AlgorithmResources,
    priority_queues: PriorityQueues<u32, u32>,
    optimized_work_orders: OptimizedWorkOrders,
    periods: Vec<Period>,
    changed: bool,
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
}

impl AlgorithmResources {
    fn to_string(&self, number_of_periods: u32) -> String{
        let mut string = String::new();
        let mut periods = self.inner.values()
            .flat_map(|inner_map| inner_map.keys())
            .collect::<Vec<_>>();
        periods.sort();
        periods.dedup();


        // Header
        write!(string, "{:<12}", "Resource").ok();
        for (_, period) in periods.iter().enumerate().take(number_of_periods as usize) {
            write!(string, "{:>12}", period.get_period_string()).ok();
        }
        writeln!(string).ok();
        

        // Rows
        for (resource, inner_map) in self.inner.iter() {
            write!(string, "{:<12}", resource.variant_name()).unwrap();
            for (_, period) in periods.iter().enumerate().take(number_of_periods as usize) {
                let value = inner_map.get(period).unwrap_or(&0.0);
                write!(string, "{:>12}", value.round()).ok();
            }
            writeln!(string).ok();
        }
        string
    }



}

impl StrategicAlgorithm {
    pub fn get_optimized_work_order(&self, work_order_number: &u32) -> Option<&OptimizedWorkOrder> {
        self.optimized_work_orders.inner.get(work_order_number)
    }

    pub fn set_periods(&mut self, periods: Vec<Period>) {
        self.periods = periods;
    }

    pub fn get_objective_value(&self) -> f64 {
        self.objective_value
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
    #[instrument(skip(self))]
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
    #[instrument(skip(self))]
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

#[derive(Debug, PartialEq, Clone, Default)]
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

    #[instrument(skip(self))]
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

impl StrategicAlgorithm {
    pub fn update_resources_state(
        &mut self,
        strategic_resources_message: StrategicResourcesMessage,
    ) -> Result<String, AgentError> {

        match strategic_resources_message {
            StrategicResourcesMessage::SetResources(manual_resources) => {
                let mut count = 0;
                for (resource, periods) in manual_resources {
                    for (period_string, capacity) in periods {
                        let period = self.periods.iter().find(|period| period.get_period_string() == period_string).expect("The period was not found in the self.periods vector. Somehow a message was sent form the frontend without the period being initialized correctly.");
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
            StrategicResourcesMessage::GetLoadings {
                periods_end,
                select_resources: _,
            
            } => {
                
                let loading = self.get_resources_loadings();

                let periods_end: u32 = periods_end.parse().unwrap();

                Ok(loading.to_string(periods_end))
            }
            StrategicResourcesMessage::GetCapacities { periods_end, select_resources: _ } => 
            {         
                let capacities = self.get_resources_capacities();

                let periods_end: u32 = periods_end.parse().unwrap();

                Ok(capacities.to_string(periods_end))
            }
        }
    }
    #[allow(dead_code)]
    pub fn update_periods_state() { todo!() }

    #[tracing::instrument(name = "update_scheduling_state", level = "DEBUG", skip(self, strategic_scheduling_message), fields(self.objective_value))]
    pub fn update_scheduling_state(
        &mut self,
        strategic_scheduling_message: StrategicSchedulingMessage,
    ) -> Result<String, AgentError> {
        match strategic_scheduling_message {
            StrategicSchedulingMessage::Schedule(schedule_work_order) => {
                let work_order_number = schedule_work_order.get_work_order_number();
                let period = self
                    .periods
                    .iter()
                    .find(|period| {
                        period.get_period_string() == schedule_work_order.get_period_string()
                    })
                    .cloned();

                    dbg!(&schedule_work_order);
                    dbg!(self.get_optimized_work_order(&schedule_work_order.get_work_order_number()));
    
                match period {
                    Some(period) => {
                        self.optimized_work_orders
                            .set_locked_in_period(work_order_number, period.clone());
             
                        Ok(format!(
                            "Work order {} has been scheduled for period {}",
                            work_order_number,
                            period.get_period_string()
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
                            period.get_period_string() == schedule_work_order.get_period_string()
                        })
                        .cloned();

                    match period {
                        Some(period) => {
                            self.optimized_work_orders
                                .set_locked_in_period(work_order_number, period.clone());

                            output_string += &format!(
                                "Work order {} has been scheduled for period {}",
                                work_order_number,
                                period.clone().get_period_string()
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
                    period.get_period_string() == exclude_from_period.get_period_string().clone()
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
                            info!("Work order {} has been excluded from period {} and the locked in period has been removed", work_order_number, period.get_period_string());
                        }
                            
                        Ok(format!(
                            "Work order {} has been excluded from period {}",
                            work_order_number,
                            period.get_period_string()
                        ))
                    }
                    None => Err(AgentError::StateUpdateError("The period was not found in the self.periods vector. Somehow a message was sent form the frontend without the period being initialized correctly.".to_string())),
                }
            }
        }
    }
}

/// Remember that it should under no circumstances be in the algorithm. It should be in the agent
/// the algothim should be updated by the agent when a message is received. If the backlog is in 
/// the algorithm it means that we will have to update the backlog in the algorithm on every message
/// that is received. This is silly as the agent should be the one that is responsible for updating
/// the algorithm. 
/// 
/// This is actually a huge problem that is going to cause a lot of issues. I need to fix this now
/// The central question here is which different types there exists for the different types of the
/// work orders. Unloading point means that it is fixed and shutdown/vendor means that action is 
/// required before scheduling is meaningful. It does not make sense to have this in the algorithm
/// itself. This is clearly on the wrong level of abstraction. It should again be handle in the 
/// agent. The agent communicates the state of the work orders to user. The idea of a priority queue
/// is a good one, but I think that relying on the period_lock and the period_exclusion is a much
/// better way of handling it. More general. There could be speedups associated with having multiple
/// priority queues, but I think that it is a bit too early to start thinking about that. Profiling 
/// should determine the appropriate course of action at that point.
impl StrategicAlgorithm {
    pub fn populate_priority_queues(&mut self) {
        for (key, work_order) in self.optimized_work_orders.inner.iter() {
            debug!("Work order {} has been added to the normal queue", key);
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
        periods: Vec<Period>,
        changed: bool,
    ) -> Self {
        StrategicAlgorithm {
            objective_value,
            resources_capacity,
            resources_loading,
            priority_queues,
            optimized_work_orders,
            periods,
            changed,
        }
    }

    pub fn get_optimized_work_orders(&self) -> &HashMap<u32, OptimizedWorkOrder> {
        &self.optimized_work_orders.inner
    }

    pub fn get_resources_loadings(&self) -> &AlgorithmResources {
        &self.resources_loading
    }

    pub fn get_resources_capacities(&self) -> &AlgorithmResources {
        &self.resources_capacity
    }


    pub fn get_periods(&self) -> &Vec<Period> {
        &self.periods
    }



}


#[cfg(test)]
mod tests {
    use shared_messages::strategic::{
        strategic_scheduling_message::SingleWorkOrder,
    };

    use super::*;

    use std::collections::HashMap;

    use crate::{
        agents::strategic_agent::strategic_algorithm::{
            OptimizedWorkOrders, PriorityQueues, StrategicAlgorithm,
        },
        models::{
            work_order::{
                functional_location::FunctionalLocation,
                order_dates::OrderDates,
                order_text::OrderText,
                order_type::{WDFPriority, WorkOrderType},
                priority::Priority,
                revision::Revision,
                status_codes::StatusCodes,
                system_condition::SystemCondition,
                unloading_point::UnloadingPoint,
                WorkOrder,
            },
            WorkOrders,
        },
    };

    #[test]
    fn test_update_scheduler_algorithm_state() {
        let mut work_orders = WorkOrders::new();

        let work_order = WorkOrder::new(
            2200002020,
            false,
            1000,
            Priority::new_int(1),
            100.0,
            HashMap::new(),
            HashMap::new(),
            vec![],
            vec![],
            vec![],
            WorkOrderType::Wdf(WDFPriority::new(1)),
            SystemCondition::new(),
            StatusCodes::new_default(),
            OrderDates::new_test(),
            Revision::new_default(),
            UnloadingPoint::new_default(),
            FunctionalLocation::new_default(),
            OrderText::new_default(),
            false,
        );

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
            periods,
            true,
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
        let strategic_resources_message = StrategicResourcesMessage::new_test();

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
