pub mod group;

use crate::*;
use std::error::Error;

pub fn show_group() -> Result<(), Box<dyn Error>> {
    let ui = crate::AdminMainMenu::new()?;
    ui.on_groups({
        move || {
            group::show_group_table().unwrap();
        }
    });
    ui.show()?;
    Ok(())
}