mod algorithm;

use std::collections::{HashMap, HashSet};
use std::fmt::Display;
use std::hash::{Hash, Hasher};

use priority_queue::PriorityQueue;
use tracing::{debug, event, span, Level};

use crate::agents::scheduler_agent::scheduler_message::InputSchedulerMessage;
use crate::models::time_environment::period::Period;
use crate::models::WorkOrders;

#[derive(Debug)]
pub struct SchedulerAgentAlgorithm {
    objective_value: f64,
    manual_resources_capacity: HashMap<(String, String), f64>,
    manual_resources_loading: HashMap<(String, String), f64>,
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

    pub fn get_optimized_work_order(&self, work_order_number: &u32) -> Option<&OptimizedWorkOrder> {
        self.optimized_work_orders.inner.get(work_order_number)
    }

    pub fn changed(&self) -> bool {
        self.changed
    }

    pub fn set_changed(&mut self, changed: bool) {
        self.changed = changed;
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

#[derive(Debug, PartialEq, Clone)]
pub struct OptimizedWorkOrder {
    pub scheduled_period: Option<Period>,
    pub locked_in_period: Option<Period>,
    pub excluded_from_periods: HashSet<Period>,
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

impl Default for OptimizedWorkOrder {
    fn default() -> Self {
        Self {
            scheduled_period: None,
            locked_in_period: None,
            excluded_from_periods: HashSet::new(),
        }
    }
}

impl OptimizedWorkOrders {
    pub fn new(inner: HashMap<u32, OptimizedWorkOrder>) -> Self {
        Self { inner }
    }

    #[cfg(test)]
    pub fn insert_optimized_work_order(
        &mut self,
        work_order_number: u32,
        optimized_work_order: OptimizedWorkOrder,
    ) {
        self.inner.insert(work_order_number, optimized_work_order);
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

    #[cfg(test)]
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
    #[allow(dead_code)]
    pub fn with_new_schedule(&mut self, scheduled_period: Option<Period>) -> Self {
        Self {
            scheduled_period,
            locked_in_period: self.locked_in_period.clone(),
            excluded_from_periods: self.excluded_from_periods.clone(),
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

    pub fn set_locked_in_period(&mut self, period: Option<Period>) {
        self.locked_in_period = period;
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
            self.manual_resources_capacity,
            self.manual_resources_loading,
            self.backlog,
            self.priority_queues,
            self.optimized_work_orders,
            self.periods
        )
    }
}

impl SchedulerAgentAlgorithm {
    pub fn log_optimized_work_orders(&self) {
        for (work_order_number, optimized) in &self.optimized_work_orders.inner {
            match &optimized.locked_in_period {
                Some(period) => {
                    event!(tracing::Level::TRACE, work_order_number = %work_order_number, period = period.period_string)
                }
                None => {
                    event!(tracing::Level::TRACE, work_order_number = %work_order_number,  period = "no locked period")
                }
            }

            match &optimized.scheduled_period {
                Some(period) => {
                    event!(tracing::Level::TRACE, work_order_number = %work_order_number, period = %period.period_string)
                }
                None => {
                    event!(tracing::Level::TRACE, work_order_number = %work_order_number,  period = "None")
                }
            }

            for period in &optimized.excluded_from_periods {
                event!(tracing::Level::TRACE, work_order_number = %work_order_number, period = %period)
            }
        }
    }
}

impl SchedulerAgentAlgorithm {
    #[tracing::instrument(name = "update_scheduler_algorithm_state", level = "DEBUG", skip(self, input_message), fields(self.objective_value))]
    pub fn update_scheduler_algorithm_state(&mut self, input_message: InputSchedulerMessage) {
        let _span = span!(Level::INFO, "update_scheduler_algorithm_state");
        self.manual_resources_capacity = input_message.get_manual_resources();

        for work_order_period_mapping in input_message.work_order_period_mappings {
            let message = match self
                .optimized_work_orders
                .inner
                .get(&work_order_period_mapping.work_order_number)
            {
                Some(work_order) => {
                    format!(
                        "work_order is suggested in {:?} \n 
                    work_order is scheduled in {:?} \n
                    work_order is excluded {:?} \n",
                        work_order.scheduled_period,
                        work_order.locked_in_period,
                        work_order.excluded_from_periods
                    )
                }
                None => "work_order is not in optimized work orders".to_string(),
            };

            event!(
                tracing::Level::DEBUG,
                "scheduler optimized work order state before update{}",
                message
            );

            event!(
                tracing::Level::DEBUG,
                "The manual resources are: {:?}",
                work_order_period_mapping
            );

            let work_order_number: u32 = work_order_period_mapping.work_order_number;
            let optimized_work_orders = &self.optimized_work_orders.inner;

            let locked_in_period = work_order_period_mapping.period_status.locked_in_period;
            let excluded_from_periods = work_order_period_mapping
                .period_status
                .excluded_from_periods;

            let scheduled_period: Option<Period> = optimized_work_orders
                .get(&work_order_number)
                .map(|ow| ow.scheduled_period.clone())
                .unwrap_or(None);

            match locked_in_period.clone() {
                Some(period) => {
                    debug!(target: "frontend input message debugging", "Locked period: {}", period.period_string.clone());
                }
                None => {
                    debug!(target: "frontend input message debugging", "Locked period: None");
                }
            }

            let optimized_work_order = OptimizedWorkOrder {
                scheduled_period,
                locked_in_period: locked_in_period.clone(),
                excluded_from_periods,
            };

            let mut excluded_periods = "".to_string();
            for period in &optimized_work_order.excluded_from_periods {
                excluded_periods += &(period.to_string() + " ");
            }

            debug!(work_order_number = %work_order_number,
                info = "Work order updated",
                suggested_period = match &optimized_work_order.scheduled_period {
                    Some(period) => period.period_string.clone(),
                    None => "no suggested period".to_string()
                },
                locked_in_period = match &optimized_work_order.locked_in_period {
                    Some(period) => period.period_string.clone(),
                    None => "no lock on period".to_string()
                },
                excluded_periods = %excluded_periods
            );
            dbg!(optimized_work_order.clone());
            self.optimized_work_orders
                .inner
                .insert(work_order_number, optimized_work_order);

            self.update_priority_queues();
        }
    }
}

impl SchedulerAgentAlgorithm {
    pub fn populate_priority_queues(&mut self) {
        for (key, work_order) in self.backlog.inner.iter() {
            if work_order.unloading_point.present {
                debug!("Work order {} has been added to the unloading queue", key);
                self.priority_queues
                    .unloading
                    .push(*key, work_order.order_weight);
            } else if work_order.revision.shutdown || work_order.vendor {
                debug!(
                    "Work order {} has been added to the shutdown/vendor queue",
                    key
                );
                self.priority_queues
                    .shutdown_vendor
                    .push(*key, work_order.order_weight);
            } else {
                debug!("Work order {} has been added to the normal queue", key);
                self.priority_queues
                    .normal
                    .push(*key, work_order.order_weight);
            }
        }
    }

    /// So the idea here is that we look through all the optimized_work_orders and then we schedule
    /// them according to the queue type. There are two cases that should be covered.
    ///
    /// Inclusion
    ///     Here we have to move a work order to the unloading point queue. If the work order is
    ///     already scheduled we have the logic in place to handle this.
    ///    
    ///
    /// Exclusion
    ///     We need to force this invariant on the data type.
    ///
    /// I am doing the wrong thing here. We only care about the
    ///
    /// The exclusion is simply a variation of the materials, EASD. In the code we should create
    /// something to handle this issue. Exclusion is already handled in the code.
    ///
    fn update_priority_queues(&mut self) {
        for (key, work_order) in &self.optimized_work_orders.inner {
            let work_order_weight = self.backlog.inner.get(key).unwrap().order_weight;
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
            .work_load
            .keys();
        let needed_resources: Vec<_> = needed_keys.cloned().collect();
        for resource in needed_resources.clone() {
            self.get_or_initialize_manual_resources_loading(
                resource.clone(),
                period.period_string.clone(),
            );
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
        manual_resources_capacity: HashMap<(String, String), f64>,
        manual_resources_loading: HashMap<(String, String), f64>,
        backlog: WorkOrders,
        priority_queues: PriorityQueues<u32, u32>,
        optimized_work_orders: OptimizedWorkOrders,
        periods: Vec<Period>,
        changed: bool,
    ) -> Self {
        SchedulerAgentAlgorithm {
            objective_value,
            manual_resources_capacity,
            manual_resources_loading,
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

    pub fn get_manual_resources_loadings(&self) -> &HashMap<(String, String), f64> {
        &self.manual_resources_loading
    }

    #[cfg(test)]
    pub fn get_manual_resources_capacities(&self) -> &HashMap<(String, String), f64> {
        &self.manual_resources_capacity
    }

    pub fn get_or_initialize_manual_resources_loading(
        &mut self,
        resource: String,
        period: String,
    ) -> f64 {
        match self
            .manual_resources_loading
            .get(&(resource.clone(), period.clone()))
        {
            Some(loading) => *loading,
            None => {
                self.set_manual_resources_loading(resource, period, 0.0);
                0.0
            }
        }
    }

    pub fn set_manual_resources_loading(&mut self, resource: String, period: String, loading: f64) {
        self.manual_resources_loading
            .insert((resource, period), loading);
    }
}

#[derive(Debug, PartialEq)]
pub enum QueueType {
    Normal,
}

#[cfg(test)]
mod tests {
    use super::*;

    use chrono::{TimeZone, Utc};
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

        let start_date = Utc.with_ymd_and_hms(2023, 11, 20, 7, 0, 0).unwrap();
        let end_date = start_date + chrono::Duration::days(13);
        let period = Period::new(1, start_date, end_date);

        let mut manual_resource_capacity: HashMap<(String, String), f64> = HashMap::new();

        manual_resource_capacity.insert(
            ("MTN_MECH".to_string(), period.period_string.clone()),
            150.0,
        );
        manual_resource_capacity.insert(
            ("MTN_ELEC".to_string(), period.period_string.clone()),
            150.0,
        );
        manual_resource_capacity.insert(
            ("PRODTECH".to_string(), period.period_string.clone()),
            150.0,
        );

        let mut scheduler_agent_algorithm = SchedulerAgentAlgorithm::new(
            0.0,
            manual_resource_capacity,
            HashMap::new(),
            work_orders,
            PriorityQueues::new(),
            OptimizedWorkOrders::new(HashMap::new()),
            vec![],
            true,
        );

        let input_message = InputSchedulerMessage::new_test();

        assert_eq!(
            scheduler_agent_algorithm
                .manual_resources_capacity
                .get(&("MTN_MECH".to_string(), period.period_string.clone())),
            Some(&150.0)
        );
        assert_eq!(
            scheduler_agent_algorithm
                .optimized_work_orders
                .inner
                .get(&2200002020),
            None
        );

        scheduler_agent_algorithm.update_scheduler_algorithm_state(input_message);

        assert_eq!(
            scheduler_agent_algorithm
                .manual_resources_capacity
                .get(&("MTN_MECH".to_string(), period.period_string.clone())),
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
        todo!()
    }

    impl SchedulerAgentAlgorithm {
        pub fn get_periods(&self) -> &Vec<Period> {
            &self.periods
        }

        pub fn get_priority_queues(&self) -> &PriorityQueues<u32, u32> {
            &self.priority_queues
        }
    }
}
