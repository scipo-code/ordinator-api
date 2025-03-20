pub mod functional_location;
pub mod priority;
pub mod revision;
pub mod system_condition;
pub mod work_order_text;
pub mod work_order_type;

use std::str::FromStr;

use anyhow::Result;
use serde::Deserialize;
use serde::Serialize;

use self::functional_location::FunctionalLocation;
use self::priority::Priority;
use self::revision::Revision;
use self::system_condition::SystemCondition;
use self::work_order_text::WorkOrderText;
use self::work_order_type::WorkOrderType;

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct WorkOrderInfo
{
    pub priority: Priority,
    pub work_order_type: WorkOrderType,
    pub functional_location: FunctionalLocation,
    pub work_order_text: WorkOrderText,
    pub revision: Revision,
    pub system_condition: SystemCondition,
    pub work_order_info_detail: WorkOrderInfoDetail,
}

#[derive(Default)]
pub struct WorkOrderInfoBuilder
{
    priority: Option<Priority>,
    work_order_type: Option<WorkOrderType>,
    functional_location: Option<FunctionalLocation>,
    work_order_text: Option<WorkOrderText>,
    revision: Option<Revision>,
    system_condition: Option<SystemCondition>,
    work_order_info_detail: Option<WorkOrderInfoDetail>,
}

impl WorkOrderInfo
{
    pub fn builder() -> WorkOrderInfoBuilder
    {
        WorkOrderInfoBuilder::default()
    }

    pub fn new(
        priority: Priority,
        work_order_type: WorkOrderType,
        functional_location: FunctionalLocation,
        work_order_text: WorkOrderText,
        revision: Revision,
        system_condition: SystemCondition,
        work_order_info_detail: WorkOrderInfoDetail,
    ) -> Self
    {
        WorkOrderInfo {
            priority,
            work_order_type,
            functional_location,
            work_order_text,
            revision,
            system_condition,
            work_order_info_detail,
        }
    }
}

impl WorkOrderInfoBuilder
{
    pub fn build(self) -> WorkOrderInfo
    {
        WorkOrderInfo {
            priority: self.priority.unwrap(),
            work_order_type: self.work_order_type.unwrap(),
            functional_location: self.functional_location.unwrap(),
            work_order_text: self.work_order_text.unwrap(),
            revision: self.revision.unwrap(),
            system_condition: self.system_condition.unwrap(),
            work_order_info_detail: self.work_order_info_detail.unwrap(),
        }
    }

    pub fn priority(mut self, priority: Priority) -> Self
    {
        self.priority = Some(priority);
        self
    }

    pub fn work_order_type(mut self, work_order_type: WorkOrderType) -> Self
    {
        self.work_order_type = Some(work_order_type);
        self
    }

    pub fn functional_location(mut self, functional_location: FunctionalLocation) -> Self
    {
        self.functional_location = Some(functional_location);
        self
    }

    pub fn work_order_text(mut self, work_order_text: WorkOrderText) -> Self
    {
        self.work_order_text = Some(work_order_text);
        self
    }

    pub fn revision(mut self, revision: Revision) -> Self
    {
        self.revision = Some(revision);
        self
    }

    pub fn system_condition(mut self, system_condition: SystemCondition) -> Self
    {
        self.system_condition = Some(system_condition);
        self
    }

    pub fn work_order_info_detail(mut self, work_order_info_detail: WorkOrderInfoDetail) -> Self
    {
        self.work_order_info_detail = Some(work_order_info_detail);
        self
    }

    pub fn functional_location_from_str(mut self, functional_location: &str) -> Self
    {
        self.functional_location = Some(FunctionalLocation::new(functional_location));
        self
    }

    pub fn revision_from_str(mut self, revision: &str) -> Self
    {
        self.revision = Some(Revision::new(revision));
        self
    }

    pub fn system_condition_from_str(mut self, system_condition: &str) -> Result<Self>
    {
        self.system_condition = Some(SystemCondition::from_str(system_condition)?);
        Ok(self)
    }
}

// WARN
// You should be careful with this here.
#[derive(Serialize, Deserialize, Clone, Debug, Default)]
pub struct WorkOrderInfoDetail
{
    pub subnetwork: String,
    pub maintenance_plan: String,
    pub planner_group: String,
    pub maintenance_plant: String,
    pub pm_collective: String,
    pub room: String,
}

impl WorkOrderInfoDetail
{
    pub fn new(
        subnetwork: String,
        maintenance_plan: String,
        planner_group: String,
        maintenance_plant: String,
        pm_collective: String,
        room: String,
    ) -> Self
    {
        Self {
            subnetwork,
            maintenance_plan,
            planner_group,
            maintenance_plant,
            pm_collective,
            room,
        }
    }
}
