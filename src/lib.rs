mod util;
mod list;
mod app;
pub mod options;

use crate::app::{App};
pub use crate::options::{PageOptions};
use crate::list::{List as InternList, Item as InternItem};

pub struct List {
    pub name: String,
    pub items: Vec<Item>,
}

impl List {
    pub fn new(name: String, items: Vec<Item>) -> List {
        List{
            name: name,
            items: items,
        }
    }
}

pub struct Item {
    pub id: String,
    pub name: String,
    pub list: Option<List>,
}

impl Item {
    pub fn new(id: String, name: String, list: Option<List>) -> Item {
        Item{
            id: id,
            name: name,
            list: list,
        }
    }
}

pub fn run(list: List, page_options: Vec<PageOptions>) -> Result<(), failure::Error> {
    let root = InternList::new(list.name);
    let mut app = App::new(root);

    for item in items_from_supplied(&mut app, 0, &list.items) {
        app.lists[0].items.push(item);
    }

    if app.lists[0].items.len() > 0 {
        app.lists[0].selected = Some(0);
    }

    app.set_page_options(page_options);
    app::run(app)
}

fn items_from_supplied(app: &mut App, current_list_index: usize, supplied_items: &Vec<Item>) -> Vec<InternItem> {
    if supplied_items.len() == 0 {
        return Vec::new();
    }

    supplied_items.iter().map(|supplied_item| {
        let mut item = InternItem::new(
            supplied_item.id.clone(),
            supplied_item.name.clone(),
        );

        match &supplied_item.list {
            Some(next_supplied_list) => {
                let list_index = app.add_list(next_supplied_list.name.clone());
                app.lists[list_index].previous = Some(current_list_index);

                item.list_index = Some(list_index);

                for next_item in items_from_supplied(app, list_index, &next_supplied_list.items) {
                    app.lists[list_index].items.push(next_item);
                }

                if app.lists[list_index].items.len() > 0 {
                    app.lists[list_index].selected = Some(0);
                }
            }
            None => {}
        }

        item
    }).collect()
}
