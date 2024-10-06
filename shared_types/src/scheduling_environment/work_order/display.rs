use crate::scheduling_environment::work_order::WorkOrder;
use std::fmt;
use std::fmt::Write;

use super::operation;

impl fmt::Display for WorkOrder {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(
            f,
            "Work order number: {:?} \n
            Work order weight: {} \n
            Work load: {:?} \n
            Number of activities: {} \n
            Vendor: {} \n
            AWSC: {} \n
            Revision: {}\n
            ---------------------\n",
            self.work_order_number,
            self.work_order_analytic.work_order_weight,
            self.work_order_analytic.work_load,
            self.operations.len(),
            self.work_order_analytic.vendor,
            self.work_order_analytic.status_codes.awsc,
            self.work_order_info.revision.string
        )
    }
}

impl WorkOrder {
    pub fn to_string_normal(&self) -> String {
        let mut message = String::new();
        writeln!(
            message,
            "Work order: {:?}    |{:>11}|{:<}|{:<}|{:<}|{:>8}|{:>4}|{:>4}|{:>4}|{:>11}|{:>5}|",
            self.work_order_number,
            self.order_dates()
                .earliest_allowed_start_period
                .period_string(),
            if self.status_codes().sch {
                " SCH"
            } else {
                "    "
            },
            if self.status_codes().awsc {
                "AWSC"
            } else {
                "    "
            },
            if self.status_codes().sece {
                "SECE"
            } else {
                "    "
            },
            self.work_order_info.revision.string,
            self.work_order_info.work_order_type.get_type_string(),
            self.work_order_info.priority.get_priority_string(),
            if self.is_vendor() { "VEN" } else { "    " },
            self.work_order_analytic.status_codes.material_status,
            self.work_order_info.functional_location.asset,
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
