#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct WorkOrderAnalytic {
    pub work_order_weight: u64,
    pub work_order_work: Work,
    pub work_load: HashMap<Resources, Work>,
    pub fixed: bool,
    pub vendor: bool,
    pub system_status_codes: SystemStatusCodes,
    pub user_status_codes: UserStatusCodes,
}

pub struct WorkOrderAnalyticBuilder {
    pub work_order_weight: u64,
    pub work_order_work: Work,
    pub work_load: HashMap<Resources, Work>,
    pub fixed: bool,
    pub vendor: bool,
    // TODO [ ]
    // You should make a builder for these if needed
    pub system_status_codes: SystemStatusCodes,
    // TODO [ ]
    // You should make a builder for these if needed
    pub user_status_codes: UserStatusCodes,
}

impl WorkOrderAnalyticBuilder {
    pub fn build(self, operations: HashMap<ActivityNumber, Operation>) -> WorkOrderAnalytic {
        WorkOrderAnalytic {
            work_order_weight: todo!(),
            work_order_work: todo!(),
            work_load: todo!(),
            fixed: todo!(),
            vendor: todo!(),
            system_status_codes: todo!(),
            user_status_codes: todo!(),
        }
    }
}

impl WorkOrderAnalytic {
    pub fn new(
        work_order_weight: u64,
        work_order_work: Work,
        work_load: HashMap<Resources, Work>,
        fixed: bool,
        vendor: bool,
        system_status_codes: SystemStatusCodes,
        user_status_codes: UserStatusCodes,
    ) -> Self {
        WorkOrderAnalytic {
            work_order_weight,
            work_order_work,
            work_load,
            fixed,
            vendor,
            system_status_codes,
            user_status_codes,
        }
    }
}
