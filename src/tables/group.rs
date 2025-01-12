use postgres::{Client, NoTls};
use slint::{ModelRc, SharedString, StandardListViewItem, TableColumn, VecModel};

use crate::*;
use std::{error::Error, rc::Rc};

fn get_colums() -> ModelRc<TableColumn> {
    let mut id = TableColumn::default();
    let mut name = TableColumn::default();
    let mut specialization = TableColumn::default();
    id.title = "ID".into();
    name.title = "Название".into();
    specialization.title = "Специальность".into();
    let colums = Rc::new(VecModel::from(vec![
        id,
        name,
        specialization,
    ]));
    ModelRc::from(colums)
}

fn get_data() -> Result<ModelRc<ModelRc<StandardListViewItem>>, postgres::Error> {
    let mut client = Client::connect(CONNECTION, NoTls)?;
    let mut data: Rc<VecModel<ModelRc<StandardListViewItem>>> = 
        Rc::from(VecModel::from(Vec::new()));
    for row in client.query("select * from get_group;", &[])? {
        let id: i32 = row.get(0);
        let name: &str = row.get(1);
        let specialization: &str = row.get(2);
        
        let row_data: Rc<VecModel<StandardListViewItem>> = Rc::from(VecModel::from(vec![
            id.to_string().as_str().into(),
            name.into(),
            specialization.into()
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
            crate::to_excel::table_to_excel(crate::to_excel::export_group);
        }
    );

    ui.set_window_title("Группы".into());
    ui.set_columns(get_colums());
    ui.set_data(get_data()?);

    ui.show()?;
    Ok(())
}