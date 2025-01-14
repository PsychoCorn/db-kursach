use postgres::{Client, NoTls};
use slint::{Model, ModelRc, SharedString, StandardListViewItem, TableColumn, VecModel};
use tables::specialization;

use crate::*;
use std::{error::Error, rc::Rc};
use authorization::get_hashed_password;

pub mod student {
    use super::*;

    fn get_colums() -> ModelRc<TableColumn> {
        let mut login = TableColumn::default();
        let mut id = TableColumn::default();
        id.title = "ID".into();
        login.title = "Логин".into();
        let colums = Rc::new(VecModel::from(vec![
            login,
            id
        ]));
        ModelRc::from(colums)
    }
    
    fn get_data() -> Result<ModelRc<ModelRc<StandardListViewItem>>, postgres::Error> {
        let mut client = Client::connect(CONNECTION, NoTls)?;
        let mut data: Rc<VecModel<ModelRc<StandardListViewItem>>> = 
            Rc::from(VecModel::from(Vec::new()));
        for row in client.query("select * from \"login_student\";", &[]).unwrap() {
            let login: &str = row.get(0);
            let id: i64 = row.get(1);
            
            let row_data: Rc<VecModel<StandardListViewItem>> = Rc::from(VecModel::from(vec![
                login.into(),
                id.to_string().as_str().into(),
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
                show_error_window("Невозможно экспортировать в xlsx").unwrap();
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
    
                let login: &str = &data_row.row_data(0).unwrap().text;
    
                if let Err(_) = change_row(login) {
                    show_error_window("Ошибка данных").unwrap();
                }
            }
        });
    
        ui.set_window_title("Логины студентов".into());
        ui.set_columns(get_colums());
        ui.set_data(get_data()?);
    
        ui.show()?;
        Ok(())
    }

    fn add_new() -> Result<(), Box<dyn Error>> {
        let ui = crate::AddStudentLoggin::new()?;
    
        ui.on_ok({
            let ui_handle = ui.as_weak();
            move |login, password, id| {
                let ui = ui_handle.unwrap();
                let client = Client::connect(CONNECTION, NoTls);
                if let Ok(mut client) = client {
                    let hashed_password = get_hashed_password(&password);
                    let query = format!(
                        "select create_student_login('{login}', '{hashed_password}', {id});"
                    );
                    if let Err(_) = client.query(&query, &[]) {
                        show_error_window("Ошибка данных").unwrap();
                    }
                }
            }
        });
    
        ui.show()?;
        Ok(())
    }

    fn change_row(login: &str) -> Result<(), Box<dyn Error>> {
        let ui = crate::ChangeStudentLoggin::new()?;

    
        ui.on_delete({
            let login = login.to_owned();
            move || {
                let client = Client::connect(CONNECTION, NoTls);
                if let Ok(mut client) = client {
                    let query = format!("select delete_student_login('{login}');");
                    if let Err(_) = client.query(&query, &[]) {
                        show_error_window("Ошибка данных").unwrap();
                    }
                }
            }
        });
    
        ui.show()?;
        Ok(())
    }

}

pub mod teachers {
    use super::*;

    fn get_colums() -> ModelRc<TableColumn> {
        let mut login = TableColumn::default();
        let mut id = TableColumn::default();
        login.title = "Логин".into();
        let colums = Rc::new(VecModel::from(vec![
            login,
        ]));
        ModelRc::from(colums)
    }
    
    fn get_data() -> Result<ModelRc<ModelRc<StandardListViewItem>>, postgres::Error> {
        let mut client = Client::connect(CONNECTION, NoTls)?;
        let mut data: Rc<VecModel<ModelRc<StandardListViewItem>>> = 
            Rc::from(VecModel::from(Vec::new()));
        for row in client.query("select login from \"users\" where id_role = 4;", &[]).unwrap() {
            let login: &str = row.get(0);
            
            let row_data: Rc<VecModel<StandardListViewItem>> = Rc::from(VecModel::from(vec![
                login.into(),
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
                show_error_window("Невозможно экспортировать в xlsx").unwrap();
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
    
                let login: &str = &data_row.row_data(0).unwrap().text;
    
                if let Err(_) = change_row(login) {
                    show_error_window("Ошибка данных").unwrap();
                }
            }
        });
    
        ui.set_window_title("Логины преподавателей".into());
        ui.set_columns(get_colums());
        ui.set_data(get_data()?);
    
        ui.show()?;
        Ok(())
    }

    fn add_new() -> Result<(), Box<dyn Error>> {
        let ui = crate::AddTeacherLoggin::new()?;
    
        ui.on_ok({
            let ui_handle = ui.as_weak();
            move |login, password| {
                let ui = ui_handle.unwrap();
                let client = Client::connect(CONNECTION, NoTls);
                if let Ok(mut client) = client {
                    let hashed_password = get_hashed_password(&password);
                    let query = format!(
                        "select create_teacher_login('{login}', '{hashed_password}');"
                    );
                    if let Err(_) = client.query(&query, &[]) {
                        show_error_window("Ошибка данных").unwrap();
                    }
                }
            }
        });
    
        ui.show()?;
        Ok(())
    }

    fn change_row(login: &str) -> Result<(), Box<dyn Error>> {
        let ui = crate::ChangeTeacherLoggin::new()?;

    
        ui.on_delete({
            let login = login.to_owned();
            move || {
                let client = Client::connect(CONNECTION, NoTls);
                if let Ok(mut client) = client {
                    let query = format!("select delete_teacher_login('{login}');");
                    if let Err(_) = client.query(&query, &[]) {
                        show_error_window("Ошибка данных").unwrap();
                    }
                }
            }
        });
    
        ui.show()?;
        Ok(())
    }

}