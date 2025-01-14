use postgres::{Client, NoTls};
use slint::{Model, ModelRc, SharedString, StandardListViewItem, TableColumn, VecModel};

use crate::*;
use std::{error::Error, rc::Rc, vec};

fn get_data(plan_id: i32, cifr: &str, year: i32, number: i32) -> Result<ModelRc<ModelRc<StandardListViewItem>>, postgres::Error> {
    let mut client = Client::connect(CONNECTION, NoTls)?;
    let mut data: Rc<VecModel<ModelRc<StandardListViewItem>>> = 
        Rc::from(VecModel::from(Vec::new()));
    let query = format!("select * from get_students_by_plan_in_group({plan_id}, '{cifr}', {year}, {number});");
    for row in client.query(&query, &[])? {
        let id: i64 = row.get(0);
        let f_name: &str = row.get(1);
        let s_name: &str = row.get(2);
        let m_name: &str = row.get(3);
        let name = format!("{f_name} {s_name} {m_name}");
        let c_type: &str = row.get(4);
        let mark: Option<&str> = row.get(5);
        
        let row_data: Rc<VecModel<StandardListViewItem>> = Rc::from(VecModel::from(vec![
            id.to_string().as_str().into(),
            name.as_str().into(),
            c_type.into(),
            mark.unwrap_or("Нет оценки").into()
        ]));
        data.push(ModelRc::from(row_data));
    }
    Ok(ModelRc::from(data))
}

pub fn show_full_table(plan_id: i32, cifr: &str, year: i32, number: i32) -> Result<(), Box<dyn Error>> {
    let ui = crate::TeacherTableWindow::new()?;

    ui.on_refresh({
        let ui_handle = ui.as_weak();
        let cifr = cifr.to_owned();
        move || {
            let ui = ui_handle.unwrap();
            ui.set_data(get_data(plan_id, &cifr, year, number).unwrap());
        }
    });


    ui.on_mark({
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
            let name: &str = &data_row.row_data(1).unwrap().text;
            let c_type: &str = &data_row.row_data(2).unwrap().text;
            if let Err(_) = mark_student(id, plan_id, c_type, name) {
                show_error_window("Ошибка данных").unwrap();
            }
        }
    });

    ui.set_data(get_data(plan_id, cifr, year, number)?);

    ui.show()?;
    Ok(())
}

fn mark_student(student_id: i32, plan_id: i32, c_type: &str, name: &str) -> Result<(), Box<dyn Error>> {
    let ui = crate::TeacherMarkWindow::new()?;

    ui.on_ok({
        let ui_handle = ui.as_weak();
        move |mark| {
            let ui = ui_handle.unwrap();
            let client = Client::connect(CONNECTION, NoTls);
            if let Ok(mut client) = client {
                let query = format!("select mark_student({plan_id}, {student_id}, '{mark}');");
                if let Err(_) = client.query(&query, &[]) {
                    show_error_window("Ошибка данных").unwrap();
                }
            }
        }
    });

    ui.set_student_name(name.into());
    ui.set_marks(
        ModelRc::from( Rc::new(
            VecModel::from( get_marks(c_type) )
        ) )
    );

    ui.show()?;
    Ok(())
}

fn get_marks(c_type: &str) -> Vec<SharedString> {
    if c_type == "Зачет" {
        vec!["Зачет".into(), "Не зачет".into(), "Не явка".into()]
    } else {
        vec![
            "Не явка".into(), 
            "Не удовлетворительно".into(), 
            "Удовлетворительно".into(), 
            "Хорошо".into(), 
            "Отлично".into()
        ]
    }
}