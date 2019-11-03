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
struct List{
    list_type: String,
    name: String,
    items: Vec<Item>,
    selected: Option<usize>,
    previous: Option<Box<List>>,
}

impl List{
    pub fn new(list_type: String, name: String) -> List {
        List{
            list_type: list_type,
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
    list: List,
}

impl Item{
    pub fn new(item_type: String, id: String, name: String) -> Item {
        Item{
            id: id,
            name: name.clone(),
            list: List::new(item_type.clone(), name.clone()),
        }
    }
}

struct App{
    list: List,
}

impl App {
}

fn main() -> Result<(), failure::Error> {
    let items = vec![
        Item{id: String::from("item1id"), name: String::from("item1name"), list: List::new(String::from("item"), String::from("item1"))},
        Item{id: String::from("item2id"), name: String::from("item2name"), list: List::new(String::from("item"), String::from("item2"))},
        Item{id: String::from("item3id"), name: String::from("item3name"), list: List::new(String::from("item"), String::from("item3"))},
        Item{id: String::from("item4id"), name: String::from("item4name"), list: List::new(String::from("item"), String::from("item4"))},
        Item{id: String::from("item5id"), name: String::from("item5name"), list: List::new(String::from("item"), String::from("item5"))},
    ];

    let mut list = List::new(String::from("list type"), String::from("list name"));
    list.items = items;
    list.selected = Some(0);

    run(list)
}

fn run(mut list: List) -> Result<(), failure::Error> {
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

            let title = format!("{}: {}", list.list_type, list.name);
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

            match list.selected {
                Some(index) => {
                    let item = list.items[index].clone();

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

                    let item_list = list.items[index].list.items.iter().map(|i| {
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

        })?;

        match events.next()? {
            Event::Input(input) => match input {
                Key::Ctrl('c') => {
                    break;
                }
                Key::Char('q') => {
                    match list.previous {
                        Some(prev_list) => {
                            list = *prev_list;
                        }
                        None => { break; }
                    }
                }
                Key::Left => {
                    list.selected = None;
                }
                Key::Down => {
                    list.selected = if let Some(selected) = list.selected {
                        if selected >= list.items.len() - 1 {
                            Some(0)
                        } else {
                            Some(selected + 1)
                        }
                    } else {
                        Some(0)
                    };
                }
                Key::Up => {
                    list.selected = if let Some(selected) = list.selected {
                        if selected > 0 {
                            Some(selected - 1)
                        } else {
                            Some(list.items.len() - 1)
                        }
                    } else {
                        Some(0)
                    }
                }
                Key::Char('a') => {
                    let mut user_input: String = String::new();

                    'input: loop {
                        draw_add_menu(&mut terminal, &list, user_input.clone())?;

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
                                        match list.selected {
                                            Some(index) => {
                                                let input: String = user_input.drain(..).collect();
                                                let id = input.clone();
                                                let name = input.clone();
                                                let item = Item::new(String::from("Item"), id, name);
                                                list.items[index].list.items.push(item);
                                            }
                                            None => {}
                                        }
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
                            _ => {}
                        }
                    };
                }
                Key::Char('e') => {
                    match list.selected {
                        Some(index) => {
                            let current_list = list.clone();
                            let mut next_list = list.items[index].list.clone();

                            if next_list.items.len() > 0 {
                                next_list.previous = Some(Box::new(current_list));
                                list = next_list;
                            }
                        }
                        None => {}
                    }
                }

                _ => {}
            },
            Event::Tick => {}
        }
    }
    Ok(())
}

fn draw_add_menu(terminal: &mut Term, list: &List, user_input: String) -> Result<(), failure::Error> {
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

        match list.selected {
            Some(index) => {
                let list = list.items[index].list
                    .items
                    .iter()
                    .enumerate()
                    .map(|(i, m)| Text::raw(format!("{}: {}", i, m.name)));
                TuiList::new(list)
                    .block(Block::default().borders(Borders::ALL).title("List"))
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
