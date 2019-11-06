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

    for item in from_supplied_items(&mut app, &list.items) {
        app.lists[0].items.push(item);
    }

    if app.lists[0].items.len() > 0 {
        app.lists[0].selected = Some(0);
    }

    app.set_page_options(page_options);
    app::run(app)
}

fn from_supplied_items(app: &mut App, items: &Vec<Item>) -> Vec<InternItem> {
    if items.len() == 0 {
        return Vec::new();
    }

    items.iter().map(|i| {
        let mut item = InternItem::new(i.id.clone(), i.name.clone());
        match &i.list {
            Some(list) => {
                let list_index = app.add_list(list.name.clone());
                item.list_index = Some(list_index);
                let items = &list.items;

                for next_item in from_supplied_items(app, items) {
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
