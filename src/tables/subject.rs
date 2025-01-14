use postgres::{Client, NoTls};
use slint::{Model, ModelRc, SharedString, StandardListViewItem, TableColumn, VecModel};

use crate::*;
use std::{error::Error, rc::Rc};

fn get_colums() -> ModelRc<TableColumn> {
    let mut id = TableColumn::default();
    let mut name = TableColumn::default();
    id.title = "ID".into();
    name.title = "Шифр".into();
    let colums = Rc::new(VecModel::from(vec![
        id,
        name
    ]));
    ModelRc::from(colums)
}

fn get_data() -> Result<ModelRc<ModelRc<StandardListViewItem>>, postgres::Error> {
    let mut client = Client::connect(CONNECTION, NoTls)?;
    let mut data: Rc<VecModel<ModelRc<StandardListViewItem>>> = 
        Rc::from(VecModel::from(Vec::new()));
    for row in client.query("select * from subject;", &[])? {
        let id: i32 = row.get(0);
        let name: &str = row.get(1);
        
        let row_data: Rc<VecModel<StandardListViewItem>> = Rc::from(VecModel::from(vec![
            id.to_string().as_str().into(),
            name.into()
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
            crate::to_excel::table_to_excel(crate::to_excel::export_subject);
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
            let name = data_row.row_data(1).unwrap().text;
            if let Err(_) = change_row(id, &name) {
                show_error_window("Ошибка данных").unwrap();
            }
        }
    });

    ui.set_window_title("Предметы".into());
    ui.set_columns(get_colums());
    ui.set_data(get_data()?);

    ui.show()?;
    Ok(())
}

fn add_new() -> Result<(), Box<dyn Error>> {
    let ui = crate::AddSubject::new()?;

    ui.on_ok({
        let ui_handle = ui.as_weak();
        move |name| {
            let ui = ui_handle.unwrap();
            let client = Client::connect(CONNECTION, NoTls);
            if let Ok(mut client) = client {
                let query = format!("select add_subject('{name}');");
                if let Err(_) = client.query(&query, &[]) {
                    show_error_window("Ошибка данных").unwrap();
                }
            }
        }
    });

    ui.show()?;
    Ok(())
}

fn change_row(id: i64, name: &str) -> Result<(), Box<dyn Error>> {
    let ui = crate::ChangeSubject::new()?;

    ui.set_name_value(name.into());

    ui.on_ok({
        let ui_handle = ui.as_weak();
        move |name| {
            let ui = ui_handle.unwrap();
            let client = Client::connect(CONNECTION, NoTls);
            if let Ok(mut client) = client {
                let query = format!("select update_subject({id}, '{name}');");
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
                let query = format!("select delete_subject({id});");
                if let Err(_) = client.query(&query, &[]) {
                    show_error_window("Ошибка данных").unwrap();
                }
            }
        }
    );

    ui.show()?;
    Ok(())
}