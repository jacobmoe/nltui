extern crate nltui;

fn main() -> Result<(), failure::Error> {
    let list = nltui::List{
        name: String::from("first list"),
        items: vec![
            nltui::Item{
                id: String::from("item 1 for first list id"),
                name: String::from("item 1 for first list name"),
                list: Some(nltui::List{
                    name: String::from("second list"),
                    items: vec![
                        nltui::Item{
                            id: String::from("item 1 for second list id"),
                            name: String::from("item 1 for second list name"),
                            list: Some(nltui::List{
                                name: String::from("third list"),
                                items: vec![
                                    nltui::Item{
                                        id: String::from("item 1 for third list id"),
                                        name: String::from("item 1 for third list name"),
                                        list: None,
                                    },
                                ],
                            }),
                        },
                    ],
                }),
            },
            nltui::Item{
                id: String::from("item 2 for first list id"),
                name: String::from("item 2 for first list name"),
                list: None,
            },
        ]
    };

    // let mut page_options = vec![
    //     nltui::PageOptions::new(String::from("Example1")),
    //     nltui::PageOptions::new(String::from("Example2")),
    //     nltui::PageOptions::new(String::from("Example3")),
    // ];

    // page_options[0].disable_delete = true;
    // page_options[2].disable_add = true;

    let mut ui = nltui::UI::new(list);
    ui.on_save(Box::new(|list: nltui::List| {
        println!("=======> {:?}", list);
    }));

    ui.run()
}
