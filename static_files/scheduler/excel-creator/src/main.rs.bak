
fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Create a new spreadsheet
    let mut book = umya_spreadsheet::new_file();

    book.get_sheet_by_name_mut("Sheet1").unwrap().get_cell_mut("A1").set_value("Hello excel");

    let path = std::path::Path::new("example.xlsx");
    let _ = umya_spreadsheet::writer::xlsx::write(&book, path);

    println!("Excel file 'example.xlsx' created successfully.");

    Ok(())
}

