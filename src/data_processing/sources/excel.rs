use calamine::{open_workbook, Xlsx};
use std::path::Path;
use std::fs::File;
use std::io::BufReader;

// load in the main data file. Remember, take small steps. The first thing is to simply 
// get the complete file into memory.
// TODO
//      TODO Load in file
//      TODO 
pub fn load_data_file(file_path: &Path) -> Result<Xlsx<BufReader<File>>, calamine::Error> {
    let mut workbook: Xlsx<_> = open_workbook(file_path)?;
    println!("Successfully loaded file.");
    Ok(workbook)
}

