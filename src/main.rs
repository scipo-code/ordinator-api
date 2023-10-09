mod models; mod data_processing;

use calamine::{Xlsx};
use std::io::BufReader;
use std::fs::File;
use crate::models::scheduling_environment::{self, SchedulingEnvironment};

use std::path::Path;

use std::env;


use crate::data_processing::sources::excel::load_data_file;


fn main() {

    let args: Vec<String> = env::args().collect();
    let xlsx: Xlsx<BufReader<File>>;
    if args.len() > 1 {
        let file_path = Path::new(&args[1]);
        load_data_file(file_path).expect("Could not load data file.");
    } else {
        println!("Please provide the data file as an argument.");
    }
    
    // let scheduling_environment = SchedulingEnvironment::initialize_from_sources(work_orders, worker_environment);



    

}
