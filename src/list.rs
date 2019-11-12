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

    pub fn get_selected_item_index(&self) -> Option<usize> {
        return self.selected;
    }

    pub fn set_selected_item_index(&mut self, index: Option<usize>) {
        self.selected = index;
    }

    pub fn set_selected_item_list_index(&mut self, list_index: Option<usize>) {
        match self.selected {
            Some(selected_item_index) => {
                self.items[selected_item_index].list_index = list_index;
            }
            None => {}
        }
    }

    pub fn remove_selected_item(&mut self) {
        match self.selected {
            Some(selected) => {
                self.items.remove(selected);

                if self.items.len() > 0 {
                    self.selected = Some(0);
                } else {
                    self.selected = None;
                }
            }
            None => {}
        }
    }

    pub fn increment_selected(&mut self) {
        if self.items.len() == 0 {
            return
        }

        let s = if let Some(selected) = self.selected {
            if selected > 0 {
                Some(selected - 1)
            } else {
                Some(self.items.len() - 1)
            }
        } else {
            Some(0)
        };

        self.set_selected_item_index(s);
    }

    pub fn decrement_selected(&mut self) {
        if self.items.len() == 0 {
            return
        }

        let s = if let Some(selected) = self.selected {
            if selected >= self.items.len() - 1 {
                Some(0)
            } else {
                Some(selected + 1)
            }
        } else {
            Some(0)
        };

        self.set_selected_item_index(s);
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
