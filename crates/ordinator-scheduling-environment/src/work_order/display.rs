use super::WorkOrder;
use std::fmt;

impl fmt::Display for WorkOrder {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(
            f,
            "Work order number: {:?} \n
            Number of activities: {} \n
            Vendor: {} \n
            AWSC: {} \n
            Revision: {}\n
            ---------------------\n",
            self.work_order_number,
            self.operations.0.len(),
            self.vendor(),
            self.work_order_analytic.user_status_codes.awsc,
            self.work_order_info.revision.to_string()
        )
    }
}
