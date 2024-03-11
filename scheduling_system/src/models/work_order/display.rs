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
    pub fn to_string_normal(&self) -> String {
        let mut message = String::new();
        writeln!(
            message,
            "                          |EARL-PERIOD|AWSC|SECE|REVISION|TYPE|PRIO|VEN*| MAT|",
        )
        .unwrap();
        writeln!(
            message,
            "Work order: {}    |{:>11}|{:<}|{:<}|{:>8}|{:>4}|{:>4}|{:>4}|{:?}|",
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
            if self.is_vendor() { "VEN" } else { "----" },
            self.get_status_codes().material_status,
        )
        .unwrap();

        message
    }

    pub fn to_string_verbose(&self) -> String {
        let mut verbose_message = self.to_string_normal();
        writeln!(verbose_message).unwrap();
        writeln!(
            verbose_message,
            "                          |Work Center|Work Remaining|Duration|Number|",
        )
        .unwrap();

        let mut keys: Vec<_> = self.operations.keys().collect();
        keys.sort();
        for key in keys {
            if let Some(operation) = self.operations.get(key) {
                write!(verbose_message, "{}", operation).unwrap();
            }
        }
        verbose_message
    }
}
