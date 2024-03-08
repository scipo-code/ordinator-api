pub mod time_environment;
pub mod work_order;
pub mod worker_environment;

use std::collections::HashMap;
use std::fmt;

use crate::models::time_environment::period::Period;
use crate::models::work_order::WorkOrder;
use crate::models::worker_environment::WorkerEnvironment;

pub struct SchedulingEnvironment {
    work_orders: WorkOrders,
    worker_environment: WorkerEnvironment,
    periods: Vec<Period>,
    // material
}

impl SchedulingEnvironment {
    pub fn new(
        work_orders: WorkOrders,
        worker_environment: WorkerEnvironment,
        periods: Vec<Period>,
    ) -> Self {
        SchedulingEnvironment {
            work_orders,
            worker_environment,
            periods,
        }
    }


    
    pub fn clone_periods(&self) -> Vec<Period> {
        self.periods.clone()
    }

    pub fn clone_work_orders(&self) -> WorkOrders {
        self.work_orders.clone()
    }

    pub fn initialize_work_orders(&mut self, periods: &[Period]) {
        for (_, work_order) in self.work_orders.inner.iter_mut() {
            work_order.initialize(periods);
        }
    }

    pub fn get_mut_periods(&mut self) -> &mut Vec<Period> {
        &mut self.periods
    }

    pub fn get_periods(&self) -> &Vec<Period> {
        &self.periods
    }

    pub fn get_worker_environment(&self) -> &WorkerEnvironment {
        &self.worker_environment
    }

    pub fn initialize_worker_environment(&mut self) {
        self.worker_environment.initialize();
    }
}

impl Default for SchedulingEnvironment {
    fn default() -> Self {
        SchedulingEnvironment {
            work_orders: WorkOrders::new(),
            worker_environment: WorkerEnvironment::new(),
            periods: vec![Period::new(
                0,
                chrono::Utc::now(),
                chrono::Utc::now() + chrono::Duration::days(14) - chrono::Duration::seconds(1),
            )]
                }
    }
}

#[derive(Clone, Debug)]
pub struct WorkOrders {
    pub inner: HashMap<u32, WorkOrder>,
}

impl WorkOrders {
    pub fn new() -> Self {
        WorkOrders {
            inner: HashMap::<u32, WorkOrder>::new(),
        }
    }

    pub fn insert(&mut self, work_order: WorkOrder) {
        self.inner
            .insert(work_order.get_work_order_number(), work_order);
    }

    pub fn new_work_order(&self, order_number: u32) -> bool {
        !self.inner.contains_key(&order_number)
    }
}

impl fmt::Display for SchedulingEnvironment {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {                 
        write!(f, "The Scheduling Environment is currently comprised of \n  number of work orders: {},\n  number of worker entries: {},\n  number of periods: {}", 
        self.work_orders.inner.len(), 
        match self.get_worker_environment().get_crew().as_ref() {
            Some(crew) => crew.get_workers().len(),
            None => 0,
    }, self.periods.len())?;
        Ok(())
    }
}

impl fmt::Display for WorkOrders {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "The Work Orders are currently comprised of \n  work_orders: {}",
            self.inner.len()
        )?;
        for (_, work_order) in self.inner.iter() {
            write!(f, "{}", work_order.to_string_low())?;
        }
        Ok(())
    }
}

// impl WorkOrders {

//     pub fn format_selected_work_orders(
//         &self,
//         work_orders_number: Vec<u32>,
//         period: Option<String>,
//     ) -> String {
//         let mut message = String::new();

//         match period {
//             Some(period) => writeln!(
//                 message,
//                 "Work orders scheduled for period: {} are: ",
//                 period,
//             ),
//             None => writeln!(message, "All work orders"),
//         }
//         .unwrap();

//         writeln!(
//             message,
//             "                      EARL-PERIOD|AWCS|SECE|REVISION|TYPE|PRIO|VEN*| MAT|",
//         )
//         .unwrap();

//         let mut work_orders = self
//             .scheduling_environment
//             .lock()
//             .unwrap()
//             .clone_work_orders();

//         for work_order_number in work_orders_number {
//             writeln!(
//                 message,
//                 "    Work order: {}    |{:>11}|{:<}|{:<}|{:>8}|{:?}|{:?}|{:<3}|{:?}|",
//                 work_order_number,
//                 work_orders
//                     .inner
//                     .get_mut(&work_order_number)
//                     .unwrap()
//                     .get_order_dates()
//                     .earliest_allowed_start_period
//                     .get_period_string(),
//                 if work_orders
//                     .inner
//                     .get(&work_order_number)
//                     .unwrap()
//                     .get_status_codes()
//                     .awsc
//                 {
//                     "AWSC"
//                 } else {
//                     "----"
//                 },
//                 if work_orders
//                     .inner
//                     .get(&work_order_number)
//                     .unwrap()
//                     .get_status_codes()
//                     .sece
//                 {
//                     "SECE"
//                 } else {
//                     "----"
//                 },
//                 work_orders
//                     .inner
//                     .get(&work_order_number)
//                     .unwrap()
//                     .get_revision()
//                     .string,
//                 work_orders
//                     .inner
//                     .get(&work_order_number)
//                     .unwrap()
//                     .get_order_type()
//                     .get_type_string(),
//                 work_orders
//                     .inner
//                     .get(&work_order_number)
//                     .unwrap()
//                     .get_priority()
//                     .get_priority_string(),
//                 if work_orders
//                     .inner
//                     .get(&work_order_number)
//                     .unwrap()
//                     .is_vendor()
//                 {
//                     "VEN"
//                 } else {
//                     "---"
//                 },
//                 work_orders
//                     .inner
//                     .get(&work_order_number)
//                     .unwrap()
//                     .get_status_codes()
//                     .material_status,
//             )
//             .unwrap();
//         }
//         message
//     }
    
// }


#[cfg(test)]
mod tests {
    use super::*;

    impl SchedulingEnvironment {
        pub fn get_work_orders(&self) -> &WorkOrders {
            &self.work_orders
        }
    }
}
