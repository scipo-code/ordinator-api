#[derive(Subcommand, Debug)]
enum OrchestratorCommands {
    /// Get the status of a specific WorkOrder
    WorkOrder {
        work_order: u32,
    },
    Periods,
}