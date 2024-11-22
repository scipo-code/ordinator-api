/// This implementation will update the current state of the scheduler agent.
///
/// I have an issue with how the scheduled work orders should be handled. I think that there are
/// multiple approaches to solving this problem. The queue idea is good but then I would have to
/// update the other queues if the work order is present in one of those queues. I could also just
/// bypass the whole thing. Hmm... I have misunderstood something here. Should I make the solution
/// scheduled_work_orders are the once that are scheduled. But there is also the question of the
/// scheduled field in the central data structure. I should find out where that comes from and
///
/// So here we update the state of the application, but what about the queues? I after the work
/// order has been scheduled in the front end we need to update the queues. As well so that the
/// work order is scheduled through the process. We should add the work order to the unloading point
/// queue but what will happen when the work order is unscheduled again at a later point? This is
/// much more difficult to reason about. I think that the best approach is
///
/// All of this should be handled in the update scheduler state function. There can be no other way
/// Remember that if this becomes complex we should refactor the code.

#[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]
pub struct SchedulingOverviewData {
    scheduled_period: String,
    scheduled_start: String,
    unloading_point: String,
    material_date: String,
    work_order_number: u32,
    activity: String,
    work_center: String,
    work_remaining: String,
    number: u32,
    notes_1: String,
    notes_2: String,
    order_description: String,
    object_description: String,
    order_user_status: String,
    order_system_status: String,
    functional_location: String,
    revision: String,
    earliest_start_datetime: String,
    earliest_finish_datetime: String,
    earliest_allowed_starting_date: String,
    latest_allowed_finish_date: String,
    order_type: String,
    priority: String,
}

/// Now the problem is that the many work orders may not even get a status, in this approach.
/// This is an issue. Now when we get the work_order_number the entry could be non-existent.
///
/// Here it is not the alrithm that should be the one that should be used to generate the overview
/// I think that here we should use the work orders from the scheduling environment to extract the
/// scheduling environment correctly. This is a good point.
impl StrategicAgent {
    fn extract_state_to_scheduler_overview(&self) -> Vec<SchedulingOverviewData> {
        let mut scheduling_overview_data: Vec<SchedulingOverviewData> = Vec::new();

        let work_orders = self
            .scheduling_environment
            .lock()
            .unwrap()
            .clone_work_orders();

        for (work_order_number, mut work_order) in work_orders.inner {
            for (operation_number, operation) in work_order.get_operations().clone() {
                let scheduling_overview_data_item = SchedulingOverviewData {
                    scheduled_period: match self
                        .scheduler_agent_algorithm
                        .get_optimized_work_order(&work_order_number)
                    {
                        Some(order_period) => match order_period.get_scheduled_period().as_ref() {
                            Some(scheduled_period) => scheduled_period.get_period_string().clone(),
                            None => "not scheduled".to_string(),
                        },
                        None => "not scheduled".to_string(),
                    },
                    scheduled_start: work_order.get_order_dates().basic_start_date.to_string(),
                    unloading_point: work_order.get_unloading_point().clone().string,
                    material_date: match work_order.get_status_codes().material_status {
                        MaterialStatus::Smat => "SMAT".to_string(),
                        MaterialStatus::Nmat => "NMAT".to_string(),
                        MaterialStatus::Cmat => "CMAT".to_string(),
                        MaterialStatus::Wmat => "WMAT".to_string(),
                        MaterialStatus::Pmat => "PMAT".to_string(),
                        MaterialStatus::Unknown => "Implement control tower".to_string(),
                    },
                    work_order_number,
                    activity: operation_number.clone().to_string(),
                    work_center: operation.work_center.variant_name(),
                    work_remaining: operation.work_remaining.to_string(),
                    number: operation.number,
                    notes_1: work_order.get_order_text().notes_1.clone(),
                    notes_2: work_order.get_order_text().notes_2.clone().to_string(),
                    order_description: work_order.get_order_text().order_description.clone(),
                    object_description: work_order.get_order_text().object_description.clone(),
                    order_user_status: work_order.get_order_text().order_user_status.clone(),
                    order_system_status: work_order.get_order_text().order_system_status.clone(),
                    functional_location: work_order.get_functional_location().clone().string,
                    revision: work_order.get_revision().clone().string,
                    earliest_start_datetime: operation.earliest_start_datetime.to_string(),
                    earliest_finish_datetime: operation.earliest_finish_datetime.to_string(),
                    earliest_allowed_starting_date: work_order
                        .get_order_dates()
                        .earliest_allowed_start_date
                        .to_string(),
                    latest_allowed_finish_date: work_order
                        .get_order_dates()
                        .latest_allowed_finish_date
                        .to_string(),
                    order_type: match work_order.get_order_type().clone() {
                        WorkOrderType::Wdf(_wdf_priority) => "WDF".to_string(),
                        WorkOrderType::Wgn(_wgn_priority) => "WGN".to_string(),
                        WorkOrderType::Wpm(_wpm_priority) => "WPM".to_string(),
                        WorkOrderType::Wro(_) => "WRO".to_string(),
                        WorkOrderType::Other => "Missing Work Order Type".to_string(),
                    },
                    priority: match work_order.get_priority().clone() {
                        Priority::IntValue(i) => i.to_string(),
                        Priority::StringValue(s) => s.to_string(),
                    },
                };
                scheduling_overview_data.push(scheduling_overview_data_item);
            }
        }
        scheduling_overview_data
    }
}

/// This is a good point. We should make the type as narrow as possible. This means that we should
/// implement everything that is algorithm specific in the SchedulerAgentAlgorithm. This is a
/// crucial insight.

/// This function should be reformulated? I think that we should make sure to create in such a way
/// that. We need an inner hashmap for each of the different
fn transform_hashmap_to_nested_hashmap(
    resources: HashMap<Resources, HashMap<Period, f64>>,
) -> HashMap<Resources, HashMap<String, f64>> {
    let mut resources_hash_map = HashMap::new();
    for (resource, periods) in resources {
        let mut periods_hash_map = HashMap::new();
        for (period, capacity) in periods {
            periods_hash_map.insert(period.get_period_string(), capacity);
        }
        resources_hash_map.insert(resource, periods_hash_map);
    }
    resources_hash_map
}
