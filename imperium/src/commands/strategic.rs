#[derive(Subcommand, Debug)]
enum StrategicCommands {
    /// overview of the strategic agent
    Status {
        #[clap(subcommand)]
        subcommand: Option<StatusStrategic>,
    },
    /// Scheduling commands
    Scheduling {
        #[clap(subcommand)]
        subcommand: Option<SchedulingSubcommands>,
    },
    /// Resources commands
    Resources {
        #[clap(subcommand)]
        subcommand: Option<ResourcesSubcommands>,
    },
}

#[derive(Subcommand, Debug)]
enum ResourcesSubcommands {
    /// Get the loading of the resources
    Loading {
        periods_end: String,
        select_resources: Option<Vec<String>>,
    },

    /// Get the capacity of the resources
    Capacity {
        periods_end: String,
        select_resources: Option<Vec<String>>,
    },

    /// Set the capacity of a resource
    SetCapacity {
        resource: String,
        period: String,
        capacity: u32,
    },

    /// Set the capacity policy of a resource (used for operation)
    SetCapacityPolicy { resource: String, capacity: u32 },
    /// Set the capacity policy to default (used for testing)
    SetCapacityPolicyDefault,
}

#[derive(Subcommand, Debug)]
enum StatusStrategic {
    /// List all work orders in a given period
    WorkOrders { period: String },
}

#[derive(Subcommand, Debug)]
enum SchedulingSubcommands {
    /// Schedule a specific work order in a given period
    Schedule(WorkOrderSchedule),
    /// Lock a period from any scheduling changes
    PeriodLock { period: String },
    /// Exclude a work order from a period
    Exclude { work_order: u32, period: String },
}
