mod util;

use std::io::{self, Write};

use termion::cursor::Goto;
use termion::event::Key;
use termion::input::MouseTerminal;
use termion::raw::IntoRawMode;
use termion::screen::AlternateScreen;
use tui::backend::TermionBackend;
use tui::layout::{Constraint, Corner, Direction, Layout};
use tui::style::{Color, Modifier, Style};
use tui::widgets::{Block, Borders, List, SelectableList, Paragraph, Text, Widget};
use tui::Terminal;
use unicode_width::UnicodeWidthStr;

use crate::util::event::{Event, Events};

#[derive(Clone, Debug)]
struct Item {
    id: String,
    name: String,
    items: Vec<Item>,
}

impl Item{
    pub fn new(id: String, name: String) -> Item {
        Item{
            id: id,
            name: name,
            items: Vec::new(),
        }
    }
}

struct App{
    items: Vec<Item>,
    selected: Option<usize>,
    item_primary_style: Style,
    item_secondary_style: Style,
    input: String,
}

impl App {
    fn new() -> App {
        let items = vec![
            Item{id: String::from("item1id"), name: String::from("item1name"), items: Vec::new()},
            Item{id: String::from("item2id"), name: String::from("item2name"), items: Vec::new()},
            Item{id: String::from("item3id"), name: String::from("item3name"), items: Vec::new()},
            Item{id: String::from("item4id"), name: String::from("item4name"), items: Vec::new()},
            Item{id: String::from("item5id"), name: String::from("item5name"), items: Vec::new()},
        ];

        App{
            items: items,
            selected: None,
            item_primary_style: Style::default().fg(Color::White),
            item_secondary_style: Style::default().fg(Color::Yellow),
            input: String::new(),
        }
    }
}

fn main() -> Result<(), failure::Error> {
    let app = App::new();
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
            let chunks = Layout::default()
                .direction(Direction::Horizontal)
                .constraints([Constraint::Percentage(30), Constraint::Percentage(70)].as_ref())
                .split(f.size());

            let style = Style::default().fg(Color::Black).bg(Color::White);
            SelectableList::default()
                .block(Block::default().borders(Borders::ALL).title("List"))
                .items(&app.items.iter().map(|i| { i.name.clone() }).collect::<Vec<_>>())
                .select(app.selected)
                .style(style)
                .highlight_style(style.fg(Color::LightGreen).modifier(Modifier::BOLD))
                .highlight_symbol("=>")
                .render(&mut f, chunks[0]);

            match app.selected {
                Some(index) => {
                    let item = app.items[index].clone();

                    let fields = vec![
                        format!("Item ID: {}", item.id),
                        format!("Item Name: {}", item.name),
                    ];

                    let item_info = fields.iter().map(|i| {
                        Text::styled(
                            format!("{}", i),
                            app.item_primary_style,
                        )
                    });

                    List::new(item_info)
                        .block(Block::default().borders(Borders::ALL).title("Selected"))
                        .start_corner(Corner::TopLeft)
                        .render(&mut f, chunks[1]);

                    let item_list = app.items[index].items.iter().map(|i| {
                        Text::styled(
                            format!("{}", i.name),
                            app.item_secondary_style,
                        )
                    });

                    List::new(item_list)
                        .block(Block::default().borders(Borders::ALL).title("Selected"))
                        .start_corner(Corner::BottomLeft)
                        .render(&mut f, chunks[1]);
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
                    break;
                }
                Key::Left => {
                    app.selected = None;
                }
                Key::Down => {
                    app.selected = if let Some(selected) = app.selected {
                        if selected >= app.items.len() - 1 {
                            Some(0)
                        } else {
                            Some(selected + 1)
                        }
                    } else {
                        Some(0)
                    };
                }
                Key::Up => {
                    app.selected = if let Some(selected) = app.selected {
                        if selected > 0 {
                            Some(selected - 1)
                        } else {
                            Some(app.items.len() - 1)
                        }
                    } else {
                        Some(0)
                    }
                }
                Key::Char('\n') => {
                    println!("{:?}", app.selected)
                }
                Key::Char('a') => {
                    println!("{:?}", app.selected);

                    'input: loop {
                        terminal.draw(|mut f| {
                            let chunks = Layout::default()
                                .direction(Direction::Vertical)
                                .margin(2)
                                .constraints([Constraint::Length(3), Constraint::Min(1)].as_ref())
                                .split(f.size());
                            Paragraph::new([Text::raw(&app.input)].iter())
                                .style(Style::default().fg(Color::Yellow))
                                .block(Block::default().borders(Borders::ALL).title("Input"))
                                .render(&mut f, chunks[0]);

                            match app.selected {
                                Some(index) => {
                                    let list = app.items[index]
                                        .items
                                        .iter()
                                        .enumerate()
                                        .map(|(i, m)| Text::raw(format!("{}: {}", i, m.name)));
                                    List::new(list)
                                        .block(Block::default().borders(Borders::ALL).title("List"))
                                        .render(&mut f, chunks[1]);
                                }
                                None => {}
                            }
                        })?;

                        write!(
                            terminal.backend_mut(),
                            "{}",
                            Goto(4 + app.input.width() as u16, 4)
                        )?;
                        // stdout is buffered, flush it to see the effect immediately when hitting backspace
                        io::stdout().flush().ok();

                        // Handle input
                        match events.next()? {
                            Event::Input(input) => match input {
                                Key::Ctrl('c') => {
                                    break 'main;
                                }
                                Key::Char('q') => {
                                    break;
                                }
                                Key::Char('\n') => {
                                    match app.selected {
                                        Some(index) => {
                                            let input: String = app.input.drain(..).collect();
                                            let id = input.clone();
                                            let name = input.clone();
                                            let item = Item::new(id, name);
                                            app.items[index].items.push(item);
                                        }
                                        None => {}
                                    }
                                }
                                Key::Char(c) => {
                                    app.input.push(c);
                                }
                                Key::Backspace => {
                                    app.input.pop();
                                }
                                _ => {}
                            },
                            _ => {}
                        }
                    };
                }
                _ => {}
            },
            Event::Tick => {
                // app.advance();
            }
        }
    }
    Ok(())
}
