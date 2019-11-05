mod util;

use std::io::{self, Write};

use termion::cursor::Goto;
use termion::event::Key;
use termion::input::MouseTerminal;
use termion::raw::IntoRawMode;
use termion::screen::AlternateScreen;
use tui::backend::TermionBackend;
use tui::layout::{Alignment, Constraint, Corner, Direction, Layout};
use tui::style::{Color, Modifier, Style};
use tui::widgets::{Block, Borders, List as TuiList, SelectableList, Paragraph, Text, Widget};
use tui::Terminal;
use unicode_width::UnicodeWidthStr;

use crate::util::event::{Event, Events};

type Term = Terminal<TermionBackend<AlternateScreen<MouseTerminal<termion::raw::RawTerminal<std::io::Stdout>>>>>;

#[derive(Debug, Clone)]
struct PageOptions{
    title: String,
    menu_box_title: String,
    selected_box_title: String,
    list_box_title: String,
    disable_delete: bool,
    disable_add: bool,
    disable_edit: bool,
}

impl PageOptions{
    pub fn new(title: String) -> PageOptions {
        PageOptions{
            title: title,
            menu_box_title: String::from("Menu"),
            selected_box_title: String::from("Selected"),
            list_box_title: String::from("List"),
            disable_delete: false,
            disable_add: false,
            disable_edit: false,
        }
    }
}

struct Options{
    page_options: Vec<PageOptions>,
}

impl Options{
    pub fn new() -> Options {
        Options{
            page_options: Vec::new(),
        }
    }
}

#[derive(Debug, Clone)]
struct List{
    name: String,
    items: Vec<Item>,
    selected: Option<usize>,
    previous: Option<usize>,
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
}

#[derive(Debug, Clone)]
struct Item {
    id: String,
    name: String,
    list_index: Option<usize>,
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

struct App{
    lists: Vec<List>,
    current: usize,
    options: Options,
    depth: usize,
}

impl App{
    pub fn new(root_list: List) -> App {
        App{
            lists: vec!(root_list),
            current: 0,
            options: Options::new(),
            depth: 0,
        }
    }

    pub fn get_current_page_options(&self) -> PageOptions {
        if self.options.page_options.len()-1 >= self.depth {
            self.options.page_options[self.depth].clone()
        } else {
            PageOptions::new(format!("{}", self.depth))
        }
    }

    pub fn get_current_list(&self) -> List {
        self.lists[self.current].clone()
    }

    pub fn get_selected_item(&self) -> Option<Item> {
        match self.lists[self.current].selected {
            Some(selected) => {
                return Some(self.lists[self.current].items[selected].clone());
            }
            None => { return None; }
        }
    }

    pub fn get_list_for_selected(&self) -> Option<List> {
        match self.lists[self.current].selected {
            Some(selected) => {
                match self.lists[self.current].items[selected].list_index {
                    Some(selected_item_list_index) => {
                        return Some(self.lists[selected_item_list_index].clone());
                    }
                    None => { return None; }

                }
            }
            None => { return None; }
        }
    }

    pub fn add_list(&mut self, name: String) -> usize {
        let mut list = List::new(name);
        list.previous = Some(self.current);
        self.lists.push(list);

        return self.lists.len() - 1;
    }

    pub fn add_list_item(&mut self, name: String, id: String) {
        let item = Item::new(id, name.clone());

        match self.lists[self.current].selected {
            Some(selected_item_index) => {
                match self.lists[self.current].items[selected_item_index].list_index {
                    Some(_) => {}
                    None => {
                        let index = self.add_list(name.clone());
                        self.lists[self.current].items[selected_item_index].list_index = Some(index);
                    }
                }

                match self.lists[self.current].items[selected_item_index].list_index {
                    Some(list_index) => {
                        self.lists[list_index].items.push(item);
                    }
                    None => {}
                }
            }
            None => {}
        }
    }

    pub fn set_item_selection(&mut self, selected: Option<usize>) {
        self.lists[self.current].selected = selected;
    }

    pub fn close_current_list(&mut self) {
        match self.lists[self.current].previous {
            Some(previous_index) => {
                self.current = previous_index;
                self.depth = self.depth - 1;

                // if list that was just closed doesn't have any items,
                // remove list_index from item that owns the list
                match self.lists[self.current].selected {
                    Some(selected) => {
                        match self.lists[self.current].items[selected].list_index {
                            Some(index) => {
                                if self.lists[index].items.len() == 0 {
                                    self.lists[self.current].items[selected].list_index = None;
                                }
                            }
                            None => {}
                        }
                    }
                    None => {}
                }
            }
            None => {}
        }
    }

    pub fn open_selected_item_list(&mut self) {
        if !self.get_current_page_options().disable_edit {
            match self.lists[self.current].selected {
                Some(selected_item_index) => {
                    match self.lists[self.current].items[selected_item_index].list_index {
                        Some(index) => {
                            self.current = index;
                            self.depth = self.depth + 1;

                            if self.lists[index].items.len() > 0 {
                                self.lists[index].selected = Some(0);
                            }
                        }
                        None => {}
                    }
                }
                None => {}
            }
        }
    }

    pub fn delete_selected_item(&mut self) {
        if !self.get_current_page_options().disable_delete {
            match self.lists[self.current].selected {
                Some(index) => {
                    self.lists[self.current].items[index].list_index = None;
                    self.lists[self.current].items.remove(index);

                    if self.lists[self.current].items.len() > 0 {
                        self.lists[self.current].selected = Some(0);
                    } else {
                        self.lists[self.current].selected = None;
                        self.close_current_list();
                    }
                }
                None => {}
            }
        }
    }
}

fn main() -> Result<(), failure::Error> {
    let items = vec![
        Item::new(String::from("item1id"), String::from("item1name")),
        Item::new(String::from("item2id"), String::from("item2name")),
        Item::new(String::from("item3id"), String::from("item3name")),
        Item::new(String::from("item4id"), String::from("item4name")),
        Item::new(String::from("item5id"), String::from("item5name")),
    ];

    let mut list = List::new(String::from("list name"));
    list.items = items;
    list.selected = Some(0);

    let mut app = App::new(list);

    let mut page_options = vec![
        PageOptions::new(String::from("Example1")),
        PageOptions::new(String::from("Example2")),
        PageOptions::new(String::from("Example3")),
    ];
    page_options[0].disable_delete = true;
    page_options[2].disable_add = true;

    app.options.page_options = page_options;

    run(app)
}

fn run(mut app: App) -> Result<(), failure::Error> {
    if app.lists.len() == 0 {
        println!("No root list found");
        return Ok(());
    }

    if app.lists[0].items.len() == 0 {
        println!("No items in root list");
        return Ok(());
    }

    // Terminal initialization
    let stdout = io::stdout().into_raw_mode()?;
    let stdout = MouseTerminal::from(stdout);
    let stdout = AlternateScreen::from(stdout);
    let backend = TermionBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;
    terminal.hide_cursor()?;

    let events = Events::new();

    'main: loop {
        let page_options = app.get_current_page_options();

        terminal.draw(|mut f| {
            let wrapper_chunks = Layout::default()
                .direction(Direction::Vertical)
                .constraints([
                    Constraint::Percentage(10),
                    Constraint::Percentage(90),
                ].as_ref())
                .split(f.size());

            let block = Block::default()
                .borders(Borders::ALL)
                .style(Style::default().bg(Color::Black));

            let list = app.get_current_list();
            let title = format!("{}: {}", page_options.title, list.name);

            Paragraph::new([
                Text::styled(
                    title,
                    Style::default().fg(Color::Red).modifier(Modifier::BOLD),
                )].iter())
                .block(block.clone())
                .alignment(Alignment::Center)
                .render(&mut f, wrapper_chunks[0]);

            let primary_chunks = Layout::default()
                .direction(Direction::Horizontal)
                .constraints([
                    Constraint::Percentage(30),
                    Constraint::Percentage(70),
                ].as_ref())
                .split(wrapper_chunks[1]);

            let info_chunks = Layout::default()
                .direction(Direction::Vertical)
                .constraints([
                    Constraint::Percentage(20),
                    Constraint::Percentage(20),
                    Constraint::Percentage(60),
                ].as_ref())
                .split(primary_chunks[1]);

            let style = Style::default().fg(Color::Black).bg(Color::White);
            SelectableList::default()
                .block(Block::default().borders(Borders::ALL).title(page_options.menu_box_title.as_str()))
                .items(&list.items.iter().map(|i| { i.name.clone() }).collect::<Vec<_>>())
                .select(list.selected)
                .style(style)
                .highlight_style(style.fg(Color::LightGreen).modifier(Modifier::BOLD))
                .highlight_symbol("=>")
                .render(&mut f, primary_chunks[0]);

            match app.get_selected_item() {
                Some(item) => {
                    let mut usage = vec![
                        "ctrl-c: exit",
                        "b: back to previous page",
                    ];

                    if !page_options.disable_add {
                        usage.push("a: add items to selection");
                    }

                    if !page_options.disable_edit {
                        usage.push("e: edit selection");
                    }

                    if !page_options.disable_delete {
                        usage.push("d: delete selection");
                    }

                    let usage_info = usage.iter().map(|i| {
                        Text::styled(
                            format!("{}", i),
                            Style::default().fg(Color::Green),
                        )
                    });

                    TuiList::new(usage_info)
                        .block(Block::default().borders(Borders::ALL).title("Commands"))
                        .start_corner(Corner::TopLeft)
                        .render(&mut f, info_chunks[0]);

                    let fields = vec![
                        format!("ID: {}", item.id),
                        format!("Name: {}", item.name),
                    ];

                    let item_info = fields.iter().map(|i| {
                        Text::styled(
                            format!("{}", i),
                            Style::default().fg(Color::White),
                        )
                    });

                    TuiList::new(item_info)
                        .block(Block::default().borders(Borders::ALL).title(page_options.selected_box_title.as_str()))
                        .start_corner(Corner::TopLeft)
                        .render(&mut f, info_chunks[1]);

                    match app.get_list_for_selected() {
                        Some(nested_list) => {
                            let item_list = nested_list.items.iter().map(|i| {
                                Text::styled(
                                    format!("{}", i.name),
                                    Style::default().fg(Color::Yellow),
                                )
                            });

                            TuiList::new(item_list)
                                .block(Block::default().borders(Borders::ALL).title(page_options.list_box_title.as_str()))
                                .start_corner(Corner::TopLeft)
                                .render(&mut f, info_chunks[2]);
                        }
                        None => {}
                    }
                }
                None => {}
            }
        })?;

        let mut list = app.get_current_list();

        match events.next()? {
            Event::Input(input) => match input {
                Key::Ctrl('c') => {
                    break;
                }
                Key::Char('b') => {
                    app.close_current_list();
                }
                Key::Left => {
                    list.selected = None;
                }
                Key::Down => {
                    let s = if let Some(selected) = list.selected {
                        if selected >= list.items.len() - 1 {
                            Some(0)
                        } else {
                            Some(selected + 1)
                        }
                    } else {
                        Some(0)
                    };

                    app.set_item_selection(s);
                }
                Key::Up => {
                    let s = if let Some(selected) = list.selected {
                        if selected > 0 {
                            Some(selected - 1)
                        } else {
                            Some(list.items.len() - 1)
                        }
                    } else {
                        Some(0)
                    };

                    app.set_item_selection(s);
                }
                Key::Char('a') => {
                    if !page_options.disable_add {
                        let mut user_input: String = String::new();

                        'input: loop {
                            draw_add_menu(&mut terminal, &app, user_input.clone())?;

                            // Handle input
                            match events.next()? {
                                Event::Input(input) => match input {
                                    Key::Ctrl('c') => {
                                        break 'main;
                                    }
                                    Key::Ctrl('s') => {
                                        break;
                                    }
                                    Key::Char('\n') => {
                                        if user_input != "" {
                                            let input: String = user_input.drain(..).collect();
                                            let id = input.clone();
                                            let name = input.clone();

                                            app.add_list_item(id, name);
                                        }
                                    }
                                    Key::Char(c) => {
                                        user_input.push(c);
                                    }
                                    Key::Backspace => {
                                        user_input.pop();
                                    }
                                    _ => {}
                                },
                            }
                        };
                    }
                }
                Key::Char('e') => {
                    app.open_selected_item_list();
                }
                Key::Char('d') => {
                    app.delete_selected_item();
                }
                _ => {}
            },
        }
    }
    Ok(())
}

fn draw_add_menu(terminal: &mut Term, app: &App, user_input: String) -> Result<(), failure::Error> {
    let page_options = app.get_current_page_options();

    terminal.draw(|mut f| {
        let wrapper_chunks = Layout::default()
            .direction(Direction::Vertical)
            .margin(2)
            .constraints([
                Constraint::Percentage(10),
                Constraint::Percentage(20),
                Constraint::Percentage(70),
            ].as_ref())
            .split(f.size());

        let block = Block::default()
            .borders(Borders::ALL)
            .style(Style::default().bg(Color::Black));

        let title: String;
        match app.get_selected_item() {
            Some(item) => {
                title = format!("{}", item.name);
            }
            None => {
                let list = app.get_current_list();
                title = format!("{}", list.name);
            }
        }

        Paragraph::new([
            Text::styled(
                title,
                Style::default().fg(Color::Red).modifier(Modifier::BOLD),
            )].iter())
            .block(block.clone())
            .alignment(Alignment::Center)
            .render(&mut f, wrapper_chunks[0]);

        let usage = vec![
            "ctrl-s: save and return to previous window",
        ];

        let usage_info = usage.iter().map(|i| {
            Text::styled(
                format!("{}", i),
                Style::default().fg(Color::Green),
            )
        });

        TuiList::new(usage_info)
            .block(Block::default().borders(Borders::ALL).title("Commands"))
            .start_corner(Corner::TopLeft)
            .render(&mut f, wrapper_chunks[1]);

        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([Constraint::Length(3), Constraint::Min(1)].as_ref())
            .split(wrapper_chunks[2]);

        Paragraph::new([Text::raw(&user_input)].iter())
            .style(Style::default().fg(Color::Yellow))
            .block(Block::default().borders(Borders::ALL).title("Input"))
            .render(&mut f, chunks[0]);

        match app.get_list_for_selected() {
            Some(list) => {
                let text_list = list
                    .items
                    .iter()
                    .enumerate()
                    .map(|(i, m)| Text::raw(format!("{}: {}", i, m.name)));
                TuiList::new(text_list)
                    .block(Block::default().borders(Borders::ALL).title(page_options.list_box_title.as_str()))
                    .render(&mut f, chunks[1]);
            }
            None => {}
        }
    })?;

    write!(
        terminal.backend_mut(),
        "{}",
        Goto(4 + user_input.width() as u16, 4)
    )?;
    // stdout is buffered, flush it to see the effect immediately when hitting backspace
    io::stdout().flush().ok();
    Ok(())
}
