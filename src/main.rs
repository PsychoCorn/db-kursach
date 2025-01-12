// Prevent console window in addition to Slint window in Windows release builds when, e.g., starting the app via file manager. Ignored on other platforms.
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use std::error::Error;

use authorization::User;
use error_window::show_error_window;

include!("secrets.rs");

mod authorization;
mod error_window;
mod tables;
mod to_excel;

slint::include_modules!();


fn main() -> Result<(), Box<dyn Error>> {
    let ui = AuthorizationWindow::new()?;

    ui.on_authorization({
        let ui_handle = ui.as_weak();
        move |login: slint::SharedString, password: slint::SharedString| {  // Принимаем значения логина и пароля
            let ui = ui_handle.unwrap();

            match authorization::check_user(&login, &password) {
                Err(err) => {
                    let error = err.code().unwrap();
                    show_error_window(error.code()).unwrap();
                },
                Ok(user) => {
                    match user {
                        User::UnknownUser => { show_error_window("Не верные логин или пароль") },
                        User::Admin => {
                            tables::show_admin_menu()
                        }
                        User::Student(student) => {
                            tables::show_student_menu(student)
                        }
                        _ => { println!("ok: {user:?}"); Ok(()) }
                    }.unwrap();
                }
            }
        
        }
    });

    ui.run()?;

    Ok(())
}
