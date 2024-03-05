use clap::Subcommand;
use shared_messages::SystemMessages;

#[derive(Subcommand, Debug)]
pub enum SapCommands {
    /// Extract scheduling relevant data from SAP (requires user authorization)
    ExtractFromSap,

    /// Push the 4M+ (strategic) optimized data to SAP (requires user authorization)
    PushStrategicToSap,

    /// Push the 5W (tactical) optimized data to SAP (requires user authorization)
    PushTacticalToSap,

    /// Access the 2WF (operational) opmized data (requires user authorization)
    Operational,
}

impl SapCommands {
    pub fn execute(&self) -> SystemMessages {
        match self {
            SapCommands::ExtractFromSap => {
                let url = "https://help.sap.com/docs/SAP_BUSINESSOBJECTS_BUSINESS_INTELLIGENCE_PLATFORM/9029a149a3314dadb8418a2b4ada9bb8/099046a701cb4014b20123ae31320959.html"; // Replace with the actual SAP authorization URL

                if webbrowser::open(url).is_ok() {
                    println!("Opened {} in the default web browser.", url);
                } else {
                    // There was an error opening the URL
                    println!("Failed to open {}.", url);
                }
                SystemMessages::Sap
            }
            SapCommands::PushStrategicToSap => {
                todo!()
            }
            SapCommands::PushTacticalToSap => {
                todo!()
            }
            SapCommands::Operational => {
                todo!()
            }
        }
    }
}
