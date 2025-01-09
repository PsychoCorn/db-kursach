use std::path::PathBuf;
use crate::*;
use native_dialog::FileDialog;
use postgres::{Client, NoTls};
use rust_xlsxwriter::{worksheet, Workbook, Worksheet};

fn write_excel(path: PathBuf, callback: impl Fn(&mut Worksheet) -> ()) {
    let mut workbook = Workbook::new();
    let worksheet = workbook.add_worksheet();

    callback(worksheet);

    workbook.save(path).unwrap();
}

pub fn table_to_excel(callback: impl Fn(&mut Worksheet) -> ()) {
    let path = FileDialog::new()
        .add_filter("Xlsx file", &["xlsx"])
        .show_save_single_file().unwrap();

    if let Some(path) = path {
        write_excel(path, callback);
    }
}

pub fn export_group(worksheet: &mut Worksheet) -> () {
    let mut client = Client::connect(CONNECTION, NoTls).unwrap();
    let mut current_row = 0;

    for row in client.query("select * from get_group;", &[]).unwrap() {
        let id: i32 = row.get(0);
        let name: &str = row.get(1);
        let specialization: &str = row.get(2);
        worksheet.write(current_row, 0, id).unwrap();
        worksheet.write(current_row, 1, name).unwrap();
        worksheet.write(current_row, 2, specialization).unwrap();
        
        current_row += 1;
    }
}