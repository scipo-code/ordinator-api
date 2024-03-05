#[derive(Subcommand, Debug)]
enum TacticalCommands {
    /// Get the status of the tactical agent
    Status,
    /// Get the objectives of the tactical agent
    Objectives,
}
