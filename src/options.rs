#[derive(Debug, Clone)]
pub struct PageOptions{
    pub title: String,
    pub menu_box_title: String,
    pub selected_box_title: String,
    pub list_box_title: String,
    pub save_command_description: String,
    pub disable_delete: bool,
    pub disable_add: bool,
    pub disable_edit: bool,
    pub disable_save: bool,
}

impl PageOptions{
    pub fn new(title: String) -> PageOptions {
        PageOptions{
            title: title,
            menu_box_title: String::from("Menu"),
            selected_box_title: String::from("Selected"),
            list_box_title: String::from("List"),
            save_command_description: String::from("save"),
            disable_delete: false,
            disable_add: false,
            disable_edit: false,
            disable_save: false,
        }
    }
}

pub struct Options{
    pub page_options: Vec<PageOptions>,
}

impl Options{
    pub fn new() -> Options {
        Options{
            page_options: Vec::new(),
        }
    }
}
