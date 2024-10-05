use shared_types::scheduling_environment::{
    work_order::{
        operation::{operation_info::NumberOfPeople, ActivityNumber},
        work_order_type::WorkOrderType,
        WorkOrderActivity, WorkOrderNumber,
    },
    worker_environment::WorkerEnvironment,
    SchedulingEnvironment,
};
use std::{collections::HashMap, error::Error, fs::File, hash::Hash, path::PathBuf};

use serde::{de::DeserializeOwned, Deserialize};

use super::{
    baptiste_csv_reader_merges::load_csv_data, create_time_environment,
    SchedulingEnvironmentFactory, SchedulingEnvironmentFactoryError, TimeInput,
};

pub struct TotalSap {
    file_path: PathBuf,
}

impl TotalSap {
    pub fn new(file_path: PathBuf) -> Self {
        Self { file_path }
    }
}

impl SchedulingEnvironmentFactory<TotalSap> for SchedulingEnvironment {
    fn create_scheduling_environment(
        data_source: TotalSap,
        time_input: TimeInput,
    ) -> Result<SchedulingEnvironment, SchedulingEnvironmentFactoryError> {
        let time_environment = create_time_environment(&time_input);

        let worker_environment: WorkerEnvironment = WorkerEnvironment::new();

        let work_orders = load_csv_data(data_source.file_path, &time_environment.strategic_periods);

        let scheduling_environment =
            SchedulingEnvironment::new(work_orders, worker_environment, time_environment);
        Ok(scheduling_environment)
    }
}

pub fn populate_csv_structures<'a, C>(file_path: PathBuf) -> Result<C::Container, Box<dyn Error>>
where
    C: DeserializeOwned + CsvType + std::fmt::Debug,
    C::Container: Default,
{
    let csv_file: File = std::fs::File::open(file_path)?;
    dbg!(&csv_file);
    let mut reader = csv::Reader::from_reader(csv_file);
    let mut container = C::Container::default();
    for row in reader.deserialize() {
        let value: C = row.unwrap();
        let key = value.get_and_clone_key();
        C::make_entry(key, &mut container, value);
    }
    Ok(container)
}

pub trait CsvType {
    type KeyType: PartialEq + Eq + Hash;
    type Container;

    fn get_and_clone_key(self: &Self) -> Self::KeyType;

    fn make_entry(key: Self::KeyType, container: &mut Self::Container, value: Self);
}

pub type WOStatusId = String;
pub type WBSID = String;
pub type OPRRoutingNumber = String;
pub type OPRCounter = u64;
pub type WOObjectNumber = String;
pub type OPRObjectNumber = String;
pub type FLOCTechnicaID = u64;

#[derive(Clone, Deserialize, Debug)]
#[allow(non_snake_case, dead_code)]
pub struct WorkCenterCsv {
    pub WBS_ID: WBSID,
    pub WBS_Name: String,
    pub WBS_Plant: String,
    pub WBS_Full_name: String,
}

impl CsvType for WorkCenterCsv {
    type KeyType = String;
    fn get_and_clone_key(self: &Self) -> Self::KeyType {
        self.WBS_ID.clone()
    }

    type Container = HashMap<Self::KeyType, Self>;

    fn make_entry(key: Self::KeyType, container: &mut Self::Container, value: Self) {
        container.insert(key, value);
    }
}

#[derive(Default, Deserialize, Debug, Clone)]
#[allow(non_snake_case, dead_code)]
pub struct WorkOperationsCsv {
    pub OPR_Routing_Number: String,
    pub OPR_Counter: u64,
    pub OPR_WBS_ID: String,
    pub OPR_Workers_Numbers: NumberOfPeople,
    pub OPR_Planned_Work: String,
    pub OPR_Actual_Work: String,
    pub OPR_Start_Date: String,
    pub OPR_Start_Time: String,
    pub OPR_End_Date: String,
    pub OPR_End_Time: String,
    pub OPR_Scheduled_Work: String,
    pub OPR_Description: String,
    pub OPR_Activity_Number: ActivityNumber,
    pub OPR_Status_ID: String,
}

impl CsvType for WorkOperationsCsv {
    type KeyType = (String, u64);
    type Container = HashMap<String, HashMap<u64, Self>>;

    fn get_and_clone_key(self: &Self) -> Self::KeyType {
        (self.OPR_Routing_Number.clone(), self.OPR_Counter.clone())
    }

    fn make_entry(key: Self::KeyType, container: &mut Self::Container, value: Self) {
        if value.OPR_WBS_ID == "0" {
            return;
        }
        let key_0 = key.0.trim_end_matches(".0").to_string();
        container
            .entry(key_0)
            .and_modify(|inner_hash_map| {
                inner_hash_map.entry(key.1.clone()).or_insert(value.clone());
            })
            .or_insert_with(|| {
                let mut new_hash_map = HashMap::new();
                new_hash_map.insert(key.1, value);
                new_hash_map
            });
    }
}

#[derive(Default, Clone, Deserialize, Debug)]
#[allow(non_snake_case, dead_code)]
pub struct WorkOrdersStatusCsv {
    pub WO_Object_Number: String,
    pub WO_Status_ID: String,
    pub WO_Status_Profile: String,
    pub WO_E_Status_Code: String,
    pub WO_E_Status_Message: String,
    pub WO_I_Status_Code: String,
    pub WO_I_Status_Message: String,
}

impl CsvType for WorkOrdersStatusCsv {
    fn get_and_clone_key(self: &Self) -> Self::KeyType {
        self.WO_Object_Number.clone()
    }

    type KeyType = String;

    type Container = Vec<Self>;

    fn make_entry(key: Self::KeyType, container: &mut Self::Container, value: Self) {
        container.push(value);
    }
}

#[allow(non_snake_case, dead_code)]
#[derive(Clone, Deserialize, Debug)]
pub struct OperationsStatusCsv {
    pub OPR_Object_Number: String,
    pub OPR_Status_ID: String,
    pub OPR_Status_Profile: String,
    pub OPR_E_Status_Code: String,
    pub OPR_E_Status_Message: String,
    pub OPR_I_Status_Code: String,
    pub OPR_I_Status_Message: String,
}

#[allow(non_snake_case, dead_code)]
impl CsvType for OperationsStatusCsv {
    type KeyType = String;

    type Container = Vec<Self>;

    fn get_and_clone_key(self: &Self) -> Self::KeyType {
        self.OPR_Object_Number.clone()
    }

    fn make_entry(key: Self::KeyType, container: &mut Self::Container, value: Self) {
        container.push(value);
    }
}

#[allow(non_snake_case, dead_code)]
#[derive(Clone, Deserialize, Debug)]
pub struct SecondaryLocationsCsv {
    pub PM_Object_Number: String,
    pub PM_Functional_Location: String,
    pub PM_Object_Sorting: String,
    pub PM_Object_Usage: String,
}

impl CsvType for SecondaryLocationsCsv {
    fn get_and_clone_key(self: &Self) -> Self::KeyType {
        todo!()
    }

    type KeyType = String;

    type Container = Vec<Self>;

    fn make_entry(key: Self::KeyType, container: &mut Self::Container, value: Self) {
        todo!()
    }
}

#[allow(non_snake_case, dead_code)]
#[derive(Clone, Deserialize, Debug)]
pub struct FunctionalLocationsCsv {
    pub FLOC_Technical_ID: FLOCTechnicaID,
    pub FLOC_Functional_ID: String,
    pub FLOC_Name: String,
    pub ILOAN_Location_Room: String,
    pub FLOC_Plant_Code: String,
}

impl CsvType for FunctionalLocationsCsv {
    type KeyType = u64;

    fn get_and_clone_key(self: &Self) -> Self::KeyType {
        self.FLOC_Technical_ID.clone()
    }

    type Container = HashMap<Self::KeyType, Self>;

    fn make_entry(key: Self::KeyType, container: &mut Self::Container, value: Self) {
        container.entry(key).or_insert(value);
    }
}

#[allow(non_snake_case, dead_code)]
#[derive(Clone, Deserialize, Debug)]
pub struct WorkOrdersCsv {
    pub WO_Number: u64,
    pub WO_Priority: String,
    pub WO_Functional_Location_Number: u64,
    pub WO_Plan_Maintenance_Number: String,
    pub WO_Planner_Group: String,
    pub WO_WBS_ID: String,
    pub WO_Revision: String,
    pub WO_Activity_Type: String,
    pub WO_Scheduled_Start_Date: String,
    pub WO_Operation_ID: String,
    pub WO_Order_Type: String,
    pub WO_Header_Description: String,
    pub WO_Phase_Order_Created: String,
    pub WO_Phase_Order_Released: String,
    pub WO_Status_ID: String,
    pub WO_Original_Deadline: String,
    pub WO_Notification_Number: String,
    pub WO_Notification_Malfunction_Started: String,
    pub WO_Notification_Created: String,
    pub WO_Notification: String,
    pub WO_Maintenance_Plan_Name: String,
    pub WO_System_Condition: String,
    pub WO_Basic_Start_Date: String,
    pub WO_Basic_End_Date: String,
    pub WO_Earliest_Allowed_Start_Date: String,
    pub WO_Latest_Allowed_Finish_Date: String,
    pub WO_SubNetwork_ID: String,
}

impl CsvType for WorkOrdersCsv {
    type KeyType = WorkOrderNumber;

    type Container = HashMap<Self::KeyType, Self>;

    fn get_and_clone_key(self: &Self) -> Self::KeyType {
        WorkOrderNumber(self.WO_Number.clone())
    }

    fn make_entry(key: Self::KeyType, container: &mut Self::Container, value: Self) {
        // This is custom logic needed to handle incorrectly formatted csv data
        // This is not a permanent solution
        if ["", "0"].contains(&value.WO_Earliest_Allowed_Start_Date.trim_end_matches(".0")) {
            return;
        }
        if ["", "0"].contains(&value.WO_Latest_Allowed_Finish_Date.trim_end_matches(".0")) {
            return;
        }
        if ["", "0"].contains(&value.WO_Basic_Start_Date.trim_end_matches(".0")) {
            return;
        }
        if ["", "0"].contains(&value.WO_Basic_End_Date.trim_end_matches(".0")) {
            return;
        }
        if ["", "0"].contains(&value.WO_System_Condition.trim_end_matches(".0")) {
            return;
        }
        if ["", "0"].contains(&value.WO_Order_Type.trim_end_matches(".0")) {
            return;
        }
        if !WorkOrderType::valid_work_order_type(&value.WO_Order_Type) {
            return;
        }

        container.insert(key, value);
    }
}

#[derive(Clone)]
pub struct WorkOrdersStatusCsvAggregated {
    pub inner: HashMap<WOObjectNumber, String>,
}

impl WorkOrdersStatusCsvAggregated {
    pub fn new(work_orders_status: Vec<WorkOrdersStatusCsv>) -> Self {
        let mut work_order_status_aggregated: HashMap<String, String> = HashMap::new();

        for work_order_status in work_orders_status {
            work_order_status_aggregated
                .entry(work_order_status.WO_Object_Number)
                .and_modify(|entry| {
                    entry.push_str(&work_order_status.WO_E_Status_Code);
                    entry.push_str(&work_order_status.WO_I_Status_Code);
                })
                .or_insert(
                    work_order_status.WO_E_Status_Code + &work_order_status.WO_I_Status_Code,
                );
        }

        work_order_status_aggregated.retain(|_, agg_sta| agg_sta.contains("REL"));

        Self {
            inner: work_order_status_aggregated,
        }
    }
}

#[derive(Clone)]
pub struct OperationsStatusCsvAggregated {
    pub inner: HashMap<OPRObjectNumber, String>,
}

impl OperationsStatusCsvAggregated {
    pub fn new(operations_status: Vec<OperationsStatusCsv>) -> Self {
        let mut operations_status_aggregated: HashMap<String, String> = HashMap::new();

        for operations_status in operations_status {
            operations_status_aggregated
                .entry(operations_status.OPR_Object_Number)
                .and_modify(|entry| {
                    entry.push_str(&operations_status.OPR_E_Status_Code);
                    entry.push_str(&operations_status.OPR_I_Status_Code);
                })
                .or_insert(
                    operations_status.OPR_E_Status_Code + &operations_status.OPR_I_Status_Code,
                );
        }

        Self {
            inner: operations_status_aggregated,
        }
    }
}
pub struct WorkOperations {
    pub inner: HashMap<WorkOrderNumber, HashMap<ActivityNumber, WorkOperationsCsv>>,
}

impl WorkOperations {
    pub fn new(
        work_orders_csv: &HashMap<WorkOrderNumber, WorkOrdersCsv>,
        operations_csv: &HashMap<OPRRoutingNumber, HashMap<OPRCounter, WorkOperationsCsv>>,
    ) -> Self {
        let mut work_operations = HashMap::new();

        for work_order_csv in work_orders_csv.iter() {
            let wo_operation_id = work_order_csv.1.WO_Operation_ID.trim_end_matches(".0");
            dbg!();
            if let Some(value) = operations_csv.get(wo_operation_id) {
                let mut inner_hash_map = HashMap::new();
                for operation_csv in value {
                    inner_hash_map
                        .insert(operation_csv.1.OPR_Activity_Number, operation_csv.1.clone());
                }
                work_operations.insert(*work_order_csv.0, inner_hash_map);
            }
        }

        Self {
            inner: work_operations,
        }
    }
}

#[cfg(test)]
mod tests {

    use super::*;

    #[test]
    fn test_populate_csv_structures() {
        let mut path = PathBuf::new();

        path.push("../temp_scheduling_environment_database/mid_work_operations.csv");
        populate_csv_structures::<WorkOperationsCsv>(path).unwrap();
    }
}
