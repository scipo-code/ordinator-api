use clap::Subcommand;
use shared_types::{Asset, LevelOfDetail, LogLevel};

/// The StatusCommands are mostly important for the scheduling environment.
#[derive(Subcommand, Debug)]
pub enum StatusCommands {
    WorkOrders {
        #[clap(subcommand)]
        work_orders: WorkOrders,
    },
    Workers,
    Time {},
    Log {
        #[clap(subcommand)]
        level: LogLevel,
    },
    Profiling {
        #[clap(subcommand)]
        level: LogLevel,
    },
}

/// We should put a lot of thought into the subcommand of the work orders.
#[derive(Subcommand, Debug)]
pub enum WorkOrders {
    /// Get the aggregated state of all work orders
    WorkOrderState {
        asset: Asset,
        level_of_detail: LevelOfDetail,
    },

    /// Get all details of a specific work order
    WorkOrder {
        work_order_number: u32,
        level_of_detail: LevelOfDetail,
    },
}
