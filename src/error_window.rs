use std::error::Error;
use super::*;

pub(super) fn show_error_window(msg: &str) -> Result<(), Box<dyn Error>> {
    let ui = super::ErrorWindow::new()?;

    ui.set_message(msg.into());

    ui.show()?;

    Ok(())
}