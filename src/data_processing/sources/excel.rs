use calamine::{open_workbook, Xlsx, Reader, DataType};
use std::collections::HashMap;
use std::path::Path;


use crate::models::scheduling_environment::{SchedulingEnvironment, WorkOrders};
use crate::models::work_order::WorkOrder;
use crate::models::worker_environment::WorkerEnvironment;


/// This function will load data from excel. It is crucial that the approach is modular and scalable
/// so that it will always be possible to add new data sources and data transformers in the future.
/// 
pub fn load_data_file(file_path: &Path) -> Result<SchedulingEnvironment, calamine::Error> {
    let mut workbook: Xlsx<_> = open_workbook(file_path)?;
    println!("Successfully loaded file.");

    let sheet: &calamine::Range<DataType> = &workbook.worksheet_range_at(0)
        .ok_or(calamine::Error::Msg("Cannot find work order sheet"))?.expect("Could not load work order sheet.");

    let mut work_orders: WorkOrders = WorkOrders::new();
    let mut worker_environment: WorkerEnvironment = WorkerEnvironment::new();

    populate_work_orders(&mut work_orders, sheet);

    let scheduling_environment = SchedulingEnvironment::new(work_orders, worker_environment);

    Ok(scheduling_environment)
}

fn populate_work_orders<'a>(work_orders: &'a mut WorkOrders, sheet: &'a calamine::Range<DataType>) -> Result<&'a mut WorkOrders, calamine::Error> {

    let headers: Vec<String> = sheet.rows()
    .next()
    .ok_or(calamine::Error::Msg("Sheet is empty"))?
    .iter()
    .filter_map(|cell| {
        if let DataType::String(s) = cell {
            Some(s.clone())
        } else {
            None
        }
    }).collect();

    let header_to_index: HashMap<String, usize> = headers.iter()
        .enumerate()
        .map(|(index, header)| (header.clone(), index))
        .collect();

    for row in sheet.rows().skip(1) {
        let mut work_order_number: u32 = 0;
        if let Some(&index) = header_to_index.get("Order") {
            if index < row.len() {
                let value = &row[index];
                
                match value {
                    DataType::String(s) => {
                        match s.parse::<u32>() {
                            Ok(n) => work_order_number = n,
                            Err(e) => {println!("Could not parse work order number as string: {}", e)}
                        }
                    },
                    DataType::Int(s) => work_order_number = *s as u32,
                    _ => { todo!("Handle other cases of DataType"); }
                }
            }
        }
        if work_orders.new_work_order(work_order_number) {
            create_new_work_order(work_orders, row);
        } else {
            create_new_operation(work_orders, row);
        }

    }
    Ok(work_orders)
}


fn create_new_work_order(work_orders: &mut WorkOrders, row: &[DataType]) {

}

fn create_new_operation(work_orders: &mut WorkOrders, row: &[DataType]) {

}