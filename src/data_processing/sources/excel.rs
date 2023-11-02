use actix_web_actors::ws::start;
use calamine::{open_workbook, Xlsx, Reader, DataType, Error};
use regex::Regex;
use core::fmt;
use std::collections::HashMap;
use std::path::Path;

use crate::models::period::Period;

use chrono::{DateTime, Utc, NaiveDate, Duration, TimeZone, naive, NaiveTime, Datelike, Weekday, Timelike};
use crate::models::scheduling_environment::{SchedulingEnvironment, WorkOrders};
use crate::models::work_order::WorkOrder;
use crate::models::work_order::revision::Revision;
use crate::models::work_order::unloading_point::UnloadingPoint;
use crate::models::work_order::functional_location::FunctionalLocation;
use crate::models::work_order::order_text::OrderText;
use crate::models::work_order::status_codes::{StatusCodes, MaterialStatus};
use crate::models::work_order::order_dates::OrderDates;
use crate::models::work_order::order_type::{WDFPriority, WGNPriority, WPMPriority};
use crate::models::work_order::priority::Priority;
use crate::models::work_order::order_type::WorkOrderType;
use crate::models::worker_environment::WorkerEnvironment;
// use crate::models::work_order::optimized_work_order::OptimizedWorkOrder;

extern crate regex;

use crate::models::work_order::operation::Operation;

#[derive(Debug)]
struct ExcelLoadError(String);

impl fmt::Display for ExcelLoadError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "ExcelLoadError: {}", self.0)
    }
}

/// This function will load data from excel. It is crucial that the approach is modular and scalable
/// so that it will always be possible to add new data sources and data transformers in the future.
/// 
pub fn load_data_file(file_path: &Path, number_of_periods: u32) -> Result<SchedulingEnvironment, calamine::Error> {
    let mut workbook: Xlsx<_> = open_workbook(file_path)?;
    println!("Successfully loaded file.");

    let sheet: &calamine::Range<DataType> = &workbook.worksheet_range_at(0)
        .ok_or(calamine::Error::Msg("Cannot find work order sheet"))?.expect("Could not load work order sheet.");

    let mut work_orders: WorkOrders = WorkOrders::new();
    let worker_environment: WorkerEnvironment = WorkerEnvironment::new();

    populate_work_orders(&mut work_orders, sheet).expect("could not populate the work orders");

    let periods: Vec<Period> = create_periods(number_of_periods).expect(&format!("Could not create periods in {} at line {}", file!(), line!()));


    let scheduling_environment = SchedulingEnvironment::new(work_orders, worker_environment, periods);

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
                    DataType::Float(s) => work_order_number = *s as u32,
                    
                    _ => { todo!("Handle other cases of DataType"); }
                }
            }
        }
        // println!("new work order key: {}", work_orders.new_work_order(work_order_number));
        if work_orders.new_work_order(work_order_number) {
            work_orders.insert(create_new_work_order(row, &header_to_index)
                .expect("Could not insert new work order"));
        } 
        
        let operation: Operation = create_new_operation(row, &header_to_index)
            .expect("Could not create a new operation");

        work_orders.inner.get_mut(&work_order_number).expect("Work order not yet created").insert_operation(operation);
        
    }
    Ok(work_orders)
}

/// The fact that I want to extend this means that we should initialize the work order with a default value.
/// This means that the WorkOrder type should receive a new method, that will create a new
/// instance that can then be used to populate the work_orders HashMap.
/// 
/// The operations field is a little more complex as we could have multiple different rows that 
/// write to the same work order. This means that we need to check if the work order already exists
/// 
/// The problem is to find the right approach that makes the function work for both work
/// 
/// Maybe we should just initialize the operations as empty here and then simply always run the 
/// operation reading on each row! Yes that is the approach that I want to take.
fn create_new_work_order(row: &[DataType], header_to_index: &HashMap<String, usize>) -> Result<WorkOrder, Error> {
    
    let priority = match row.get(*header_to_index.get("Priority").ok_or("Priority header not found")?).cloned() {
        Some(DataType::Int(n)) => Priority::IntValue(n as i32),
        Some(DataType::String(s)) => {

            match s.parse::<i32>() {
                Ok(num) => Priority::IntValue(num), // If successful, use the integer value
                Err(_) => Priority::StringValue(s), // If not, fall back to using the string
            }

        }
        Some(DataType::Float(n)) => Priority::IntValue(n as i32),
        _ => Priority::StringValue(String::new())
    };

    println!("priority: {:?}", priority);
    Ok(WorkOrder {
        order_number: match row.get(*header_to_index.get("Order").ok_or("Order header not found")?).cloned() {
            Some(DataType::Int(n)) => n as u32,
            Some(DataType::Float(n)) => n as u32,
            Some(DataType::String(s)) => s.parse::<u32>().unwrap_or(0),
            _ => 0
        },
        // optimized_work_order: OptimizedWorkOrder::empty(),
        fixed: false,
        order_weight: 0, // TODO: Implement calculate_weight method.
        priority: priority.clone(),
        order_work: 0.0,
        operations: HashMap::<u32, Operation>::new(),
        work_load: HashMap::<String, f64>::new(),
        start_start: Vec::<bool>::new(), 
        finish_start: Vec::<bool>::new(),
        postpone: Vec::<DateTime<Utc>>::new(),
        order_type: match row.get(*header_to_index.get("Order_Type").ok_or("Order_Type header not found")?).cloned() {
            Some(DataType::String(work_order_type)) => {
                match work_order_type.as_str() {
                    "WDF" => match &priority {
                        Priority::IntValue(value) => {
                            dbg!(value);
                            match value {
                                1 => Ok(WorkOrderType::WDF(WDFPriority::One)),
                                2 => Ok(WorkOrderType::WDF(WDFPriority::Two)),
                                3 => Ok(WorkOrderType::WDF(WDFPriority::Three)),
                                4 => Ok(WorkOrderType::WDF(WDFPriority::Four)),
                                _ => Ok(WorkOrderType::Other),
                            }
                        },
                        _ => Err(ExcelLoadError("Could not parse WDF priority as int".into()))
                    },
                    "WGN" => match &priority {
                        Priority::IntValue(value) => {
                            match value {
                                1 => Ok(WorkOrderType::WGN(WGNPriority::One)),
                                2 => Ok(WorkOrderType::WGN(WGNPriority::Two)),
                                3 => Ok(WorkOrderType::WGN(WGNPriority::Three)),
                                4 => Ok(WorkOrderType::WGN(WGNPriority::Four)),
                                _ => Ok(WorkOrderType::Other),
                            }
                        },
                        _ => Err(ExcelLoadError("Could not parse WGN priority as int".into()))
                    },
                    "WPM" => match &priority {
                        Priority::StringValue(value) => {
                            match value.as_str() {
                                "A" => Ok(WorkOrderType::WPM(WPMPriority::A)),
                                "B" => Ok(WorkOrderType::WPM(WPMPriority::B)),
                                "C" => Ok(WorkOrderType::WPM(WPMPriority::C)),
                                "D" => Ok(WorkOrderType::WPM(WPMPriority::D)),
                                _ => Ok(WorkOrderType::Other),
                            }
                        },
                        _ => Err(ExcelLoadError("Could not parse WPM priority as int".into()))
                    
                    },
                    _ => Ok(WorkOrderType::Other),
                }
            },
            None => {
                println!("Order_Type is None");
                Ok(WorkOrderType::Other)
            }
            _ => return Err(Error::Msg("Could not parse revision as string"))

        }.expect("Could not parse order type"),
        status_codes: extract_status_codes(row, &header_to_index).expect("Failed to extract StatusCodes"), 
        order_dates: extract_order_dates(row, &header_to_index).expect("Failed to extract OrderDates"), 
        revision: extract_revision(row, &header_to_index).expect("Failed to extract Revision"), 
        unloading_point: extract_unloading_point(row, &header_to_index).expect("Failed to extract UnloadingPoint"),
        functional_location: extract_functional_location(row, &header_to_index).expect("Failed to extract FunctionalLocation"),
        order_text: extract_order_text(row, &header_to_index).expect("Failed to extract OrderText"),
        vendor: false 
    })
}

fn create_new_operation(row: &[DataType], header_to_index: &HashMap<String, usize>) -> Result<Operation, Error> {
    
    let default_future_date = Utc.with_ymd_and_hms(2026, 1, 1, 7, 0, 0).unwrap();

    let work_possible_headers = ["Work_Remaining", "Work_Planned"];
    let earliest_start_time_headers = ["Latest_Start_Time", "Earliest_Start_Time"];
    let earliest_finish_date_headers = ["Earliest_Finish_Date", "Earliest_End_Date"];
    let earliest_finish_time_headers = ["Earliest_Finish_Time", "Latest_Finish_Time"];

    let work_remaining_data = get_data_from_headers(&row, &header_to_index, &work_possible_headers);
    let earliest_start_time_data = get_data_from_headers(&row, &header_to_index, &earliest_start_time_headers);
    let earliest_finish_date_data = get_data_from_headers(&row, &header_to_index, &earliest_finish_date_headers);
    let earliest_finish_time_data = get_data_from_headers(&row, &header_to_index, &earliest_finish_time_headers);

    Ok(Operation {
        activity: match row.get(*header_to_index.get("Activity").ok_or("Activity header not found")?).cloned() {
            Some(DataType::Int(n)) => n as u32,
            Some(DataType::Float(n)) => n as u32,
            Some(DataType::String(s)) => s.parse::<u32>().unwrap_or(0),
            _ => 0
        },
        number: match row.get(*header_to_index.get("Number").ok_or("Number header not found")?).cloned() {
            Some(DataType::Int(n)) => n as u32,
            Some(DataType::Float(n)) => n as u32,
            Some(DataType::String(s)) => s.parse::<u32>().unwrap_or(0),
            _ => 0
        },
        work_center: match row.get(*header_to_index.get("Work_Center").ok_or("Work Center header not found")?).cloned() {
            Some(DataType::String(s)) => s,
            _ => return Err(Error::Msg("Could not parse work center as string")),
        },
        preparation_time: 0.0,
        work_remaining: match work_remaining_data.cloned() {
            Some(DataType::Int(n)) => n as f64,
            Some(DataType::Float(n)) => n as f64,
            Some(DataType::String(s)) => s.parse::<f64>().unwrap_or(0.0),
            _ => 0.0
        },
        work_performed: match row.get(*header_to_index.get("Work_Actual").ok_or("Work Actual header not found")?).cloned() {
            Some(DataType::Int(n)) => n as f64,
            Some(DataType::Float(n)) => n as f64,
            Some(DataType::String(s)) => s.parse::<f64>().unwrap_or(0.0),
            _ => 0.0
        },
        work_adjusted: 0.0,
        operating_time: 0.0,
        duration: match header_to_index.get("Duration") {
            Some(index) => match row.get(*index).cloned() {
                Some(DataType::Int(n)) => n as u32,
                Some(DataType::Float(n)) => n as u32,
                Some(DataType::String(s)) => s.parse::<u32>().expect("Duration is not a valid number"),
                _ => 0
            },
            None => {
                // dbg!("Duration is None");
                0
            }
        },
        possible_start: default_future_date,
        target_finish: default_future_date,
        earliest_start_datetime: 
            {
                let date = match row.get(*header_to_index.get("Earliest_Start_Date").ok_or("Earliest Start Date header not found")?).cloned() {
                    Some(DataType::String(s)) => {
                        parse_date(&s)
                    }
                    _ => return Err(Error::Msg("Could not parse Earliest_Start_Date as string"))

                };

                let time = match earliest_start_time_data.cloned() {
                    Some(DataType::String(s)) => {
                        match NaiveTime::parse_from_str(&s, "%H:%M:%s") {
                            Ok(naive_date) => naive_date,
                            Err(_) => {
                                println!("Could not parse earliest_start_time_data from string: {}", s);
                                NaiveTime::from_hms_opt(7, 0, 0).unwrap()
                            }
                        }
                    }
                    _ => return Err(Error::Msg("Could not parse earliest_start_time_data as string"))

                };

                Utc.from_utc_datetime(&naive::NaiveDateTime::new(date, time))
            },
        earliest_finish_datetime: 
            {
                let date = match earliest_finish_date_data.cloned() {
                    Some(DataType::String(s)) => {
                        parse_date(&s)
                    }
                    _ => return Err(Error::Msg("Could not earliest_finish_date_data revision as string"))
                };

                let time = match earliest_finish_time_data.cloned() {
                    Some(DataType::String(s)) => {
                        match NaiveTime::parse_from_str(&s, "%H:%M:%s") {
                            Ok(naive_date) => naive_date,
                            Err(_) => {
                                dbg!();
                                println!("Could not parse earliest_finish_time_data from string: {}", s);
                                NaiveTime::from_hms_opt(7, 0, 0).unwrap()
                            }
                        }
                    }
                    _ => return Err(Error::Msg("Could not parse earliest_finish_time_data"))
                };
                Utc.from_utc_datetime(&naive::NaiveDateTime::new(date, time))
            },
    })
}

/// This function will extract the status codes from the row and return them as a StatusCodes struct.
fn extract_status_codes(row: &[DataType], header_to_index: &HashMap<String, usize>) -> Result<StatusCodes, Error> {

    let system_status = match row.get(*header_to_index.get("System_Status").ok_or("System_Status header not found")?).cloned() {
        Some(DataType::String(s)) => s,
        _ => return Err(Error::Msg("Could not parse system status as string")),
    };

    let user_status = match row.get(*header_to_index.get("User_Status").ok_or("User_Status header not found")?).cloned() {
        Some(DataType::String(s)) => s,
        _ => return Err(Error::Msg("Could not parse user status as string")),
    };

    let opr_user_status = match row.get(*header_to_index.get("Opr_User_Status").ok_or("Opr_User_Status header not found")?).cloned() {
        Some(DataType::String(s)) => s,
        _ => return Err(Error::Msg("Could not parse opr user status as string")),
    };

    let opr_system_status = match row.get(*header_to_index.get("Opr_System_Status").ok_or("Opr_System_Status header not found")?).cloned() {
        Some(DataType::String(s)) => s,
        _ => return Err(Error::Msg("Could not parse opr system status as string")),
    };

    // concatenate the status codes into a single string
    let status_codes_string = format!("{} {} {} {}", system_status, user_status, opr_user_status, opr_system_status);

    let pcnf_pattern = regex::Regex::new(r"PCNF").unwrap();
    let awsc_pattern = regex::Regex::new(r"AWSC").unwrap();
    let well_pattern = regex::Regex::new(r"WELL").unwrap();
    let sch_pattern = regex::Regex::new(r"SCH").unwrap();
    let sece_pattern = regex::Regex::new(r"SECE").unwrap();

    let material_status: MaterialStatus = MaterialStatus::from_status_code_string(&status_codes_string);

    Ok(StatusCodes {
        material_status,
        pcnf: pcnf_pattern.is_match(&status_codes_string),
        awsc: awsc_pattern.is_match(&status_codes_string),
        well: well_pattern.is_match(&status_codes_string),
        sch: sch_pattern.is_match(&status_codes_string),
        sece: sece_pattern.is_match(&status_codes_string),
        unloading_point: false, // Assuming default value; modify as needed
    })
}

fn extract_order_dates(row: &[DataType], header_to_index: &HashMap<String, usize>) -> Result<OrderDates, Error> {

    let earliest_allowed_start_date = match row.get(*header_to_index.get("Earliest_Start_Date").ok_or("Earliest_Start_Date header not found")?).cloned() {
        Some(DataType::String(s)) => {
            parse_date(&s)
        }
        _ => return Err(Error::Msg("Could not parse revision as string"))

    };
    
    let latest_allowed_finish_date = match row.get(*header_to_index.get("Latest_Allowed_Finish_Date").ok_or("Latest_Allowed_Finish_Date header not found")?).cloned() {
        Some(DataType::String(s)) => {
            parse_date(&s)
        }
        _ => return Err(Error::Msg("Could not parse revision as string"))

    };

    let basic_start_date = match row.get(*header_to_index.get("Basic_Start_Date").ok_or("Basic_Start_Date header not found")?).cloned() {
        Some(DataType::String(s)) => {
            parse_date(&s)
        }
        _ => return Err(Error::Msg("Could not parse revision as string"))

    };

    let basic_finish_date = match row.get(*header_to_index.get("Basic_Finish_Date").ok_or("Basic_Finish_Date header not found")?).cloned() {
        Some(DataType::String(s)) => {
            parse_date(&s)
        }
        _ => return Err(Error::Msg("Could not parse revision as string"))
    };

    Ok(OrderDates {
        earliest_allowed_start_date: DateTime::<Utc>::from_naive_utc_and_offset(earliest_allowed_start_date.and_hms_opt(7, 0, 0).unwrap(), Utc),
        latest_allowed_finish_date: DateTime::<Utc>::from_naive_utc_and_offset(latest_allowed_finish_date.and_hms_opt(7, 0, 0).unwrap(), Utc),
        basic_start_date: DateTime::<Utc>::from_naive_utc_and_offset(basic_start_date.and_hms_opt(7, 0, 0).unwrap(), Utc),
        basic_finish_date: DateTime::<Utc>::from_naive_utc_and_offset(basic_finish_date.and_hms_opt(7, 0, 0).unwrap(), Utc),
        duration: basic_finish_date.signed_duration_since(basic_start_date),
        basic_start_scheduled: None,
        basic_finish_scheduled: None,
        material_expected_date: None,
    })
}

fn extract_revision(row: &[DataType], header_to_index: &HashMap<String, usize>) -> Result<Revision, Error> {

    let string = match row.get(*header_to_index.get("Revision").ok_or("Revision header not found")?).cloned() {
        Some(DataType::String(s)) => s,
        _ => return Err(Error::Msg("Could not parse revision as string")),
    };
    
    let shutdown_pattern = r"NOSD|NE";

    let shutdown = Regex::new(shutdown_pattern).unwrap();
    let shutdown = !shutdown.is_match(&string);

    Ok(Revision {
        string,
        shutdown,
    })
}

fn extract_unloading_point(row: &[DataType], header_to_index: &HashMap<String, usize>) -> Result<UnloadingPoint, Error> {
    
    let string = match row.get(*header_to_index.get("Unloading_Point").ok_or("Unloading_Point header not found")?).cloned() {
        Some(DataType::String(s)) => s,
        Some(DataType::Int(n)) => n.to_string(),
        Some(DataType::Float(n)) => n.to_string(),
        Some(DataType::Bool(b)) => b.to_string(),
        Some(DataType::Error(e)) => e.to_string(),
        Some(DataType::Empty) => String::from("Empty"),
        None => String::from("None"),
        _ => return Err(Error::Msg("Could not parse unloading point as string")),
    };
    
    let (start_week, end_week, present) = _extract_weeks(&string);
    let start_date = _week_to_date(start_week, true);
    let end_date = _week_to_date(end_week, false);

    if present {
        Ok(UnloadingPoint {
            string: string,
            present,
            period: Some(Period::new(0, start_date, end_date))
        })     
    } else {
        Ok(UnloadingPoint {
            string: string,
            present,
            period: None
        })  
    }
}

fn _week_to_date(week_number: u32, start_of_week: bool) -> DateTime<Utc> {
    let today_date = chrono::Local::now().naive_local();
    let current_year = today_date.year();
    let current_week = today_date.iso_week().week();

    // Determine the target year based on the week number and current date
    let target_year = if week_number >= current_week {
        current_year
    } else {
        current_year + 1
    };

    // Compute the date corresponding to the start of the target week (Monday)
    let new_year_date = NaiveDate::from_ymd_opt(target_year, 1, 1);  // January 1st of the target year
    let first_week_day = new_year_date.unwrap().weekday();
    let offset: Duration = if first_week_day.num_days_from_sunday() <= Weekday::Mon.num_days_from_sunday() {
        Duration::days((Weekday::Mon.num_days_from_sunday() - first_week_day.num_days_from_sunday()) as i64)
    } else {
        Duration::days((7 - (first_week_day.num_days_from_sunday() - Weekday::Mon.num_days_from_sunday())) as i64)
    };

    let start_date = new_year_date.unwrap() + offset + Duration::weeks(week_number as i64 - 1);

    let time = NaiveTime::from_hms_opt(0, 0, 0);

    let naive_datetime = start_date.and_time(time.unwrap());

    let start_datetime = Utc.from_utc_datetime(&naive_datetime);
    if start_of_week {
        start_datetime
    } else {
        start_datetime + Duration::days(6)
    }
}

fn _extract_weeks(input_string: &str) -> (u32, u32, bool) {
    let re = regex::Regex::new(r"W(\d+)-(\d+)").unwrap();
    let captures = re.captures(input_string);

    if let Some(cap) = captures {
        let start_week = cap[1].parse::<u32>().unwrap_or(0);
        let end_week = cap[2].parse::<u32>().unwrap_or(0);
        (start_week, end_week, true)
    } else {
        (0, 0, false)
    }
}

fn extract_functional_location(row: &[DataType], header_to_index: &HashMap<String, usize>) -> Result<FunctionalLocation, Error> {
    let string = row.get(*header_to_index.get("Functional_Location").ok_or("Functional_Location header not found")?).cloned();
    
    Ok(FunctionalLocation {
        string: string.unwrap().to_string(),
    })
}

fn extract_order_text(row: &[DataType], header_to_index: &HashMap<String, usize>) -> Result<OrderText, Error> {
    let notes_1 = match row.get(header_to_index.get("Notes_1").unwrap_or(&usize::MAX).clone()) {
        Some(DataType::String(s)) => s.to_string(),
        None => "Notes 1 is not part of the inputed data".to_string(),
        _ => return Err(Error::Msg("Could not parse notes_1 as string")),
    };
    
    let notes_2 = match row.get(header_to_index.get("Notes_2").unwrap_or(&usize::MAX).clone()) {
        Some(DataType::String(s)) => s.parse::<u32>().unwrap_or(8) ,
        Some(DataType::Int(n)) => *n as u32,
        None => 5,
        _ => return Err(Error::Msg("Could not parse notes_2 as an integer")),
    };
        
    let object_description = match row.get(*header_to_index.get("Description_2").ok_or("object_description header not found")?).cloned() {
        Some(DataType::String(s)) => s,
        _ => return Err(Error::Msg("Could not parse object_description as string")),
    };
    
    let order_description = match row.get(*header_to_index.get("Description_1").ok_or("order_description header not found")?).cloned() {
        Some(DataType::String(s)) => s,
        _ => return Err(Error::Msg("Could not parse order_description as string")),
    };
    
    let operation_description = match row.get(*header_to_index.get("Short_Text").ok_or("operation_description header not found")?).cloned() {
        Some(DataType::String(s)) => s,
        _ => return Err(Error::Msg("Could not parse operation_description as string")),
    };

    let order_system_status = match row.get(*header_to_index.get("System_Status").ok_or("order_system_status header not found")?).cloned() {
        Some(DataType::String(s)) => s,
        _ => return Err(Error::Msg("Could not parse order_system_status as string")),
    };

    let order_user_status = match row.get(*header_to_index.get("User_Status").ok_or("order_user_status header not found")?).cloned() {
        Some(DataType::String(s)) => s,
        _ => return Err(Error::Msg("Could not parse order_user_status as string")),
    };

    Ok(OrderText {
        order_system_status,
        order_user_status,
        order_description,
        operation_description,
        object_description,
        notes_1,
        notes_2,
    })
}

fn parse_date(s: &str) -> NaiveDate {
    let formats = ["%Y%m%d", "%Y-%m-%d", "%d/%m/%Y", "%d-%m-%Y", "%d.%m.%Y"];

    for format in &formats {
        match NaiveDate::parse_from_str(s, format) {
            Ok(naive_date) => return naive_date,
            Err(_) => continue,
        }
    }

    println!("Could not parse date from string: {}", s);
    NaiveDate::from_ymd_opt(2026, 1, 1).unwrap()
}

fn get_data_from_headers<'a>(
    row: &'a [DataType], 
    header_to_index: &HashMap<String, usize>, 
    headers: &[&str]
) -> Option<&'a DataType> {
    for &header in headers {
        if let Some(&index) = header_to_index.get(header) {
            if let Some(data) = row.get(index) {
                return Some(data);
            }
        }
    }
    None
}


#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_date() {
        let date = parse_date("2021-01-01");
        assert_eq!(date, NaiveDate::from_ymd_opt(2021, 1, 1).unwrap());
    }
    // fn parse_date(s: &str) -> NaiveDate {
    //     let formats = ["%Y%m%d", "%Y-%m-%d", "%d/%m/%Y", "%d-%m-%Y", "%d.%m.%Y"];
    
    //     for format in &formats {
    //         match NaiveDate::parse_from_str(s, format) {
    //             Ok(naive_date) => return naive_date,
    //             Err(_) => continue,
    //         }
    //     }
    
    //     println!("Could not parse date from string: {}", s);
    //     NaiveDate::from_ymd_opt(2026, 1, 1).unwrap()
    // }


}


fn create_periods(number_of_periods: u32) -> Result<Vec<Period>, Error> {
    let mut periods: Vec<Period> = Vec::<Period>::new();
    let mut start_date = Utc::now();

    // Get the ISO week number
    let week_number = start_date.iso_week().week();
    // Determine target week number: If current is even, target is the previous odd
    let target_week = if week_number % 2 == 0 {
        week_number  - 1
    } else {
        week_number
    };

    // Compute the offset in days to reach Monday of the target week
    let days_to_offset = (start_date.weekday().num_days_from_monday() as i64) + 
                         (7 * (week_number - target_week) as i64);

    // dbg!(days_to_offset);

    // dbg!(start_date);

    start_date = start_date - Duration::days(days_to_offset);

    start_date = start_date.with_hour(0)
        .and_then(|d| d.with_minute(0))
        .and_then(|d| d.with_second(0))
        .and_then(|d| d.with_nanosecond(0))
        .unwrap();

    // dbg!(start_date);

    let mut end_date = start_date + Duration::weeks(2);
    // dbg!(end_date);
    
    end_date = end_date - Duration::days(1);
    
    end_date = end_date.with_hour(23)
    .and_then(|d| d.with_minute(59))
    .and_then(|d| d.with_second(59))
    .and_then(|d| d.with_nanosecond(0))
    .unwrap();

    // dbg!(end_date);

    for i in 0..number_of_periods {
        dbg!(start_date);
        dbg!(end_date);
        periods.push(Period::new(i, start_date, end_date));
        start_date = start_date + Duration::weeks(2);
        end_date = end_date + Duration::weeks(2);

    }
    Ok(periods)
}
