use csv::{Reader, ReaderBuilder, WriterBuilder};
use serde::Deserialize;
use std::error::Error;
use std::collections::{HashSet, HashMap};
use xlsxwriter::{Workbook, Worksheet, Format};
use std::time::SystemTime;
use std::path::Path;
use std::io::{BufReader, BufWriter, BufRead, Write};
use std::fs::File;

pub fn read_csv_files() {

    let input_iw39_path = Path::new("data/data_model_input/IW39.csv");
    let input_opr_path = Path::new("data/data_model_input/Opr.csv");
    let input_time_postings_path = Path::new("data/data_model_input/Time_Postings.csv");

    // let output_iw39_path = Path::new("data/data_model_input/IW39_fixed.csv");
    // let output_opr_path = Path::new("data/data_model_input/Opr_fixed.csv");
    // let output_time_postings_path = Path::new("data/data_model_input/Time_Postings_fixed.csv");

    // remove_start_end_issues(input_iw39_path, output_iw39_path).unwrap();
    // remove_start_end_issues(input_opr_path, output_opr_path).unwrap();
    // remove_start_end_issues(input_time_postings_path, output_time_postings_path).unwrap();

    let work_order: Vec<Job> = read_csv::<Job>(input_iw39_path).expect("Failed to read CSV file");
    let activities: Vec<Activity> = read_csv::<Activity>(input_opr_path).expect("Failed to read CSV file");
    let time_postings: Vec<TimePosting> = read_csv::<TimePosting>(input_time_postings_path).expect("Failed to read CSV file");

    join_csv_files(work_order, activities, time_postings);
}

fn join_csv_files(jobs: Vec<Job>, activities: Vec<Activity>, time_postings: Vec<TimePosting>) {
    let activities_by_order: HashMap<String, Vec<Activity>> = 
    activities.iter().fold(HashMap::new(), |mut acc, activity| {
        acc.entry(activity.Order.clone().unwrap()).or_insert_with(Vec::new).push(activity.clone());
        acc
    });

    let time_postings_by_activity: HashMap<String, TimePosting> =
    time_postings.into_iter().fold(HashMap::new(), |mut acc, time_posting| {
        // If the activity is already in the map, replace only if the new time_posting date is more recent.
        if let Some(existing) = acc.get(&time_posting.Activity.clone().unwrap()) {
            if existing.Time_Posting_Date < time_posting.Time_Posting_Date {
                acc.insert(time_posting.Activity.clone().unwrap(), time_posting);
            }
        } else {
            acc.insert(time_posting.Activity.clone().unwrap(), time_posting);
        }
        acc
    });


    let workbook = export_to_excel();

    let header_set = get_header_set();

    let mut row = 0;

    if row == 0 {

        let mut i = 0;
        for header in header_set {
            workbook.get_worksheet("scheduler_data").unwrap().unwrap().write_string(row, i, header, None).unwrap();
            i += 1;
        }

    } else {
        for job in &jobs {
            if let Some(related_activities) = activities_by_order.get(&job.Order.clone().unwrap()) {
                for activity in related_activities {
                    if let Some(time_posting) = time_postings_by_activity.get(&activity.Activity.clone().unwrap()) {
                        

                        // Remaining_Duration	Short_Text	Actual_Start_Date	Basic_Start_Date	Description_3	Description_1	Revision	Order	Created_On_Year	Ealiest_Finish_Date	Unloading_Point	Median_Time_Posting_Date	Extraction_Date	Long_Txt_Key	Priority_Original	Latest_Start_Time	Long_Text_Flag	Priority_Days	Actual_Finish_Date	Scheduled_Start_Date	Work_Center_Main	Notification_No	Description_2	User_Status_All	Material	Latest_Start_Date	Activity_Type	Activity	Opr_User_Status	System_Status	Priority_Original_Finish_Days	System_Condition_Desc	Priority	Changed_On_Date	Scheduled_Finish_Date	Time_Posting_Date	Order_Type	Original_Latest_Allowed_Finish_Date	Basic_Finish_Date	System_Condition	Plant_Section	%Maint_Plan_Key	Work_Center	Work_Actual	Distinct_Opr	Work_Forecast	Opr_System_Status	Original_Start_Date	VIS	Created_On	Remaining_Work	Latest_Finish_Date	Actual_Start_Time	Priority_Type	Maintenance_Plant	Actual_State_Date	Priority_Original_Start_Days	Equipment	Work_Planned	Latest_Finish_Time	Earliest_Start_Date	Functional_Location	Leading_Order	Actual_Release_Date	Description_4	Latest_Allowed_Finish_Date	Suporior_Order	User_Status	Actual_Finish_Time	Maintenance_Plan	Location	Number	Maintenance_Item	Personnel_ID	Material_Number	Created_On_Date	Earliest_End_Date	Priority_Desc

                        workbook.get_worksheet("scheduler_data").unwrap().unwrap().write_string(row, 0, &time_posting.Functional_Location.clone().unwrap_or("".to_string()), None).unwrap();






                        row += 1;
                    }
                }
            }
        }
    }
    workbook.close().expect("Failed to close the workbook");
}

fn export_to_excel() -> Workbook  {
    let current_time = SystemTime::now();
    let datetime: chrono::DateTime<chrono::Utc> = current_time.into();
    let formatted_time = datetime.format("%Y%m%d%H%M%S");
    let filename = format!("data/input/JoinedData_{}.xlsx", formatted_time);
    let workbook = Workbook::new(&filename).expect("Failed to create WorkBook");
    workbook.add_worksheet(Some("scheduler_data")).expect("Failed to add worksheet");   
    workbook
}

fn read_csv<T: for<'de> Deserialize<'de>>(path: &Path) -> Result<Vec<T>, Box<dyn Error>> {
    
    dbg!("Type of T: {}", std::any::type_name::<T>());
    
    let mut rdr = ReaderBuilder::new()
        .delimiter(b',')          // Delimiter used in your CSV
        .quote(b'"')              // Quote character used in your CSV
        .double_quote(true)       // Set to true if "" should be parsed as a single "
        .flexible(true)           // Allows for a flexible number of fields per record
        .from_path(path)?;

    let mut records = Vec::new();
    for result in rdr.deserialize() {
        match result  {
            Ok(record) => {
                records.push(record);
            }
            Err(e) => {
                // Print the current record that caused an error
                if let Some(pos) = e.position() {
                    println!("Error in record: {}", pos.byte());
                }
                return Err(Box::new(e));
            }
        }
    }
    Ok(records)
}

// Struct for TimePosting CSV
#[derive(Debug, Deserialize)]
struct TimePosting {
    Order: Option<String>,
    Activity: Option<String>,
    Functional_Location: Option<String>,
    Work_Actual: Option<String>,
    Created_On: Option<String>,
    Time_Posting_Date: Option<String>,
    Work_Center: Option<String>,
    Work_Planned: Option<String>,
    Extraction_Date: Option<String>,
    Personnel_ID: Option<String>,
    Actual_Finish_Date: Option<String>,
    Actual_State_Date: Option<String>,
    Actual_Finish_Time: Option<String>,
    Actual_Start_Time: Option<String>,
    Remaining_Duration: Option<String>,
    Remaining_Work: Option<String>,
    Distinct_Opr: Option<String>,
    Median_Time_Posting_Date: Option<String>,
}

// Struct for Activities CSV
#[derive(Debug, Deserialize, Clone)]
struct Activity {
    Order: Option<String>,
    Activity: Option<String>,
    Work_Center: Option<String>,
    Short_Text: Option<String>,
    Opr_User_Status: Option<String>,
    Opr_System_Status: Option<String>,
    Material: Option<String>,
    Functional_Location: Option<String>,
    System_Condition: Option<String>,
    Work_Planned: Option<String>,
    Work_Actual: Option<String>,
    Activity_Type: Option<String>,
    Actual_Finish_Date: Option<String>,
    Actual_Finish_Time: Option<String>,
    Actual_Start_Date: Option<String>,
    Actual_Start_Time: Option<String>,
    Earliest_End_Date: Option<String>,
    Ealiest_Finish_Date: Option<String>,
    Equipment: Option<String>,
    Work_Forecast: Option<String>,
    Latest_Finish_Date: Option<String>,
    Latest_Finish_Time: Option<String>,
    Latest_Start_Date: Option<String>,
    Latest_Start_Time: Option<String>,
    Location: Option<String>,
    Long_Text_Flag: Option<String>,
    Number: Option<String>,
    Long_Txt_Key: Option<String>,
    Unloading_Point: Option<String>,
}

// Struct for Jobs CSV
#[derive(Debug, Deserialize, Clone)]
struct Job {
    Priority: Option<String>,
    System_Status: Option<String>,
    Order_Type: Option<String>,
    Priority_Original: Option<String>,
    Work_Center_Main: Option<String>,
    Order: Option<String>,
    Notification_No: Option<String>,
    Basic_Start_Date: Option<String>,
    Latest_Allowed_Finish_Date: Option<String>,
    Description_1: Option<String>,
    Functional_Location: Option<String>,
    Description_2: Option<String>,
    User_Status: Option<String>,
    Original_Latest_Allowed_Finish_Date: Option<String>,
    Revision: Option<String>,
    System_Condition: Option<String>,
    Maintenance_Plan: Option<String>,
    VIS: Option<String>,
    Material_Number: Option<String>,
    Priority_Type: Option<String>,
    Actual_Release_Date: Option<String>,
    Actual_Finish_Date: Option<String>,
    Description_3: Option<String>,
    User_Status_All: Option<String>,
    Changed_On_Date: Option<String>,
    Plant_Section: Option<String>,
    Description_4: Option<String>,
    Equipment: Option<String>,
    Created_On_Date: Option<String>,
    Created_On_Year: Option<String>,
    Basic_Finish_Date: Option<String>,
    Scheduled_Finish_Date: Option<String>,
    Scheduled_Start_Date: Option<String>,
    Actual_Start_Date: Option<String>,
    Leading_Order: Option<String>,
    Suporior_Order: Option<String>,
    Priority_Desc: Option<String>,
    Maintenance_Item: Option<String>,
    Original_Start_Date: Option<String>,
    Earliest_Start_Date: Option<String>,
    Maintenance_Plant: Option<String>,
    Maint_Plan_Key: Option<String>,
    Priority_Original_Finish_Days: Option<String>,
    Priority_Original_Start_Days: Option<String>,
    Extraction_Date: Option<String>,
    System_Condition_Desc: Option<String>,
    Priority_Days: Option<String>,
}

fn get_header_set() -> HashSet<&'static str>{
    let headers_list = [
        "Priority,System_Status,Order_Type,Priority_Original,Work_Center_Main,Order,Notification_No,Basic_Start_Date,Latest_Allowed_Finish_Date,Description_1,Functional_Location,Description_2,User_Status,Original_Latest_Allowed_Finish_Date,Revision,System_Condition,Maintenance_Plan,VIS,Material_Number,Priority_Type,Actual_Release_Date,Actual_Finish_Date,Description_3,User_Status_All,Changed_On_Date,Plant_Section,Description_4,Equipment,Created_On_Date,Created_On_Year,Basic_Finish_Date,Scheduled_Finish_Date,Scheduled_Start_Date,Actual_Start_Date,Leading_Order,Suporior_Order,Priority_Desc,Maintenance_Item,Original_Start_Date,Earliest_Start_Date,Maintenance_Plant,%Maint_Plan_Key,Priority_Original_Finish_Days,Priority_Original_Start_Days,Extraction_Date,System_Condition_Desc,Priority_Days",
        "Order,Activity,Work_Center,Short_Text,Opr_User_Status,Opr_System_Status,Material,Functional_Location,System_Condition,Work_Planned,Work_Actual,Activity_Type,Actual_Finish_Date,Actual_Finish_Time,Actual_Start_Date,Actual_Start_Time,Earliest_End_Date,Ealiest_Finish_Date,Equipment,Work_Forecast,Latest_Finish_Date,Latest_Finish_Time,Latest_Start_Date,Latest_Start_Time,Location,Long_Text_Flag,Number,Long_Txt_Key,Unloading_Point",
        "Order,Activity,Functional_Location,Work_Actual,Created_On,Time_Posting_Date,Work_Center,Work_Planned,Extraction_Date,Personnel_ID,Actual_Finish_Date,Actual_State_Date,Actual_Finish_Time,Actual_Start_Time,Remaining_Duration,Remaining_Work,Distinct_Opr,Median_Time_Posting_Date"
    ];
    
    let mut headers_set = HashSet::new();
    
    for header_list in &headers_list {
        for header in header_list.split(',') {
            headers_set.insert(header);
        }
    }
    headers_set
}