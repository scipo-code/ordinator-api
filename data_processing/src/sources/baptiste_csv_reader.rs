use chrono::{DateTime, Utc};
use shared_types::scheduling_environment::{
    work_order::{
        operation::{operation_info::NumberOfPeople, ActivityNumber, Work},
        priority::Priority,
        work_order_type::WorkOrderType,
        WorkOrderNumber,
    },
    SchedulingEnvironment,
};
use std::{
    collections::HashMap,
    error::Error,
    hash::Hash,
    path::{Path, PathBuf},
};

use serde::{de::DeserializeOwned, Deserialize};

use super::{baptiste_csv_reader_merges::create_work_orders, SchedulingEnvironmentFactory, SchedulingEnvironmentFactoryError};

pub struct TotalSap {
    file_path: PathBuf,
    number_of_strategic_periods: u64,
    number_of_tactical_periods: u64,
    number_of_days: u64,
}

impl TotalSap {
    pub fn new(
        file_path: PathBuf,
        number_of_strategic_periods: u64,
        number_of_tactical_periods: u64,
        number_of_days: u64,
    ) -> Self {
        Self {
            file_path,
            number_of_strategic_periods,
            number_of_tactical_periods,
            number_of_days,
        }
    }
}

impl SchedulingEnvironmentFactory<TotalSap> for SchedulingEnvironment {
    fn create_scheduling_environment(
        data_source: TotalSap,
    ) -> Result<SchedulingEnvironment, SchedulingEnvironmentFactoryError> {

        let time_environment = create_time_environment(&data_source);

        let worker_environment: WorkerEnvironment = WorkerEnvironment::new();

        let work_orders = create_work_orders(, , , , , , )

        let scheduling_environment =
            SchedulingEnvironment::new(work_orders, worker_environment, time_environment);
        Ok(scheduling_environment)


        
    }
}

pub enum ContainerType<C: CsvType> {
    HashMap(HashMap<C::KeyType, C>),
    Vec(Vec<C>),
}

pub fn populate_csv_structures<'a, C>(
    file_path: &'a str,
    container_type: &'a mut ContainerType<C>,
) -> Result<&'a mut ContainerType<C>, Box<dyn Error>>
where
    C: DeserializeOwned,
    C: CsvType,
{
    let csv_file = std::fs::File::open(file_path)?;
    let mut reader = csv::Reader::from_reader(csv_file);
    for row in reader.deserialize() {
        let value: C = row.unwrap();
        match container_type {
            ContainerType::HashMap(hash_map) => {
                let key = value.get_and_clone_key();

                hash_map.insert(key, value);
            }
            ContainerType::Vec(vec) => {
                vec.push(value);
            }
        }
    }
    Ok(container_type)
}

pub trait CsvType {
    type KeyType: PartialEq + Eq + Hash;

    fn get_and_clone_key(self: &Self) -> Self::KeyType;
}

pub type WOStatusId = String;
pub type WBSID = String;
pub type OPRRoutingNumber = String;
pub type WOObjectNumber = String;
pub type OPRObjectNumber = String;
pub type FLOCTechnicaID = String;

#[derive(Deserialize, Debug)]
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
}

#[derive(Deserialize, Debug)]
#[allow(non_snake_case, dead_code)]
pub struct WorkOperationsCsv {
    pub OPR_Routing_Number: String,
    pub OPR_Counter: String,
    pub OPR_WBS_ID: String,
    pub OPR_Workers_Numbers: NumberOfPeople,
    pub OPR_Planned_Work: Work,
    pub OPR_Actual_Work: Work,
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
    fn get_and_clone_key(self: &Self) -> Self::KeyType {
        self.OPR_Routing_Number.clone()
    }

    type KeyType = String;
}

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
}

#[allow(non_snake_case, dead_code)]
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
    fn get_and_clone_key(self: &Self) -> Self::KeyType {
        self.OPR_Object_Number.clone()
    }

    type KeyType = String;
}

#[allow(non_snake_case, dead_code)]
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
}

#[allow(non_snake_case, dead_code)]
pub struct FunctionalLocationsCsv {
    pub FLOC_Technical_ID: String,
    pub FLOC_Functional_ID: String,
    pub FLOC_Name: String,
    pub ILOAN_Location_Room: String,
    pub FLOC_Plant_Code: String,
}

impl CsvType for FunctionalLocationsCsv {
    type KeyType = String;

    fn get_and_clone_key(self: &Self) -> Self::KeyType {
        self.FLOC_Technical_ID.clone()
    }
}

#[allow(non_snake_case, dead_code)]
pub struct WorkOrdersCsv {
    pub WO_Number: String,
    pub WO_Priority: Priority,
    pub WO_Functional_Location_Number: String,
    pub WO_Plan_Maintenance_Number: String,
    pub WO_Planner_Group: String,
    pub WO_WBS_ID: String,
    pub WO_Revision: String,
    pub WO_Activity_Type: String,
    pub WO_Scheduled_Start_Date: String,
    pub WO_Operation_ID: String,
    pub WO_Order_Type: WorkOrderType,
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
    pub WO_Basic_Start_Date: DateTime<Utc>,
    pub WO_Basic_End_Date: DateTime<Utc>,
    pub WO_Earliest_Allowed_Start_Date: DateTime<Utc>,
    pub WO_Latest_Allowed_Finish_Date: DateTime<Utc>,
    pub WO_SubNetwork_ID: String,
}

impl CsvType for WorkOrdersCsv {
    fn get_and_clone_key(self: &Self) -> Self::KeyType {
        self.WO_Number.clone().parse::<Self::KeyType>().unwrap()
    }

    type KeyType = WorkOrderNumber;
}

pub struct WorkOrdersStatusCsvAggregated {
    pub inner: HashMap<WOObjectNumber, String>,
}

impl WorkOrdersStatusCsvAggregated {
    pub fn new(work_orders_status: Vec<WorkOrdersStatusCsv>) -> Self {
        let mut work_order_stutus_aggregated: HashMap<String, String> = HashMap::new();

        for work_order_status in work_orders_status {
            work_order_stutus_aggregated
                .entry(work_order_status.WO_Object_Number)
                .and_modify(|entry| {
                    entry.push_str(&work_order_status.WO_E_Status_Code);
                    entry.push_str(&work_order_status.WO_I_Status_Code);
                })
                .or_insert(
                    work_order_status.WO_E_Status_Code + &work_order_status.WO_I_Status_Code,
                );
        }

        Self {
            inner: work_order_stutus_aggregated,
        }
    }
}
