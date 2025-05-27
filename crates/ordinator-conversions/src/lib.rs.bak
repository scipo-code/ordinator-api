use crate::supervisor::SupervisorObjectiveValue;

impl From<SharedSolution> for ApiSolution {
    fn from(_value: SharedSolution) -> Self {
        ApiSolution {
            strategic: "NEEDS TO BE IMPLEMENTED".to_string(),
            tactical: "NEEDS TO BE IMPLEMENTED".to_string(),
            supervisor: "NEEDS TO BE IMPLEMENTED".to_string(),
            operational: "NEEDS TO BE IMPLEMENTED".to_string(),
        }
    }
}

impl From<TomlTimeInterval> for TimeInterval {
    fn from(value: TomlTimeInterval) -> Self {
        Self {
            start: NaiveTime::parse_from_str(&value.start.to_string(), "%H:%M:%S").unwrap(),
            end: NaiveTime::parse_from_str(&value.end.to_string(), "%H:%M:%S").unwrap(),
        }
    }
}

impl WorkOrderResponse {
    pub fn new(
        work_order: &WorkOrder,
        api_solution: ApiSolution,
        periods: &[Period],
        work_order_configurations: &WorkOrderConfigurations,
        material_to_period: &MaterialToPeriod,
    ) -> Self {
        // WARN
        // Crucial lesson here. Derived needed information with functions allows you to
        // expose weak structures in data flow. Below we see that introducing a function
        // makes the code require an argument.
        // QUESTION
        // Do you even need `Periods` can they not always be derived instead? I think that
        // they can.
        let earliest_period = work_order
            .earliest_allowed_start_period(periods, material_to_period)
            .clone();

        let work_order_info = work_order.work_order_info.clone();
        let work_order_work_load = work_order.work_order_load();
        let vendor = work_order.vendor();
        // This is a good sign. You should be able to provide the work_order_configurations for this
        // and the `MessageHandler` trait has to be updated.
        let weight = work_order.work_order_value(work_order_configurations);
        let system_status_codes = work_order.work_order_analytic.system_status_codes.clone();
        let user_status_codes = work_order.work_order_analytic.user_status_codes.clone();

        Self {
            earliest_period,
            work_order_info,
            vendor,
            weight,
            work_order_work_load,
            system_status_codes,
            user_status_codes,
            api_solution,
        }
    }
}
// TODO [x]
// This is a low level type it should not be in here you are creating
// spaghetti code. Now you know what to do about this type which is really
// really cool.
impl From<StrategicObjectiveValue> for StrategicObjectiveValueResponse {
    fn from(value: StrategicObjectiveValue) -> Self {
        todo!()
    }
}

impl From<SupervisorSolution> for SupervisorResponseStatus {
    fn from(value: SupervisorSolution) -> Self {
        todo!()
    }
}
impl SupervisorResponseStatus {
    pub fn new(
        main_work_center: Vec<Id>,
        delegated_work_order_activities: usize,
        objective: SupervisorObjectiveValue,
    ) -> Self {
        Self {
            supervisor_resource: main_work_center,
            delegated_work_order_activities,
            objective,
        }
    }
}
// This should be a JSON formatted struct that is send into the system.
// You have made this wrong all along... You were arrogant and you are
// paying the prize for it deeply.
impl TacticalResourceRequest {
    pub fn new_set_resources(resources: TacticalResources) -> Self {
        TacticalResourceRequest::SetResources(resources)
    }
}
impl tacticalresponsestatus {
    pub fn new(objective: tacticalobjectivevalue, time_horizon: vec<day>) -> self {
        self {
            objective,
            time_horizon,
        }
    }
}
