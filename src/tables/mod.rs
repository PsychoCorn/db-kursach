pub mod group;
pub mod specialization;

use crate::*;
use std::error::Error;

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
    ui.show()?;
    Ok(())
}

pub fn show_student_menu(student: authorization::Student) -> Result<(), Box<dyn Error>> {
    let ui = crate::StudentMainMenu::new()?;

    ui.set_info(format!("{student}").into());

    ui.show()?;
    Ok(())
}