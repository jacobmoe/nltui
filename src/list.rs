#[derive(Debug, Clone)]
pub struct List{
    pub name: String,
    pub items: Vec<Item>,
    pub selected: Option<usize>,
    pub previous: Option<usize>,
}

impl List{
    pub fn new(name: String) -> List {
        List{
            name: name,
            items: Vec::new(),
            selected: None,
            previous: None,
        }
    }

    pub fn get_selected_item(&self) -> Option<&Item> {
        match self.selected {
            Some(selected) => {
                return Some(&self.items[selected]);
            }
            None => { return None; }
        }
    }
}

#[derive(Debug, Clone)]
pub struct Item {
    pub id: String,
    pub name: String,
    pub list_index: Option<usize>,
}

impl Item{
    pub fn new(id: String, name: String) -> Item {
        Item{
            id: id,
            name: name.clone(),
            list_index: None,
        }
    }
}
