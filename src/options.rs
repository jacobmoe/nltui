#[derive(Debug, Clone)]
pub struct PageOptions{
    pub title: String,
    pub menu_box_title: String,
    pub selected_box_title: String,
    pub list_box_title: String,
    pub body_box_title: String,
    pub disable_delete: bool,
    pub disable_add: bool,
    pub disable_edit: bool,
}

impl PageOptions{
    pub fn new(title: String) -> PageOptions {
        PageOptions{
            title: title,
            menu_box_title: String::from("Menu"),
            selected_box_title: String::from("Selected"),
            list_box_title: String::from("List"),
            body_box_title: String::from("Body"),
            disable_delete: false,
            disable_add: false,
            disable_edit: false,
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
