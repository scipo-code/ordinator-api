#[derive(Subcommand, Debug)]
enum SapCommands {
    /// Extract scheduling relevant data from SAP (requires user authorization)
    ExtractFromSap,

    /// Push the 4M+ (strategic) optimized data to SAP (requires user authorization)
    PushStrategicToSap,

    /// Push the 5W (tactical) optimized data to SAP (requires user authorization)
    PushTacticalToSap,

    /// Access the 2WF (operational) opmized data (requires user authorization)
    Operational,
}
