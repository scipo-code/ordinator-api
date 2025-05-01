pub mod time_environment;
pub mod work_order;
pub mod worker_environment;

use std::fmt::Display;
use std::fmt::{self};
use std::sync::Arc;
use std::sync::Mutex;

use anyhow::Result;
use serde::Deserialize;
use serde::Serialize;
use strum::IntoEnumIterator;
use strum_macros::EnumIter;
use time_environment::TimeEnvironmentBuilder;
use work_order::WorkOrders;
use work_order::WorkOrdersBuilder;

use self::time_environment::TimeEnvironment;
use self::worker_environment::WorkerEnvironment;

#[derive(Deserialize, Serialize, Debug)]
pub struct SchedulingEnvironment {
    pub work_orders: WorkOrders,
    pub worker_environment: WorkerEnvironment,
    pub time_environment: TimeEnvironment,
    // material
}
pub struct SchedulingEnvironmentBuilder {
    work_orders: Option<WorkOrders>,
    worker_environment: Option<WorkerEnvironment>,
    time_environment: Option<TimeEnvironment>,
}

impl SchedulingEnvironment {
    pub fn builder() -> SchedulingEnvironmentBuilder {
        SchedulingEnvironmentBuilder {
            work_orders: None,
            worker_environment: None,
            time_environment: None,
        }
    }
}

pub trait IntoSchedulingEnvironment {
    type S: SystemConfigurationTrait;

    fn into_scheduling_environment(
        self,
        system_configuration: &Self::S,
    ) -> Result<Arc<Mutex<SchedulingEnvironment>>>;
}

pub trait SystemConfigurationTrait {}

pub trait DatabaseConfigurationTrait {}

impl SchedulingEnvironmentBuilder {
    // QUESTION
    // Do you believe that this is the most appropriate way of structuring the code
    // here? Yes I think that this is the best way of doing it.
    pub fn build(self) -> Arc<Mutex<SchedulingEnvironment>> {
        Arc::new(Mutex::new(SchedulingEnvironment {
            work_orders: self
                .work_orders
                .expect("You should build the WorkOrders with the correct parameters injected."),
            worker_environment: self.worker_environment.unwrap_or_default(),
            time_environment: self.time_environment.unwrap_or_default(),
        }))
    }

    pub fn time_environment(mut self, time_environment: TimeEnvironment) -> Self {
        self.time_environment = Some(time_environment);
        self
    }

    pub fn time_environment_builder<F>(mut self, f: F) -> Self
    where
        F: FnOnce(&mut TimeEnvironmentBuilder) -> &mut TimeEnvironmentBuilder,
    {
        let mut time_environment_builder = TimeEnvironmentBuilder::default();

        f(&mut time_environment_builder);

        self.time_environment = Some(time_environment_builder.build());
        self
    }

    pub fn worker_environment(mut self, worker_environment: WorkerEnvironment) -> Self {
        self.worker_environment = Some(worker_environment);
        self
    }

    pub fn work_orders(mut self, work_orders: WorkOrders) -> Self {
        self.work_orders = Some(work_orders);
        self
    }

    pub fn work_orders_builder<F>(&mut self, f: F) -> &mut Self
    where
        F: FnOnce(&mut WorkOrdersBuilder) -> &mut WorkOrdersBuilder,
    {
        let mut work_orders_builder = WorkOrders::builder();

        f(&mut work_orders_builder);

        self.work_orders = Some(work_orders_builder.build());
        self
    }
}

impl fmt::Display for SchedulingEnvironment {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        // let workers = self
        //     .worker_environment
        //     .actor_specification
        //     .iter
        //     .operational
        //     .len();
        // write!(
        //     f,
        //     "The Scheduling Environment is currently comprised of
        // \n  number of work orders: {}
        // \n  number of worker entries: {}
        // \n  number of strategic periods: {},
        // \n  number of tactical days: {}",
        //     self.work_orders.inner.len(),
        //     workers,
        //     self.time_environment.strategic_periods.len(),
        //     self.time_environment.tactical_days.len(),
        // )?;
        Ok(())
    }
}
#[derive(PartialEq, Eq, Hash, Serialize, Deserialize, Debug, Clone, EnumIter)]
pub enum Asset {
    DF,
    DM,
    DE,
    GO,
    HB,
    HC,
    HD,
    HW,
    KR,
    RO,
    RF,
    SK,
    SV,
    TE,
    TS,
    VA,
    VB,
    Unknown,
    Test,
}

#[derive(Serialize)]
pub struct AssetNames {
    value: String,
    label: String,
}

impl Display for Asset {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Asset::DF => write!(f, "DF"),
            Asset::DM => write!(f, "DM"),
            Asset::DE => write!(f, "DE"),
            Asset::GO => write!(f, "GO"),
            Asset::HB => write!(f, "HB"),
            Asset::HC => write!(f, "HC"),
            Asset::HD => write!(f, "HD"),
            Asset::HW => write!(f, "HW"),
            Asset::KR => write!(f, "KR"),
            Asset::RO => write!(f, "RO"),
            Asset::RF => write!(f, "RF"),
            Asset::SK => write!(f, "SK"),
            Asset::SV => write!(f, "SV"),
            Asset::TE => write!(f, "TE"),
            Asset::TS => write!(f, "TS"),
            Asset::VA => write!(f, "VA"),
            Asset::VB => write!(f, "VB"),
            Asset::Test => write!(f, "TEST"),
            Asset::Unknown => write!(f, "Unknown"),
        }
    }
}

impl Asset {
    pub fn new_from_string(asset_string: &str) -> Option<Asset> {
        match asset_string {
            "DF" => Some(Asset::DF),
            "DM" => Some(Asset::DM),
            "DE" => Some(Asset::DE),
            "GO" => Some(Asset::GO),
            "HB" => Some(Asset::HB),
            "HC" => Some(Asset::HC),
            "HD" => Some(Asset::HD),
            "HW" => Some(Asset::HW),
            "KR" => Some(Asset::KR),
            "RO" => Some(Asset::RO),
            "RF" => Some(Asset::RF),
            "SK" => Some(Asset::SK),
            "SV" => Some(Asset::SV),
            "TE" => Some(Asset::TE),
            "TS" => Some(Asset::TS),
            "VA" => Some(Asset::VA),
            "VB" => Some(Asset::VB),
            _ => None,
        }
    }

    pub fn convert_to_asset_names() -> Vec<AssetNames> {
        let mut vec = Vec::new();
        for asset in Asset::iter() {
            let asset_name = AssetNames {
                value: asset.to_string(),
                label: asset.to_string(),
            };
            vec.push(asset_name);
        }
        vec
    }
}
