use std::fmt;

use crate::models::work_order::WorkOrder;

impl fmt::Display for WorkOrder {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, 
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