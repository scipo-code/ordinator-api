use crate::models::work_order;
use crate::models::scheduling_environment;

/// This function will load in data from excel row by row and populate the data structures found in 
/// the models module. 
fn load_row_by_row(row: &[DataType], header_to_index: HashMap<String, usize>) {
    if let Some(&index) = header_to_index.get("Order") {
        if index < row.len() {
            let value = &row[index];
            let mut work_order_number: u32 = 0;
            match value {
                DataType::String(s) => {
                    match s.parse::<u32>() {
                        Ok(n) => work_order_number = n,
                        Err(e) => {println!("Could not parse work order number: {}", e)}
                    }
                }
            }

            if 


        }
    }

}