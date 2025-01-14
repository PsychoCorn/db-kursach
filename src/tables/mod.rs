pub mod group;
pub mod specialization;
pub mod student;
pub mod subject;
pub mod academic_plan;
pub mod student_card;
pub mod accounts;
pub mod teacher;

use academic_plan::show_plan_for_student;
use group::show_group_for_student;
use postgres::{Client, NoTls};
use slint::{ModelRc, SharedString, VecModel};
use student_card::show_marks_for_student;

use crate::*;
use std::{error::Error, rc::Rc};

pub fn show_admin_menu() -> Result<(), Box<dyn Error>> {
    let ui = crate::AdminMainMenu::new()?;

    ui.on_specializaions({
        move || {
            specialization::show_full_table().unwrap();
        }
    });

    ui.on_groups({
        move || {
            group::show_full_table().unwrap();
        }
    });

    ui.on_students({
        move || {
            student::show_full_table().unwrap();
        }
    });

    ui.on_subjects({
        move || {
            subject::show_full_table().unwrap();
        }
    });

    ui.on_academic_plan({
        move || {
            academic_plan::show_full_table().unwrap();
        }
    });

    ui.on_students_cards({
        move || {
            student_card::show_choose_group().unwrap();
        }
    });

    ui.on_student_accounts({
        move || {
            accounts::student::show_full_table().unwrap();
        }
    });

    ui.on_teacher_accounts({
        move || {
            accounts::teachers::show_full_table().unwrap();
        }
    });

    ui.show()?;
    Ok(())
}

pub fn show_student_menu(student: authorization::Student) -> Result<(), Box<dyn Error>> {
    let ui = crate::StudentMainMenu::new()?;

    ui.set_info(format!("{student}").into());

    ui.on_group({
        let student = student.clone();
        let ui_handle = ui.as_weak();
        move || {
            let ui = ui_handle.unwrap();
            show_group_for_student(&student).unwrap()
        }
    });

    ui.on_plan({
        let ui_handle = ui.as_weak();
        let student = student.clone();
        move || {
            let ui = ui_handle.unwrap();
            show_plan_for_student(&student).unwrap()
        }
    });

    ui.on_marks({
        let ui_handle = ui.as_weak();
        let student = student.clone();
        move || {
            let ui = ui_handle.unwrap();
            show_marks_for_student(&student).unwrap()
        }
    });

    ui.show()?;
    Ok(())
}

pub fn show_teacher_menu(teacher: authorization::Teacher) -> Result<(), Box<dyn Error>> {
    let ui = crate::TeacherMainMenu::new()?;


    ui.set_subjects(
        ModelRc::from( Rc::new(
            VecModel::from( teacher.subjects.clone() )
        ) )
    );

    ui.on_update_groups({
        let ui_handle = ui.as_weak();
        move |subject| {
            let ui = ui_handle.unwrap();
            ui.set_groups(
                ModelRc::from( Rc::new(
                    VecModel::from( get_groups(
                        get_spec_name(&subject).unwrap()
                    ).unwrap() )
                ) )
            );
        }
    });

    ui.set_groups(
        ModelRc::from( Rc::new(
            VecModel::from( get_groups(
                get_spec_name(&teacher.subjects[0]).unwrap()
            )? )
        ) )
    );

    ui.on_show_group({
        let ui_handle = ui.as_weak();
        move |subject, group| {
            let ui = ui_handle.unwrap();
            let plan_id: i32 = get_plan_id(&subject).unwrap();
            if let Some((cifr, year, number)) = crate::tables::group::group_from_str(&group) {
                teacher::show_full_table(plan_id, cifr, year, number).unwrap();
            } else {
                show_error_window("Ошибка данных").unwrap();
            }
        }
    });

    ui.show()?;
    Ok(())
}

fn get_spec_name(subject: &str) -> Option<&str> {
    let mut iter = subject.split("; ");
    Some(iter.nth(1).unwrap())
}

fn get_groups(spec_name: &str) -> Result<Vec<SharedString>, postgres::Error> {
    let mut groups: Vec<SharedString> = Vec::new();
    let mut client = Client::connect(CONNECTION, NoTls)?;
    let query = format!("select * from get_groups_by_name('{spec_name}');");
    for row in client.query(&query, &[])? {
        let cifr: &str = row.get(0);
        let year: i64 = row.get(1);
        let number: i64 = row.get(2);
        groups.push(format!("{cifr}-{year}-{number}").into());
    }
    Ok(groups)
}

fn get_plan_id(subject: &str) -> Option<i32> {
    let mut iter = subject.split("; ");
    Some(iter.nth(0).unwrap().parse().unwrap())
}