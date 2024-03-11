use crate::models::work_order::WorkOrder;
use std::fmt;
use std::fmt::Write;

impl fmt::Display for WorkOrder {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(
            f,
            "Work order number: {} \n
            Work order weight: {} \n
            Work load: {:?} \n
            Number of activities: {} \n
            Vendor: {} \n
            AWSC: {} \n
            Revision: {}\n
            ---------------------\n",
            self.order_number,
            self.order_weight,
            self.work_load,
            self.operations.len(),
            self.vendor,
            self.status_codes.awsc,
            self.revision.string
        )
    }
}

impl WorkOrder {
    pub fn to_string_low(&self) -> String {
        let mut message = String::new();

        
        writeln!(
            message,
            "    Work order: {}    |{:>11}|{:<}|{:<}|{:>8}|{:?}|{:?}|{:<3}|{:?}|",
            self.get_work_order_number(),
            self.get_order_dates()
                .earliest_allowed_start_period
                .get_period_string(),
            if self.get_status_codes().awsc {
                "AWSC"
            } else {
                "----"
            },
            if self.get_status_codes().sece {
                "SECE"
            } else {
                "----"
            },
            self.get_revision().string,
            self.get_order_type().get_type_string(),
            self.get_priority().get_priority_string(),
            if self.is_vendor() { "VEN" } else { "---" },
            self.get_status_codes().material_status,
        )
        .unwrap();

        message
    }

    pub fn to_string_medium(&self) -> String {
        let max_key_length = [
            "Work order number",
            "Work order weight",
            "Work load",
            "Number of activities",
            "Vendor",
            "AWSC",
            "Revision",
        ]
        .iter()
        .map(|s| s.len())
        .max()
        .unwrap_or(0)
            + 3;

        let format_line = |key: &str, value: &str| {
            format!("{:<width$}: {} \n", key, value, width = max_key_length)
        };

        let ven = if self.is_vendor() { "VEN" } else { "---" };
        let awsc = if self.status_codes.awsc {
            "AWSC".to_string()
        } else {
            "----".to_string()
        };

        let mut operations_string = String::new();
        for operation in self.operations.values() {
            operations_string.push_str(format!("    {}", operation).as_str());
        }

        format!(
            "{}{}{}{}{}{}{}-----------------",
            format_line("Work order number", &self.order_number.to_string()),
            format_line("Work order weight", &self.order_weight.to_string()),
            format_line("Work load", &format!("{:?}", self.work_load)),
            format_line("Number of activities", &self.operations.len().to_string()),
            format_line("Vendor", ven),
            format_line("AWSC", awsc.as_str()),
            format_line("Revision", &self.revision.string)
        )
    }

    pub fn to_string_high(&self) -> String {
        let max_key_length = [
            "Work order number",
            "Work order weight",
            "Work load",
            "Number of activities",
            "Vendor",
            "AWSC",
            "Revision",
        ]
        .iter()
        .map(|s| s.len())
        .max()
        .unwrap_or(0)
            + 3;

        let format_line = |key: &str, value: &str| {
            format!("{:<width$}: {} \n", key, value, width = max_key_length)
        };

        let ven = if self.is_vendor() { "VEN" } else { "---" };
        let awsc = if self.status_codes.awsc {
            "AWSC".to_string()
        } else {
            "----".to_string()
        };

        let mut operations_string = String::new();
        for operation in self.operations.values() {
            operations_string.push_str(format!("    {}", operation).as_str());
        }

        format!(
            "{}{}{}{}{}{}{}{}---------------",
            format_line("Work order number", &self.order_number.to_string()),
            format_line("Work order weight", &self.order_weight.to_string()),
            format_line("Work load", &format!("{:?}", self.work_load)),
            format_line("Number of activities", &self.operations.len().to_string()),
            operations_string,
            format_line("Vendor", ven),
            format_line("AWSC", awsc.as_str()),
            format_line("Revision", &self.revision.string)
        )
    }
}
