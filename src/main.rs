extern crate nltui;

fn main() -> Result<(), failure::Error> {
    let nested_items = vec![
        nltui::Item::new(String::from("nested item 1 id"), String::from("nested item 1 name"), None),
        nltui::Item::new(String::from("nested item 2 id"), String::from("nested item 2 name"), None),
        nltui::Item::new(String::from("nested item 3 id"), String::from("nested item 3 name"), None),
        nltui::Item::new(String::from("nested item 4 id"), String::from("nested item 4 name"), None),
        nltui::Item::new(String::from("nested item 5 id"), String::from("nested item 5 name"), None),
    ];

    let nested_list = nltui::List::new(String::from("second list"), nested_items);

    let items = vec![
        nltui::Item::new(String::from("item 1 id"), String::from("item 1 name"), Some(nested_list)),
        nltui::Item::new(String::from("item 2 id"), String::from("item 2 name"), None),
        nltui::Item::new(String::from("item 3 id"), String::from("item 3 name"), None),
        nltui::Item::new(String::from("item 4 id"), String::from("item 4 name"), None),
        nltui::Item::new(String::from("item 5 id"), String::from("item 5 name"), None),
    ];

    let list = nltui::List::new(String::from("first list"), items);

    let mut page_options = vec![
        nltui::PageOptions::new(String::from("Example1")),
        nltui::PageOptions::new(String::from("Example2")),
        nltui::PageOptions::new(String::from("Example3")),
    ];
    page_options[0].disable_delete = true;
    page_options[2].disable_add = true;

    nltui::run(list, page_options)
}
