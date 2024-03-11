use clap::Subcommand;
use shared_messages::LevelOfDetail;

/// The StatusCommands are mostly important for the schedulingenvironment.
#[derive(Subcommand, Debug)]
pub enum StatusCommands {
    WorkOrders {
        #[clap(subcommand)]
        work_orders: WorkOrders,
    },
    Workers,
    Time {},
}

/// We should put a lot of thought into the subcommand of the work orders.
#[derive(Subcommand, Debug)]
pub enum WorkOrders {
    /// Get the aggregated state of all work orders
    WorkOrderState { level_of_detail: LevelOfDetail },

    /// Get all details of a specific work order
    WorkOrder {
        work_order_number: u32,
        level_of_detail: LevelOfDetail,
    },
}
