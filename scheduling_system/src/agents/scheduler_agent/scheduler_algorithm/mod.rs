mod algorithm;

use std::collections::{HashMap, HashSet};
use std::error::Error;
use std::fmt::Display;
use std::hash::{Hash, Hasher};

use priority_queue::PriorityQueue;
use shared_messages::strategic::strategic_resources_message::{self, StrategicResourcesMessage};
use shared_messages::strategic::strategic_scheduling_message::StrategicSchedulingMessage;
use shared_messages::Response;
use tracing::{debug, event, info, span, Level};

use crate::models::time_environment::period::Period;
use crate::models::WorkOrders;
use shared_messages::resources::Resources;

#[derive(Debug)]
pub struct SchedulerAgentAlgorithm {
    objective_value: f64,
    resources_capacity: HashMap<(Resources, Period), f64>,
    resources_loading: HashMap<(Resources, Period), f64>,
    backlog: WorkOrders,
    priority_queues: PriorityQueues<u32, u32>,
    optimized_work_orders: OptimizedWorkOrders,
    periods: Vec<Period>,
    changed: bool,
}

impl SchedulerAgentAlgorithm {
    pub fn get_backlog(&self) -> &WorkOrders {
        &self.backlog
    }

    pub fn get_mut_backlog(&mut self) -> &mut WorkOrders {
        &mut self.backlog
    }

    pub fn get_optimized_work_order(&self, work_order_number: &u32) -> Option<&OptimizedWorkOrder> {
        self.optimized_work_orders.inner.get(work_order_number)
    }

    pub fn changed(&self) -> bool {
        self.changed
    }

    pub fn set_changed(&mut self, changed: bool) {
        self.changed = changed;
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
        for period in &self.excluded_from_periods {
            period.hash(state);
        }
    }
}

impl OptimizedWorkOrders {
    pub fn new(inner: HashMap<u32, OptimizedWorkOrder>) -> Self {
        Self { inner }
    }

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

    pub fn insert_optimized_work_order(
        &mut self,
        work_order_number: u32,
        optimized_work_order: OptimizedWorkOrder,
    ) {
        self.inner.insert(work_order_number, optimized_work_order);
    }
}

#[derive(Debug, PartialEq, Clone, Default)]
pub struct OptimizedWorkOrder {
    pub scheduled_period: Option<Period>,
    pub locked_in_period: Option<Period>,
    pub excluded_from_periods: HashSet<Period>,
}

impl OptimizedWorkOrder {
    pub fn new(
        scheduled_period: Option<Period>,
        locked_in_period: Option<Period>,
        excluded_from_periods: HashSet<Period>,
    ) -> Self {
        Self {
            scheduled_period,
            locked_in_period,
            excluded_from_periods,
        }
    }

    pub fn get_scheduled_period(&self) -> Option<Period> {
        self.scheduled_period.clone()
    }

    /// This is a huge no-no! I think that this will lets us violate the invariant that we have
    /// created between scheduled work and the loadings. We should test for this
    pub fn update_scheduled_period(&mut self, period: Option<Period>) {
        self.scheduled_period = period;
    }
}

impl Display for SchedulerAgentAlgorithm {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "SchedulerAgentAlgorithm: \n
            objective_value: {}, \n
            manual_resources_capacity: {:?}, \n
            manual_resources_loading: {:?}, \n
            backlog: {:?}, \n
            priority_queues: {:?}, \n
            optimized_work_orders: {:?}, \n
            periods: {:?}",
            self.objective_value,
            self.resources_capacity,
            self.resources_loading,
            self.backlog,
            self.priority_queues,
            self.optimized_work_orders,
            self.periods
        )
    }
}

impl SchedulerAgentAlgorithm {
    pub fn update_resources_state(
        &mut self,
        strategic_resources_message: StrategicResourcesMessage,
    ) -> shared_messages::Response {
        for (key, capacity) in strategic_resources_message.get_manual_resources() {
            let period = self.periods.iter().find(|period| period.get_period_string() == key.1).expect("The period was not found in the self.periods vector. Somehow a message was sent form the frontend without the period being initialized correctly.");
            self.resources_capacity
                .insert((key.0, period.clone()), capacity);
        }
        shared_messages::Response::Success(Some("Resouce updated correctly".to_string()))
    }

    pub fn update_periods_state() {}

    #[tracing::instrument(name = "update_scheduling_state", level = "DEBUG", skip(self, strategic_scheduling_message), fields(self.objective_value))]
    pub fn update_scheduling_state(
        &mut self,
        strategic_scheduling_message: StrategicSchedulingMessage,
    ) -> shared_messages::Response {
        let response: shared_messages::Response = match strategic_scheduling_message {
            StrategicSchedulingMessage::Schedule(schedule_work_order) => {
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
                        self.initialize_loading_used_in_work_order(
                            work_order_number,
                            period.clone(),
                        );
                        shared_messages::Response::Success(Some(format!(
                            "Work order {} has been scheduled for period {}",
                            work_order_number,
                            period.get_period_string()
                        )))
                    }
                    None => shared_messages::Response::Failure,
                }
            }
            StrategicSchedulingMessage::ScheduleMultiple(schedule_work_orders) => {
                let mut output_string = String::new();
                let mut period_result = Result::Ok(());
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
                            self.initialize_loading_used_in_work_order(
                                work_order_number,
                                period.clone(),
                            );
                            output_string += &format!(
                                "Work order {} has been scheduled for period {}",
                                work_order_number,
                                period.clone().get_period_string()
                            )
                            .to_string();
                            period_result = Result::Ok(())
                        }
                        None => {
                            period_result = Result::Err(
                            "The period was not found in the self.periods vector. Somehow a message was sent form the frontend without the period being initialized correctly.",
                            );
                            break;
                        }
                    }
                }
        
                match period_result {
                    Ok(()) => shared_messages::Response::Success(Some(output_string.to_string())),
                    Err(_) => shared_messages::Response::Failure,
                }
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
                            .excluded_from_periods
                            .insert(period.clone());

                        let overwrite = if let Some(locked_in_period) = &optimized_work_order.locked_in_period {
                            if optimized_work_order.excluded_from_periods.contains(locked_in_period) {
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
                            

                        shared_messages::Response::Success(Some(format!(
                            "Work order {} has been excluded from period {}",
                            work_order_number,
                            period.get_period_string()
                        )))
                    }
                    None => shared_messages::Response::Failure,
                }
            }
        };
        self.update_priority_queues();
        response
    }
}

impl SchedulerAgentAlgorithm {
    pub fn populate_priority_queues(&mut self) {
        for (key, work_order) in self.backlog.inner.iter() {
            if work_order.get_unloading_point().present {
                debug!("Work order {} has been added to the unloading queue", key);
                self.priority_queues
                    .unloading
                    .push(*key, work_order.get_order_weight());
            } else if work_order.get_revision().shutdown || work_order.get_vendor() {
                debug!(
                    "Work order {} has been added to the shutdown/vendor queue",
                    key
                );
                self.priority_queues
                    .shutdown_vendor
                    .push(*key, work_order.get_order_weight());
            } else {
                debug!("Work order {} has been added to the normal queue", key);
                self.priority_queues
                    .normal
                    .push(*key, work_order.get_order_weight());
            }
        }
    }

    #[tracing::instrument]
    fn update_priority_queues(&mut self) {
        for (key, work_order) in &self.optimized_work_orders.inner {
            let work_order_weight = self.backlog.inner.get(key).unwrap().get_order_weight();
            match &work_order.locked_in_period {
                Some(_work_order) => {
                    self.priority_queues.unloading.push(*key, work_order_weight);
                }
                None => {}
            }
        }
    }

    fn initialize_loading_used_in_work_order(&mut self, work_order_number: u32, period: Period) {
        let needed_keys = self
            .backlog
            .inner
            .get(&work_order_number)
            .unwrap()
            .get_work_load()
            .keys();
        let needed_resources: Vec<_> = needed_keys.cloned().collect();
        for resource in needed_resources.clone() {
            self.get_or_initialize_manual_resources_loading(resource.clone(), period.clone());
        }
    }
}

#[derive(Debug, Clone)]
pub struct PriorityQueues<T, P>
where
    T: Hash + Eq,
    P: Ord,
{
    unloading: PriorityQueue<T, P>,
    shutdown_vendor: PriorityQueue<T, P>,
    normal: PriorityQueue<T, P>,
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

impl SchedulerAgentAlgorithm {
    pub fn new(
        objective_value: f64,
        manual_resources_capacity: HashMap<(Resources, Period), f64>,
        manual_resources_loading: HashMap<(Resources, Period), f64>,
        backlog: WorkOrders,
        priority_queues: PriorityQueues<u32, u32>,
        optimized_work_orders: OptimizedWorkOrders,
        periods: Vec<Period>,
        changed: bool,
    ) -> Self {
        SchedulerAgentAlgorithm {
            objective_value,
            resources_capacity: manual_resources_capacity,
            resources_loading: manual_resources_loading,
            backlog,
            priority_queues,
            optimized_work_orders,
            periods,
            changed,
        }
    }

    pub fn get_optimized_work_orders(&self) -> &HashMap<u32, OptimizedWorkOrder> {
        &self.optimized_work_orders.inner
    }

    pub fn get_manual_resources_loadings(&self) -> &HashMap<(Resources, Period), f64> {
        &self.resources_loading
    }

    pub fn get_manual_resources_capacities(&self) -> &HashMap<(Resources, Period), f64> {
        &self.resources_capacity
    }

    pub fn get_or_initialize_manual_resources_loading(
        &mut self,
        resource: Resources,
        period: Period,
    ) -> f64 {
        match self
            .resources_loading
            .get(&(resource.clone(), period.clone()))
        {
            Some(loading) => *loading,
            None => {
                panic!(
                    "This should not happen, all resources should be initialized at the outset or
                though messages from the frontend",
                );
            }
        }
    }

    pub fn get_periods(&self) -> &Vec<Period> {
        &self.periods
    }
}

#[derive(Debug, PartialEq)]
pub enum QueueType {
    Normal,
}

#[cfg(test)]
mod tests {
    use shared_messages::strategic::{
        strategic_resources_message, strategic_scheduling_message::SingleWorkOrder,
    };

    use super::*;

    use std::collections::HashMap;

    use crate::{
        agents::scheduler_agent::scheduler_algorithm::{
            OptimizedWorkOrders, PriorityQueues, SchedulerAgentAlgorithm,
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

        let mut manual_resource_capacity: HashMap<(Resources, Period), f64> = HashMap::new();

        manual_resource_capacity.insert(
            (
                Resources::new_from_string("MTN-MECH".to_string()),
                Period::new_from_string(&period.get_period_string()).unwrap(),
            ),
            150.0,
        );
        manual_resource_capacity.insert(
            (
                Resources::new_from_string("MTN-ELEC".to_string()),
                Period::new_from_string(&period.get_period_string()).unwrap(),
            ),
            150.0,
        );
        manual_resource_capacity.insert(
            (
                Resources::new_from_string("PRODTECH".to_string()),
                Period::new_from_string(&period.get_period_string()).unwrap(),
            ),
            150.0,
        );

        let mut scheduler_agent_algorithm = SchedulerAgentAlgorithm::new(
            0.0,
            manual_resource_capacity,
            HashMap::new(),
            work_orders,
            PriorityQueues::new(),
            OptimizedWorkOrders::new(HashMap::new()),
            periods,
            true,
        );

        let optimized_work_order =
            OptimizedWorkOrder::new(None, Some(period.clone()), HashSet::new());

        scheduler_agent_algorithm.set_optimized_work_order(2200002020, optimized_work_order);

        let strategic_scheduling_message = StrategicSchedulingMessage::Schedule(
            SingleWorkOrder::new(2200002020, "2023-W47-48".to_string()),
        );
        let strategic_resources_message = StrategicResourcesMessage::new_test();

        assert_eq!(
            scheduler_agent_algorithm.resources_capacity.get(&(
                Resources::new_from_string("MTN-MECH".to_string()),
                Period::new_from_string(&period.get_period_string()).unwrap()
            )),
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
                HashSet::new()
            ))
        );

        scheduler_agent_algorithm.update_scheduling_state(strategic_scheduling_message);
        scheduler_agent_algorithm.update_resources_state(strategic_resources_message);

        assert_eq!(
            scheduler_agent_algorithm.resources_capacity.get(&(
                Resources::new_from_string("MTN-MECH".to_string()),
                Period::new_from_string(&period.get_period_string()).unwrap()
            )),
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

    impl SchedulerAgentAlgorithm {
        pub fn get_priority_queues(&self) -> &PriorityQueues<u32, u32> {
            &self.priority_queues
        }
    }
}
