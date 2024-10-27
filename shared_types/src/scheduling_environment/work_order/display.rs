use crate::scheduling_environment::work_order::WorkOrder;
use std::fmt;

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
            self.work_order_analytic.user_status_codes.awsc,
            self.work_order_info.revision.string
        )
    }
}
