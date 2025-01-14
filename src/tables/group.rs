use authorization::Student;
use postgres::{Client, NoTls};
use slint::{Model, ModelRc, SharedString, StandardListViewItem, TableColumn, VecModel};

use crate::*;
use std::{error::Error, rc::Rc};

fn get_colums() -> ModelRc<TableColumn> {
    let mut id = TableColumn::default();
    let mut name = TableColumn::default();
    let mut specialization = TableColumn::default();
    id.title = "ID".into();
    name.title = "Название".into();
    specialization.title = "Специальность".into();
    let colums = Rc::new(VecModel::from(vec![
        id,
        name,
        specialization,
    ]));
    ModelRc::from(colums)
}

fn get_data() -> Result<ModelRc<ModelRc<StandardListViewItem>>, postgres::Error> {
    let mut client = Client::connect(CONNECTION, NoTls)?;
    let mut data: Rc<VecModel<ModelRc<StandardListViewItem>>> = 
        Rc::from(VecModel::from(Vec::new()));
    for row in client.query("select * from get_group;", &[])? {
        let id: i32 = row.get(0);
        let name: &str = row.get(1);
        let specialization: &str = row.get(2);
        
        let row_data: Rc<VecModel<StandardListViewItem>> = Rc::from(VecModel::from(vec![
            id.to_string().as_str().into(),
            name.into(),
            specialization.into()
        ]));
        data.push(ModelRc::from(row_data));
    }
    Ok(ModelRc::from(data))
}

pub fn show_full_table() -> Result<(), Box<dyn Error>> {
    let ui = crate::FullTableWindow::new()?;

    ui.on_refresh({
        let ui_handle = ui.as_weak();
        move || {
            let ui = ui_handle.unwrap();
            ui.set_data(get_data().unwrap());
        }
    });

    ui.on_export_to_excel(
        move || {
            crate::to_excel::table_to_excel(crate::to_excel::export_group);
        }
    );

    ui.on_add_new(move || {
        add_new().unwrap();
    });

    ui.on_change_row({
        let ui_handle = ui.as_weak();
        move |row_index| {
            if row_index < 0 {
                show_error_window("Выберите строку").unwrap();
                return;
            }
            let ui = ui_handle.unwrap();
            let data = ui.get_data();
            let data: &VecModel<ModelRc<StandardListViewItem>> = data.as_any().downcast_ref().unwrap();
            let data_row = data.row_data(row_index as usize).unwrap();
            let data_row: &VecModel<StandardListViewItem> = data_row.as_any().downcast_ref().unwrap();
            let id: i64 = data_row.row_data(0).unwrap().text.parse().unwrap();
            let group = data_row.row_data(1).unwrap().text;
            if let Some((cifr, year, number)) = group_from_str(&group) {
                if let Err(_) = change_row(id as i32, year, number, &cifr) {
                    show_error_window("Ошибка данных").unwrap();
                }
            } else {
                show_error_window("Ошибка данных").unwrap();
            }
        }
    });

    ui.set_window_title("Группы".into());
    ui.set_columns(get_colums());
    ui.set_data(get_data()?);

    ui.show()?;
    Ok(())
}

fn get_cifrs() -> Result<Vec<SharedString>, postgres::Error> {
    let mut cifrs: Vec<SharedString> = Vec::new();
    let mut client = Client::connect(CONNECTION, NoTls)?;
    for row in client.query(r#"select cifr from "specialization";"#, &[])? {
        let cifr: &str = row.get(0);
        cifrs.push(cifr.into());
    }
    Ok(cifrs)
}

fn add_new() -> Result<(), Box<dyn Error>> {
    let ui = crate::AddGroup::new()?;

    ui.on_ok({
        let ui_handle = ui.as_weak();
        move |cifr, year, number| {
            let ui = ui_handle.unwrap();
            let client = Client::connect(CONNECTION, NoTls);
            if let Ok(mut client) = client {
                let cifr: &str = &cifr;
                let query = format!("select add_group({year}, {number}, '{cifr}');");
                if let Err(_) = client.query(&query, &[]) {
                    show_error_window("Ошибка данных").unwrap();
                }
            }
        }
    });

    ui.set_cifrs(
        ModelRc::from( Rc::new(
            VecModel::from( get_cifrs()? )
        ) )
    );

    ui.show()?;
    Ok(())
}

fn change_row(id: i32, year: i32, number: i32, cifr: &str) -> Result<(), Box<dyn Error>> {
    let ui = crate::ChangeGroup::new()?;

    ui.on_ok({
        let ui_handle = ui.as_weak();
        move |cifr, year, number| {
            let ui = ui_handle.unwrap();
            let client = Client::connect(CONNECTION, NoTls);
            if let Ok(mut client) = client {
                let cifr: &str = &cifr;
                let query = format!("select update_group({id}, {year}, {number}, '{cifr}');");
                if let Err(_) = client.query(&query, &[]) {
                    show_error_window("Ошибка данных").unwrap();
                }
            }
        }
    });

    ui.on_delete(
        move || {
            let client = Client::connect(CONNECTION, NoTls);
            if let Ok(mut client) = client {
                let query = format!("select delete_group({id});");
                if let Err(_) = client.query(&query, &[]) {
                    show_error_window("Ошибка данных").unwrap();
                }
            }
        }
    );

    ui.set_cifr_value(cifr.into());
    ui.set_number_value(number);
    ui.set_year_value(year);

    ui.set_cifrs(
        ModelRc::from( Rc::new(
            VecModel::from( get_cifrs()? )
        ) )
    );

    ui.show()?;
    Ok(())
}

pub fn group_from_str(s: &str) -> Option<(&str, i32, i32)> {
    let mut iter = s.split('-');
    let cifr = iter.next()?.into();
    let year: i32 = iter.next()?.parse().unwrap_or_default();
    let number: i32 = iter.next()?.parse().unwrap_or_default();
    Some((cifr, year, number))
}

pub fn show_group_for_student(student: &Student) -> Result<(), Box<dyn Error>> {
    let ui = crate::StudentGroupTableWindow::new()?;

    ui.on_refresh({
        let ui_handle = ui.as_weak();
        let student = student.clone();
        move || {
            let ui = ui_handle.unwrap();
            ui.set_data(get_data_for_student(&student).unwrap());
        }
    });

    ui.on_export_to_excel({
        let ui_handle = ui.as_weak();
        let student = student.clone();
        move || {
            crate::to_excel::export_student_group(&student);
        }
    });

    ui.set_data(get_data_for_student(student)?);

    ui.show()?;
    Ok(())
}

fn get_data_for_student(student: &Student) -> Result<ModelRc<ModelRc<StandardListViewItem>>, postgres::Error> {
    let mut client = Client::connect(CONNECTION, NoTls)?;
    let mut data: Rc<VecModel<ModelRc<StandardListViewItem>>> = 
        Rc::from(VecModel::from(Vec::new()));
    let query = format!(
        "SELECT second_name, first_name, middle_name FROM get_students_in_group('{}', {}, {}) order by second_name;",
        student.group.cifr,
        student.group.year,
        student.group.number,
    );
    for row in client.query(&query, &[]).unwrap() {
        let s_name: &str = row.get(0);
        let f_name: &str = row.get(1);
        let m_name: &str = row.get(2);
        let name = format!("{s_name} {f_name} {m_name}");
        
        let row_data: Rc<VecModel<StandardListViewItem>> = Rc::from(VecModel::from(vec![
            name.as_str().into()            
        ]));
        data.push(ModelRc::from(row_data));
    }
    Ok(ModelRc::from(data))
}