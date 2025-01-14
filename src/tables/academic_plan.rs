use authorization::Student;
use postgres::{Client, NoTls};
use slint::{Model, ModelRc, SharedString, StandardListViewItem, TableColumn, VecModel};
use tables::specialization;

use crate::*;
use std::{error::Error, rc::Rc};

fn get_colums() -> ModelRc<TableColumn> {
    let mut id = TableColumn::default();
    let mut specialization_name = TableColumn::default();
    let mut subject_name = TableColumn::default();
    let mut certification_type = TableColumn::default();
    let mut hours = TableColumn::default();
    let mut semester = TableColumn::default();
    id.title = "ID".into();
    specialization_name.title = "Название специальности".into();
    subject_name.title = "Название предмета".into();
    certification_type.title = "Вид аттестации".into();
    hours.title = "Количество часов".into();
    semester.title = "Семестр".into();
    let colums = Rc::new(VecModel::from(vec![
        id,
        specialization_name,
        subject_name,
        certification_type,
        hours,
        semester,
    ]));
    ModelRc::from(colums)
}

fn get_data() -> Result<ModelRc<ModelRc<StandardListViewItem>>, postgres::Error> {
    let mut client = Client::connect(CONNECTION, NoTls)?;
    let mut data: Rc<VecModel<ModelRc<StandardListViewItem>>> = 
        Rc::from(VecModel::from(Vec::new()));
    for row in client.query("select * from get_academic_plan;", &[]).unwrap() {
        let id: i32 = row.get(0);
        let specialization_name: &str = row.get(1);
        let subject_name: &str = row.get(2);
        let certification_type: &str = row.get(3);
        let hours: i64 = row.get(4);
        let semester: i64 = row.get(5);
        
        let row_data: Rc<VecModel<StandardListViewItem>> = Rc::from(VecModel::from(vec![
            id.to_string().as_str().into(),
            specialization_name.into(),
            subject_name.into(),
            certification_type.into(),
            hours.to_string().as_str().into(),
            semester.to_string().as_str().into(),
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
            crate::to_excel::table_to_excel(crate::to_excel::export_academic_plan);
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
            let spec: &str = &data_row.row_data(1).unwrap().text;
            let sub: &str = &data_row.row_data(2).unwrap().text;
            let c: &str = &data_row.row_data(3).unwrap().text;
            let h: i32 = data_row.row_data(4).unwrap().text.parse().unwrap();
            let s: i32 = data_row.row_data(5).unwrap().text.parse().unwrap();

            if let Err(_) = change_row(
                id, spec, sub, c, h, s
            ) {
                show_error_window("Ошибка данных").unwrap();
            }
        }
    });

    ui.set_window_title("Учебный план".into());
    ui.set_columns(get_colums());
    ui.set_data(get_data()?);

    ui.show()?;
    Ok(())
}

fn get_specializations() -> Result<Vec<SharedString>, postgres::Error> {
    let mut result: Vec<SharedString> = Vec::new();
    let mut client = Client::connect(CONNECTION, NoTls)?;
    for row in client.query("select full_name from specialization;", &[])? {
        let name: &str = row.get(0);
        result.push(name.into());
    }
    Ok(result)
}

fn get_subjects() -> Result<Vec<SharedString>, postgres::Error> {
    let mut result: Vec<SharedString> = Vec::new();
    let mut client = Client::connect(CONNECTION, NoTls)?;
    for row in client.query("select name from subject;", &[])? {
        let name: &str = row.get(0);
        result.push(name.into());
    }
    Ok(result)
}

fn get_certification_type() -> Result<Vec<SharedString>, postgres::Error> {
    let mut result: Vec<SharedString> = Vec::new();
    let mut client = Client::connect(CONNECTION, NoTls)?;
    for row in client.query("select name from certification_type;", &[])? {
        let name: &str = row.get(0);
        result.push(name.into());
    }
    Ok(result)
}

fn add_new() -> Result<(), Box<dyn Error>> {
    let ui = crate::AddAcademicPlan::new()?;

    ui.on_ok({
        let ui_handle = ui.as_weak();
        move |spec, sub, c, h, s| {
            let ui = ui_handle.unwrap();
            let client = Client::connect(CONNECTION, NoTls);
            if let Ok(mut client) = client {
                let query = format!(
                    "select add_academic_plan('{spec}', '{sub}', '{c}', {h}, {s});"
                );
                if let Err(_) = client.query(&query, &[]) {
                    show_error_window("Ошибка данных").unwrap();
                }
            }
        }
    });

    ui.set_specializaions(
        ModelRc::from( Rc::new(
            VecModel::from( get_specializations()? )
        ) )
    );

    ui.set_subjects(
        ModelRc::from( Rc::new(
            VecModel::from( get_subjects()? )
        ) )
    );

    ui.set_c_types(
        ModelRc::from( Rc::new(
            VecModel::from( get_certification_type()? )
        ) )
    );

    ui.show()?;
    Ok(())
}

fn change_row(id: i32, spec: &str, sub: &str, c: &str, h: i32, s: i32) -> Result<(), Box<dyn Error>> {
    let ui = crate::ChangeAcademicPlan::new()?;

    ui.on_ok({
        let ui_handle = ui.as_weak();
        move |spec, sub, c, h, s| {
            let ui = ui_handle.unwrap();
            let client = Client::connect(CONNECTION, NoTls);
            if let Ok(mut client) = client {
                let query = format!(
                    "select update_academic_plan({id}, '{spec}', '{sub}', '{c}', {h}, {s});"
                );
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
                let query = format!("select delete_academic_plan({id});");
                if let Err(_) = client.query(&query, &[]) {
                    show_error_window("Ошибка данных").unwrap();
                }
            }
        }
    );

    ui.set_c_type_value(c.into());
    ui.set_hours_value(h);
    ui.set_semester_value(s);
    ui.set_specialization_value(spec.into());
    ui.set_subject_value(sub.into());

    ui.set_specializations(
        ModelRc::from( Rc::new(
            VecModel::from( get_specializations()? )
        ) )
    );

    ui.set_subjects(
        ModelRc::from( Rc::new(
            VecModel::from( get_subjects()? )
        ) )
    );

    ui.set_c_types(
        ModelRc::from( Rc::new(
            VecModel::from( get_certification_type()? )
        ) )
    );

    ui.show()?;
    Ok(())
}

pub fn show_plan_for_student(student: &Student) -> Result<(), Box<dyn Error>> {
    let ui = crate::StudentPlanTableWindow::new()?;

    ui.on_refresh({
        let ui_handle = ui.as_weak();
        let student = student.clone();
        move |course: i32| {
            let ui = ui_handle.unwrap();
            ui.set_data(get_data_for_student(&student, course).unwrap());
        }
    });

    ui.on_export_to_excel({
        let ui_handle = ui.as_weak();
        let student = student.clone();
        move |course: i32| {
            let ui = ui_handle.unwrap();
            crate::to_excel::export_student_plan(&student, course);
        }
    });

    ui.set_data(get_data_for_student(student, 1)?);

    ui.show()?;
    Ok(())
}

fn get_data_for_student(student: &Student, course: i32) -> Result<ModelRc<ModelRc<StandardListViewItem>>, postgres::Error> {
    let mut client = Client::connect(CONNECTION, NoTls)?;
    let mut data: Rc<VecModel<ModelRc<StandardListViewItem>>> = 
        Rc::from(VecModel::from(Vec::new()));
    let query = format!(
        "select * from get_student_plan({}) where semester = {} or semester = {} order by semester;",
        student.number,
        course << 1,
        (course << 1) - 1
    );
    for row in client.query(&query, &[]).unwrap() {
        let sub: &str = row.get(0);
        let semester: i64 = row.get(1);
        let hours: i64 = row.get(2);
        let c_type: &str = row.get(3);
        
        let row_data: Rc<VecModel<StandardListViewItem>> = Rc::from(VecModel::from(vec![
            sub.into(),
            semester.to_string().as_str().into(),
            hours.to_string().as_str().into(),
            c_type.into(),            
        ]));
        data.push(ModelRc::from(row_data));
    }
    Ok(ModelRc::from(data))
}