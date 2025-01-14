use std::path::PathBuf;
use crate::*;
use authorization::Student;
use native_dialog::FileDialog;
use postgres::{Client, NoTls};
use rust_xlsxwriter::{worksheet, Workbook, Worksheet};
use tables::specialization;

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
    let mut current_row = 1;

    worksheet.write(0, 0, "ID").unwrap();
    worksheet.write(0, 1, "Название").unwrap();
    worksheet.write(0, 2, "Специальность").unwrap();

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

pub fn export_specialization(worksheet: &mut Worksheet) -> () {
    let mut client = Client::connect(CONNECTION, NoTls).unwrap();
    let mut current_row = 1;

    worksheet.write(0, 0, "ID").unwrap();
    worksheet.write(0, 1, "Шифр").unwrap();
    worksheet.write(0, 2, "Полное название").unwrap();

    for row in client.query(r#"select * from "specialization";"#, &[]).unwrap() {
        let id: i32 = row.get(0);
        let cifr: &str = row.get(1);
        let full_name: &str = row.get(2);
        worksheet.write(current_row, 0, id).unwrap();
        worksheet.write(current_row, 1, cifr).unwrap();
        worksheet.write(current_row, 2, full_name).unwrap();
        
        current_row += 1;
    }
}

pub fn export_student(worksheet: &mut Worksheet) -> () {
    let mut client = Client::connect(CONNECTION, NoTls).unwrap();
    let mut current_row = 1;

    worksheet.write(0, 0, "ID").unwrap();
    worksheet.write(0, 1, "Имя").unwrap();
    worksheet.write(0, 2, "Фамилия").unwrap();
    worksheet.write(0, 3, "Отчетство").unwrap();
    worksheet.write(0, 4, "Группа").unwrap();

    for row in client.query("select * from get_students;", &[]).unwrap() {
        let id: i32 = row.get(0);
        let first_name: &str = row.get(1);
        let second_name: &str = row.get(2);
        let middle_name: &str = row.get(3);
        let year: i64 = row.get(4);
        let number: i64 = row.get(5);
        let cifr: &str = row.get(6);
        let group = format!("{cifr}-{year}-{number}");

        worksheet.write(current_row, 0, id).unwrap();
        worksheet.write(current_row, 1, first_name).unwrap();
        worksheet.write(current_row, 2, second_name).unwrap();
        worksheet.write(current_row, 3, middle_name).unwrap();
        worksheet.write(current_row, 4, group).unwrap();
        
        current_row += 1;
    }
}

pub fn export_subject(worksheet: &mut Worksheet) -> () {
    let mut client = Client::connect(CONNECTION, NoTls).unwrap();
    let mut current_row = 1;

    worksheet.write(0, 0, "ID").unwrap();
    worksheet.write(0, 1, "Название").unwrap();

    for row in client.query("select * from subject;", &[]).unwrap() {
        let id: i32 = row.get(0);
        let name: &str = row.get(1);
        worksheet.write(current_row, 0, id).unwrap();
        worksheet.write(current_row, 1, name).unwrap();
        
        current_row += 1;
    }
}

pub fn export_academic_plan(worksheet: &mut Worksheet) -> () {
    let mut client = Client::connect(CONNECTION, NoTls).unwrap();
    let mut current_row = 1;

    worksheet.write(0, 0, "ID").unwrap();
    worksheet.write(0, 1, "Название специальности").unwrap();
    worksheet.write(0, 2, "Название предмета").unwrap();
    worksheet.write(0, 3, "Вид аттестации").unwrap();
    worksheet.write(0, 4, "Количество часов").unwrap();
    worksheet.write(0, 5, "Семестр").unwrap();

    for row in client.query("select * from get_academic_plan;", &[]).unwrap() {
        let id: i32 = row.get(0);
        let specialization_name: &str = row.get(1);
        let subject_name: &str = row.get(2);
        let certification_type: &str = row.get(3);
        let hours: i64 = row.get(4);
        let semester: i64 = row.get(5);
        worksheet.write(current_row, 0, id).unwrap();
        worksheet.write(current_row, 1, specialization_name).unwrap();
        worksheet.write(current_row, 2, subject_name).unwrap();
        worksheet.write(current_row, 3, certification_type).unwrap();
        worksheet.write(current_row, 4, hours).unwrap();
        worksheet.write(current_row, 5, semester).unwrap();
        
        current_row += 1;
    }
}

pub fn write_admin_marks(worksheet: &mut Worksheet, cifr: &str, year: i32, number: i32) -> () {
    let mut client = Client::connect(CONNECTION, NoTls).unwrap();
    let mut current_row = 1;

    worksheet.write(0, 0, "ID").unwrap();
    worksheet.write(0, 1, "Имя").unwrap();
    worksheet.write(0, 2, "Фамилия").unwrap();
    worksheet.write(0, 3, "Предмет").unwrap();
    worksheet.write(0, 4, "Семестр").unwrap();
    worksheet.write(0, 5, "Вид аттестации").unwrap();
    worksheet.write(0, 6, "Оценка").unwrap();

    let query = format!("select * from get_student_card_for('{cifr}', {year}, {number})");
    for row in client.query(&query, &[]).unwrap() {
        let id: i32 = row.get(0);
        let f_name: &str = row.get(1);
        let s_name: &str = row.get(2);
        let subject: &str = row.get(3);
        let semester: i64 = row.get(4);
        let c_type: &str = row.get(5);
        let mark: Option<&str> = row.get(6);
        worksheet.write(current_row, 0, id).unwrap();
        worksheet.write(current_row, 1, f_name).unwrap();
        worksheet.write(current_row, 2, s_name).unwrap();
        worksheet.write(current_row, 3, subject).unwrap();
        worksheet.write(current_row, 4, semester).unwrap();
        worksheet.write(current_row, 5, c_type).unwrap();
        worksheet.write(current_row, 6, mark.unwrap_or("Нет оценки")).unwrap();
        
        current_row += 1;
    }
}

pub fn export_admin_marks(cifr: &str, year: i32, number: i32) {
    let path = FileDialog::new()
        .add_filter("Xlsx file", &["xlsx"])
        .show_save_single_file().unwrap();

    if let Some(path) = path {
        let mut workbook = Workbook::new();
        let worksheet = workbook.add_worksheet();

        write_admin_marks(worksheet, cifr, year, number);

        workbook.save(path).unwrap();
    }
}

pub fn write_student_group(worksheet: &mut Worksheet, cifr: &str, year: i64, number: i64) -> () {
    let mut client = Client::connect(CONNECTION, NoTls).unwrap();
    let mut current_row = 1;

    worksheet.write(0, 0, "ФИО").unwrap();

    let query = format!(
        "SELECT second_name, first_name, middle_name FROM get_students_in_group('{}', {}, {}) order by second_name;",
        cifr,
        year,
        number,
    );
    for row in client.query(&query, &[]).unwrap() {
        let s_name: &str = row.get(0);
        let f_name: &str = row.get(1);
        let m_name: &str = row.get(2);
        let name = format!("{s_name} {f_name} {m_name}");
        worksheet.write(current_row, 0, name).unwrap();
        
        current_row += 1;
    }
}

pub fn export_student_group(student: &Student) {
    let path = FileDialog::new()
        .add_filter("Xlsx file", &["xlsx"])
        .show_save_single_file().unwrap();

    if let Some(path) = path {
        let mut workbook = Workbook::new();
        let worksheet = workbook.add_worksheet();

        write_student_group(worksheet, &student.group.cifr, student.group.year, student.group.number);

        workbook.save(path).unwrap();
    }
}

pub fn write_student_plan(worksheet: &mut Worksheet, student_number: i32, course: i32) -> () {
    let mut client = Client::connect(CONNECTION, NoTls).unwrap();
    let mut current_row = 1;

    worksheet.write(0, 0, "Предмет").unwrap();
    worksheet.write(0, 1, "Семестр").unwrap();
    worksheet.write(0, 2, "Часы").unwrap();
    worksheet.write(0, 3, "Вид аттестации").unwrap();

    let query = format!(
        "select * from get_student_plan({}) where semester = {} or semester = {} order by semester;",
        student_number,
        course << 1,
        (course << 1) - 1
    );
    for row in client.query(&query, &[]).unwrap() {
        let sub: &str = row.get(0);
        let semester: i64 = row.get(1);
        let hours: i64 = row.get(2);
        let c_type: &str = row.get(3);

        worksheet.write(current_row, 0, sub).unwrap();
        worksheet.write(current_row, 1, semester).unwrap();
        worksheet.write(current_row, 2, hours).unwrap();
        worksheet.write(current_row, 3, c_type).unwrap();
        
        current_row += 1;
    }
}

pub fn export_student_plan(student: &Student, course: i32) {
    let path = FileDialog::new()
        .add_filter("Xlsx file", &["xlsx"])
        .show_save_single_file().unwrap();

    if let Some(path) = path {
        let mut workbook = Workbook::new();
        let worksheet = workbook.add_worksheet();

        write_student_plan(worksheet, student.number, course);

        workbook.save(path).unwrap();
    }
}

pub fn write_student_marks(worksheet: &mut Worksheet, student_number: i32) -> () {
    let mut client = Client::connect(CONNECTION, NoTls).unwrap();
    let mut current_row = 1;

    worksheet.write(0, 0, "Предмет").unwrap();
    worksheet.write(0, 1, "Семестр").unwrap();
    worksheet.write(0, 2, "Часы").unwrap();
    worksheet.write(0, 3, "Вид аттестации").unwrap();
    worksheet.write(0, 3, "Оценка").unwrap();

    let query = format!(
        "SELECT * FROM get_marks_student({}) order by semester;",
        student_number
    );
    for row in client.query(&query, &[]).unwrap() {
        let sub: &str = row.get(0);
        let semester: i64 = row.get(1);
        let hours: i64 = row.get(2);
        let c_type: &str = row.get(3);
        let mark: Option<&str> = row.get(4);

        worksheet.write(current_row, 0, sub).unwrap();
        worksheet.write(current_row, 1, semester).unwrap();
        worksheet.write(current_row, 2, hours).unwrap();
        worksheet.write(current_row, 3, c_type).unwrap();
        worksheet.write(current_row, 4, mark.unwrap_or("Нет оценки")).unwrap();
        
        current_row += 1;
    }
}

pub fn export_student_marks(student: &Student) {
    let path = FileDialog::new()
        .add_filter("Xlsx file", &["xlsx"])
        .show_save_single_file().unwrap();

    if let Some(path) = path {
        let mut workbook = Workbook::new();
        let worksheet = workbook.add_worksheet();

        write_student_marks(worksheet, student.number);

        workbook.save(path).unwrap();
    }
}