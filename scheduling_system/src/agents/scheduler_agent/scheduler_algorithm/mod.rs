mod algorithm;

use std::collections::{HashMap, HashSet};
use std::fmt::Display;
use std::hash::{Hash, Hasher};

use priority_queue::PriorityQueue;
use tracing::{debug, event, span, Level};

use crate::agents::scheduler_agent::scheduler_message::InputSchedulerMessage;
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

#[derive(Debug, PartialEq, Clone, Default)]
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
    pub fn log_optimized_work_orders(&self) {
        for (work_order_number, optimized) in &self.optimized_work_orders.inner {
            match &optimized.locked_in_period {
                Some(period) => {
                    event!(tracing::Level::TRACE, work_order_number = %work_order_number, period = period.get_period_string())
                }
                None => {
                    event!(tracing::Level::TRACE, work_order_number = %work_order_number,  period = "no locked period")
                }
            }

            match &optimized.scheduled_period {
                Some(period) => {
                    event!(tracing::Level::TRACE, work_order_number = %work_order_number, period = %period.get_period_string())
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

        for maunal_resource_capacity in input_message.get_manual_resources() {
            // What should happen if the period is not found? This is a fundamental questions. And
            // the kind of question that I will have to answer for the whole system many times. I
            // think that if the manual period cannot be found in the self.periods that the code
            // should fail. Yes I see no way around this. If we get a manual resource that is not
            // part of the self.periods it means that the periods were not created correctly and
            // that we should panic the thread.

            // I have multiple Period objects here and they are only similar by equality and not
            // by the underlying data. Hmm... I do not like this implementation.

            // What should the goal be here? It code should work inside of business
            let test_manual_resource_capacity = maunal_resource_capacity.clone();
            dbg!(test_manual_resource_capacity.clone());
            dbg!(test_manual_resource_capacity.clone().0);
            dbg!(test_manual_resource_capacity.clone().0 .0);

            let period = self.periods.iter().find(|period| period.get_period_string() == maunal_resource_capacity.0.1).expect("The period was not found in the self.periods vector. Somehow a message was sent form the frontend without the period being initialized correctly.");
            self.resources_capacity.insert(
                (maunal_resource_capacity.0 .0, period.clone()),
                maunal_resource_capacity.1,
            );

            dbg!(self.resources_capacity.clone());
        }

        for work_order_period_mapping in input_message.get_work_order_period_mappings() {
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

            debug!(
                "scheduler optimized work order state before update{}",
                message
            );

            debug!("The manual resources are: {:?}", work_order_period_mapping);

            let work_order_number: u32 = work_order_period_mapping.work_order_number;
            let optimized_work_orders = &self.optimized_work_orders.inner;

            // What should happen if the Option is None? This is a fundamental question. I think
            // that the program should be able to... There is a locked in period_string in the
            // work_order_period_mapping and if this is not to be found in the self.periods then it
            // means that something has gone wrong with the period initialization and the program
            // should panic. This is a fundamental invariant that should be upheld. No this is wrong
            // the Option is on the locked_in_period meaning that there does not have to be a locked
            // period on the work order. This means that locked_in_period should be an
            // Option<Period>
            let locked_in_period: Option<Period> =
                match &work_order_period_mapping.period_status.locked_in_period {
                    Some(period_mapping) => self
                        .get_periods()
                        .iter()
                        .find(|period| {
                            period.get_period_string() == period_mapping.get_period_string().clone()
                        })
                        .cloned(),
                    None => None,
                };

            let mut excluded_from_periods = HashSet::<Period>::new();

            for period_string in &work_order_period_mapping
                .period_status
                .excluded_from_periods
            {
                let excluded_period = self
                    .periods
                    .iter()
                    .find(|period| {
                        period.get_period_string() == period_string.period_string.clone()
                    })
                    .cloned();

                excluded_from_periods.insert(excluded_period.unwrap());
            }

            let scheduled_period: Option<Period> = optimized_work_orders
                .get(&work_order_number)
                .map(|ow| ow.scheduled_period.clone())
                .unwrap_or(None);

            match locked_in_period.clone() {
                Some(period) => {
                    debug!(target: "frontend input message debugging", "Locked period: {}", period.get_period_string().clone());
                }
                None => {
                    debug!(target: "frontend input message debugging", "Locked period: None");
                }
            }

            let optimized_work_order = OptimizedWorkOrder {
                scheduled_period,
                locked_in_period: locked_in_period.clone(),
                excluded_from_periods: excluded_from_periods.clone(),
            };

            let mut excluded_periods = "".to_string();
            for period in &optimized_work_order.excluded_from_periods {
                excluded_periods += &(period.to_string() + " ");
            }

            debug!(work_order_number = %work_order_number,
                info = "Work order updated",
                suggested_period = match &optimized_work_order.scheduled_period {
                    Some(period) => period.get_period_string().clone(),
                    None => "no suggested period".to_string()
                },
                locked_in_period = match &optimized_work_order.locked_in_period {
                    Some(period) => period.get_period_string().clone(),
                    None => "no lock on period".to_string()
                },
                excluded_periods = %excluded_periods
            );
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
        // All periods should be specified at the outset, this means that if a period is none then
        // the whole period vector should be updated. This is essential for the invariant to hold
        // across the whole scheduling environment. There is a more fundamental problem here. The
        // Problem is that either a resource or a period could be missing and hence this means that
        // there are multiple ways for the code to be wrong.

        // Should you be able to initialize a new resource? I think that this is a bad idea.
        // This should be given from the backend and not from the frontend. If this is true then it
        // means that the only way there can be a missing resource is if the period is missing.

        // dbg!(resource.clone());
        // dbg!(period.clone());
        // dbg!(self
        //     .resources_loading
        //     .get(&(resource.clone(), period.clone())));

        match self
            .resources_loading
            .get(&(resource.clone(), period.clone()))
        {
            Some(loading) => *loading,
            None => panic!(
                "This should not happen, all resources should be initialized at the outset or
                though messages from the frontend",
            ),
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

        let input_message = InputSchedulerMessage::new_test();

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
            None
        );

        scheduler_agent_algorithm.update_scheduler_algorithm_state(input_message);

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

    impl OptimizedWorkOrders {
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
}
