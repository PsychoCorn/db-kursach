use postgres::{Client, NoTls};
use slint::{Model, ModelRc, SharedString, StandardListViewItem, TableColumn, VecModel};

use crate::*;
use std::{error::Error, rc::Rc};

fn get_colums() -> ModelRc<TableColumn> {
    let mut id = TableColumn::default();
    let mut cifr = TableColumn::default();
    let mut full_name = TableColumn::default();
    id.title = "ID".into();
    cifr.title = "Шифр".into();
    full_name.title = "Полное название".into();
    let colums = Rc::new(VecModel::from(vec![
        id,
        cifr,
        full_name,
    ]));
    ModelRc::from(colums)
}

fn get_data() -> Result<ModelRc<ModelRc<StandardListViewItem>>, postgres::Error> {
    let mut client = Client::connect(CONNECTION, NoTls)?;
    let mut data: Rc<VecModel<ModelRc<StandardListViewItem>>> = 
        Rc::from(VecModel::from(Vec::new()));
    for row in client.query(r#"select * from "specialization";"#, &[])? {
        let id: i32 = row.get(0);
        let cifr: &str = row.get(1);
        let full_name: &str = row.get(2);
        
        let row_data: Rc<VecModel<StandardListViewItem>> = Rc::from(VecModel::from(vec![
            id.to_string().as_str().into(),
            cifr.into(),
            full_name.into()
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
            crate::to_excel::table_to_excel(crate::to_excel::export_specialization);
        }
    );

    ui.on_add_new(move || {
        add_new().unwrap();
    });

    ui.on_change_row({
        let ui_handle = ui.as_weak();
        move |row_index| {
            let ui = ui_handle.unwrap();
            let data = ui.get_data();
            let data: &VecModel<ModelRc<StandardListViewItem>> = data.as_any().downcast_ref().unwrap();
            let data_row = data.row_data(row_index as usize).unwrap();
            let data_row: &VecModel<StandardListViewItem> = data_row.as_any().downcast_ref().unwrap();
            let id: i64 = data_row.row_data(0).unwrap().text.parse().unwrap();
            let cifr = data_row.row_data(1).unwrap().text;
            let name = data_row.row_data(2).unwrap().text;
            if let Err(_) = change_row(id, &cifr, &name) {
                show_error_window("Ошибка данных").unwrap();
            }
        }
    });

    ui.set_window_title("Специализация".into());
    ui.set_columns(get_colums());
    ui.set_data(get_data()?);

    ui.show()?;
    Ok(())
}

fn add_new() -> Result<(), Box<dyn Error>> {
    let ui = crate::AddSpecialization::new()?;

    ui.on_ok({
        let ui_handle = ui.as_weak();
        move |cifr: SharedString, name: SharedString| {
            let ui = ui_handle.unwrap();
            let client = Client::connect(CONNECTION, NoTls);
            if let Ok(mut client) = client {
                let cifr = cifr.as_str();
                let name = name.as_str();
                let query = format!("select add_specialization('{cifr}', '{name}');");
                if let Err(_) = client.query(&query, &[]) {
                    show_error_window("Ошибка данных").unwrap();
                }
            }
        }
    });

    ui.show()?;
    Ok(())
}

fn change_row(id: i64, cifr: &str, name: &str) -> Result<(), Box<dyn Error>> {
    let ui = crate::ChangeSpecialization::new()?;

    ui.set_cifr_value(cifr.into());
    ui.set_name_value(name.into());

    ui.on_ok({
        let ui_handle = ui.as_weak();
        move |cifr: SharedString, name: SharedString| {
            let ui = ui_handle.unwrap();
            let client = Client::connect(CONNECTION, NoTls);
            if let Ok(mut client) = client {
                let cifr = cifr.as_str();
                let name = name.as_str();
                let query = format!("select update_specialization({id}, '{cifr}', '{name}');");
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
                let query = format!("select delete_specialization({id});");
                if let Err(_) = client.query(&query, &[]) {
                    show_error_window("Ошибка данных").unwrap();
                }
            }
        }
    );

    ui.show()?;
    Ok(())
}