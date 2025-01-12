use postgres::{Client, NoTls};
use slint::{Model, ModelRc, SharedString, StandardListViewItem, TableColumn, VecModel};

use crate::*;
use std::{error::Error, rc::Rc};

fn get_colums() -> ModelRc<TableColumn> {
    let mut id = TableColumn::default();
    let mut first_name = TableColumn::default();
    let mut second_name = TableColumn::default();
    let mut middle_name = TableColumn::default();
    let mut group = TableColumn::default();
    id.title = "ID".into();
    first_name.title = "Имя".into();
    second_name.title = "Фамилия".into();
    middle_name.title = "Отчетство".into();
    group.title = "Группа".into();
    let colums = Rc::new(VecModel::from(vec![
        id,
        first_name,
        second_name,
        middle_name,
        group
    ]));
    ModelRc::from(colums)
}

fn get_data() -> Result<ModelRc<ModelRc<StandardListViewItem>>, postgres::Error> {
    let mut client = Client::connect(CONNECTION, NoTls)?;
    let mut data: Rc<VecModel<ModelRc<StandardListViewItem>>> = 
        Rc::from(VecModel::from(Vec::new()));
    for row in client.query("select * from get_students;", &[])? {
        let id: i32 = row.get(0);
        let first_name: &str = row.get(1);
        let second_name: &str = row.get(2);
        let middle_name: &str = row.get(3);
        let year: i64 = row.get(4);
        let number: i64 = row.get(5);
        let cifr: &str = row.get(6);
        let group = format!("{cifr}-{year}-{number}");
        
        let row_data: Rc<VecModel<StandardListViewItem>> = Rc::from(VecModel::from(vec![
            id.to_string().as_str().into(),
            first_name.into(),
            second_name.into(),
            middle_name.into(),
            group.as_str().into()
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
            crate::to_excel::table_to_excel(crate::to_excel::export_student);
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

            let id: i32 = data_row.row_data(0).unwrap().text.parse().unwrap();
            let f_name: &str = &data_row.row_data(1).unwrap().text;
            let s_name: &str = &data_row.row_data(2).unwrap().text;
            let m_name: &str = &data_row.row_data(3).unwrap().text;
            let group: &str = &data_row.row_data(4).unwrap().text;

            if let Err(_) = change_row(
                id, f_name, s_name, m_name, group
            ) {
                show_error_window("Ошибка данных").unwrap();
            }
        }
    });

    ui.set_window_title("Студенты".into());
    ui.set_columns(get_colums());
    ui.set_data(get_data()?);

    ui.show()?;
    Ok(())
}

fn get_groups() -> Result<Vec<SharedString>, postgres::Error> {
    let mut groups: Vec<SharedString> = Vec::new();
    let mut client = Client::connect(CONNECTION, NoTls)?;
    for row in client.query("select name from get_group;", &[])? {
        let cifr: &str = row.get(0);
        groups.push(cifr.into());
    }
    Ok(groups)
}

fn add_new() -> Result<(), Box<dyn Error>> {
    let ui = crate::AddStudent::new()?;

    ui.on_ok({
        let ui_handle = ui.as_weak();
        move |f_name, s_name, m_name, group| {
            let ui = ui_handle.unwrap();
            let client = Client::connect(CONNECTION, NoTls);
            if let Ok(mut client) = client {
                if let Some((cifr, year, number)) = crate::tables::group::group_from_str(&group) {
                    let query = format!(
                        "select add_student('{f_name}', '{s_name}', '{m_name}', '{cifr}', {year}, {number});"
                    );
                    if let Err(_) = client.query(&query, &[]) {
                        show_error_window("Ошибка данных").unwrap();
                    }
                } else {
                    show_error_window("Ошибка данных").unwrap();
                }
            }
        }
    });

    ui.set_groups(
        ModelRc::from( Rc::new(
            VecModel::from( get_groups()? )
        ) )
    );

    ui.show()?;
    Ok(())
}

fn change_row(id: i32, f_name: &str, s_name: &str, m_name: &str, group: &str) -> Result<(), Box<dyn Error>> {
    let ui = crate::ChangeStudent::new()?;

    ui.on_ok({
        let ui_handle = ui.as_weak();
        move |f_name, s_name, m_name, group| {
            let ui = ui_handle.unwrap();
            let client = Client::connect(CONNECTION, NoTls);
            if let Ok(mut client) = client {
                if let Some((cifr, year, number)) = crate::tables::group::group_from_str(&group) {
                    let query = format!(
                        "select update_student({id}, '{f_name}', '{s_name}', '{m_name}', '{cifr}', {year}, {number});"
                    );
                    if let Err(_) = client.query(&query, &[]) {
                        show_error_window("Ошибка данных").unwrap();
                    }
                } else {
                    show_error_window("Ошибка данных").unwrap();
                }
            }
        }
    });

    ui.on_delete(
        move || {
            let client = Client::connect(CONNECTION, NoTls);
            if let Ok(mut client) = client {
                let query = format!("select delete_student({id});");
                if let Err(_) = client.query(&query, &[]) {
                    show_error_window("Ошибка данных").unwrap();
                }
            }
        }
    );

    ui.set_f_name_value(f_name.into());
    ui.set_s_name_value(s_name.into());
    ui.set_m_name_value(m_name.into());
    ui.set_group_value(group.into());

    ui.set_groups(
        ModelRc::from( Rc::new(
            VecModel::from( get_groups()? )
        ) )
    );

    ui.show()?;
    Ok(())
}