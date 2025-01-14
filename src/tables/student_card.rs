use authorization::Student;
use postgres::{Client, NoTls};
use slint::{Model, ModelRc, SharedString, StandardListViewItem, TableColumn, VecModel};
use tables::specialization;

use crate::*;
use std::{error::Error, rc::Rc};

fn get_groups() -> Result<Vec<SharedString>, postgres::Error> {
    let mut groups: Vec<SharedString> = Vec::new();
    let mut client = Client::connect(CONNECTION, NoTls)?;
    for row in client.query("select name from get_group;", &[])? {
        let cifr: &str = row.get(0);
        groups.push(cifr.into());
    }
    Ok(groups)
}

pub fn show_choose_group() -> Result<(), Box<dyn Error>> {
    let ui = crate::AdminChooseGroup::new()?;

    ui.set_groups(
        ModelRc::from( Rc::new(
            VecModel::from( get_groups()? )
        ) )
    );

    ui.on_ok({
        move |group| {
            if let Some((cifr, year, number)) = crate::tables::group::group_from_str(&group) {
                show_admin_marks(cifr, year, number).unwrap();
            }
        }
    });

    ui.show()?;
    Ok(())
}

fn get_data(cifr: &str, year: i32, number: i32) -> Result<ModelRc<ModelRc<StandardListViewItem>>, postgres::Error> {
    let mut client = Client::connect(CONNECTION, NoTls)?;
    let mut data: Rc<VecModel<ModelRc<StandardListViewItem>>> = 
        Rc::from(VecModel::from(Vec::new()));
    let query = format!("select * from get_student_card_for('{cifr}', {year}, {number})");
    for row in client.query(&query, &[]).unwrap() {
        let id: i32 = row.get(0);
        let f_name: &str = row.get(1);
        let s_name: &str = row.get(2);
        let subject: &str = row.get(3);
        let semester: i64 = row.get(4);
        let c_type: &str = row.get(5);
        let mark: Option<&str> = row.get(6);
        
        let row_data: Rc<VecModel<StandardListViewItem>> = Rc::from(VecModel::from(vec![
            id.to_string().as_str().into(),
            f_name.into(),
            s_name.into(),
            subject.into(),
            semester.to_string().as_str().into(),
            c_type.to_string().as_str().into(),
            mark.unwrap_or("Нет оценки").to_string().as_str().into(),
        ]));
        data.push(ModelRc::from(row_data));
    }
    Ok(ModelRc::from(data))
}

pub fn show_admin_marks(cifr: &str, year: i32, number: i32) -> Result<(), Box<dyn Error>> {
    let ui = crate::AdminMarksTableWindow::new()?;

    ui.on_export_to_excel({
        let cifr = cifr.to_string();
        move || {
            crate::to_excel::export_admin_marks(&cifr, year, number);
        }
    });

    ui.on_refresh({
        let ui_handle = ui.as_weak();
        let cifr = cifr.to_string();
        move || {
            let ui = ui_handle.unwrap();
            ui.set_data(get_data(&cifr, year, number).unwrap());
        }
    });

    ui.set_data(get_data(cifr, year, number)?);

    ui.show()?;
    Ok(())
}

pub fn show_marks_for_student(student: &Student) -> Result<(), Box<dyn Error>> {
    let ui = crate::StudentMarksTableWindow::new()?;

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
            let ui = ui_handle.unwrap();
            crate::to_excel::export_student_marks(&student);
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
        "SELECT * FROM get_marks_student({}) order by semester;",
        student.number,
    );
    for row in client.query(&query, &[]).unwrap() {
        let sub: &str = row.get(0);
        let semester: i64 = row.get(1);
        let hours: i64 = row.get(2);
        let c_type: &str = row.get(3);
        let mark: Option<&str> = row.get(4);
        
        let row_data: Rc<VecModel<StandardListViewItem>> = Rc::from(VecModel::from(vec![
            sub.into(),
            semester.to_string().as_str().into(),
            hours.to_string().as_str().into(),
            c_type.into(),            
            mark.unwrap_or("Нет оценки").into(),            
        ]));
        data.push(ModelRc::from(row_data));
    }
    Ok(ModelRc::from(data))
}