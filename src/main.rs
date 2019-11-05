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

struct Options{
    restrict_delete: Vec<usize>,
}

impl Options{
    pub fn new() -> Options {
        Options{
            restrict_delete: Vec::new(),
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
}

impl App{
    pub fn new(root_list: List) -> App {
        App{
            lists: vec!(root_list),
            current: 0,
            options: Options::new(),
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
            }
            None => {}
        }
    }

    pub fn open_selected_item_list(&mut self) {
        match self.lists[self.current].selected {
            Some(selected_item_index) => {
                match self.lists[self.current].items[selected_item_index].list_index {
                    Some(index) => {
                        self.current = index;
                    }
                    None => {}
                }
            }
            None => {}
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
    app.options.restrict_delete = vec!(0);

    run(app)
}

fn run(mut app: App) -> Result<(), failure::Error> {
    // Terminal initialization
    let stdout = io::stdout().into_raw_mode()?;
    let stdout = MouseTerminal::from(stdout);
    let stdout = AlternateScreen::from(stdout);
    let backend = TermionBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;
    terminal.hide_cursor()?;

    let events = Events::new();

    'main: loop {
        terminal.draw(|mut f| {
            let wrapper_chunks = Layout::default()
                .direction(Direction::Vertical)
                .constraints([
                    Constraint::Percentage(5),
                    Constraint::Percentage(95),
                ].as_ref())
                .split(f.size());

            let block = Block::default()
                .borders(Borders::LEFT | Borders::RIGHT)
                .border_style(Style::default().fg(Color::White))
                .style(Style::default().bg(Color::Black));

            let list = app.get_current_list();

            let title = format!("{}: {}", app.current, list.name);
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
                    Constraint::Percentage(80),
                ].as_ref())
                .split(primary_chunks[1]);

            let style = Style::default().fg(Color::Black).bg(Color::White);
            SelectableList::default()
                .block(Block::default().borders(Borders::ALL).title("Menu"))
                .items(&list.items.iter().map(|i| { i.name.clone() }).collect::<Vec<_>>())
                .select(list.selected)
                .style(style)
                .highlight_style(style.fg(Color::LightGreen).modifier(Modifier::BOLD))
                .highlight_symbol("=>")
                .render(&mut f, primary_chunks[0]);

            match app.get_selected_item() {
                Some(item) => {
                    let fields = vec![
                        format!("Item ID: {}", item.id),
                        format!("Item Name: {}", item.name),
                    ];

                    let item_info = fields.iter().map(|i| {
                        Text::styled(
                            format!("{}", i),
                            Style::default().fg(Color::White),
                        )
                    });

                    TuiList::new(item_info)
                        .block(Block::default().borders(Borders::ALL).title("Selected"))
                        .start_corner(Corner::TopLeft)
                        .render(&mut f, info_chunks[0]);

                    match app.get_list_for_selected() {
                        Some(nested_list) => {
                            let item_list = nested_list.items.iter().map(|i| {
                                Text::styled(
                                    format!("{}", i.name),
                                    Style::default().fg(Color::Yellow),
                                )
                            });

                            TuiList::new(item_list)
                                .block(Block::default().borders(Borders::ALL).title("Items"))
                                .start_corner(Corner::TopLeft)
                                .render(&mut f, info_chunks[1]);
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
                Key::Char('q') => {
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
                Key::Char('e') => {
                    app.open_selected_item_list();
                }
                Key::Char('d') => {
                }

                _ => {}
            },
        }
    }
    Ok(())
}

fn draw_add_menu(terminal: &mut Term, app: &App, user_input: String) -> Result<(), failure::Error> {
    terminal.draw(|mut f| {
        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .margin(2)
            .constraints([Constraint::Length(3), Constraint::Min(1)].as_ref())
            .split(f.size());
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
                    .block(Block::default().borders(Borders::ALL).title("List"))
                    .render(&mut f, chunks[1]);
            }
            None => {}
        }

        // match list.selected {
        //     Some(index) => {
        //         let list = list.items[index].list
        //             .items
        //             .iter()
        //             .enumerate()
        //             .map(|(i, m)| Text::raw(format!("{}: {}", i, m.name)));
        //         TuiList::new(list)
        //             .block(Block::default().borders(Borders::ALL).title("List"))
        //             .render(&mut f, chunks[1]);
        //     }
        //     None => {}
        // }
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
