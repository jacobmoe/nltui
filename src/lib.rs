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

    for item in items_from_user(&mut app, 0, &list.items) {
        app.lists[0].items.push(item);
    }

    if app.lists[0].items.len() > 0 {
        app.lists[0].selected = Some(0);
    }

    app.set_page_options(page_options);
    app::run(app)
}

fn items_from_user(app: &mut App, current_list_index: usize, user_items: &Vec<Item>) -> Vec<InternItem> {
    if user_items.len() == 0 {
        return Vec::new();
    }

    user_items.iter().map(|user_item| {
        let mut item = InternItem::new(
            user_item.id.clone(),
            user_item.name.clone(),
        );

        match &user_item.list {
            Some(next_user_list) => {
                let list_index = app.add_list(next_user_list.name.clone());
                app.lists[list_index].previous = Some(current_list_index);

                item.list_index = Some(list_index);

                for next_item in items_from_user(app, list_index, &next_user_list.items) {
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

fn items_to_user(app: &App, items: &Vec<InternItem>) -> Vec<Item> {
    if items.len() == 0 {
        return Vec::new();
    }

    return items.iter().map(|item| {
        let mut user_item = Item::new(
            item.id.clone(),
            item.name.clone(),
            None,
        );

        match item.list_index {
            Some(index) => {
                let mut next_user_list = List::new(
                    app.lists[index].name.clone(),
                    Vec::new(),
                );

                for next_user_item in items_to_user(app, &app.lists[index].items) {
                    next_user_list.items.push(next_user_item);
                }

                user_item.list = Some(next_user_list);
            }
            None => {}
        }

        user_item
    }).collect();
}
