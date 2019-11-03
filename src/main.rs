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
use tui::widgets::{Block, Borders, List, SelectableList, Paragraph, Text, Widget};
use tui::Terminal;
use unicode_width::UnicodeWidthStr;

use crate::util::event::{Event, Events};

type Term = Terminal<TermionBackend<AlternateScreen<MouseTerminal<termion::raw::RawTerminal<std::io::Stdout>>>>>;

#[derive(Debug, Clone)]
struct Page{
    page_type: String,
    name: String,
    items: Vec<Item>,
    selected: Option<usize>,
    previous: Option<Box<Page>>,
}

impl Page{
    pub fn new(page_type: String, name: String) -> Page {
        Page{
            page_type: page_type,
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
    page: Page,
}

impl Item{
    pub fn new(item_type: String, id: String, name: String) -> Item {
        Item{
            id: id,
            name: name.clone(),
            page: Page::new(item_type.clone(), name.clone()),
        }
    }
}

struct App{
    page: Page,
    item_primary_style: Style,
    item_secondary_style: Style,
    input: String,
}

impl App {
    fn new() -> App {
        let items = vec![
            Item{id: String::from("item1id"), name: String::from("item1name"), page: Page::new(String::from("item"), String::from("item1"))},
            Item{id: String::from("item2id"), name: String::from("item2name"), page: Page::new(String::from("item"), String::from("item2"))},
            Item{id: String::from("item3id"), name: String::from("item3name"), page: Page::new(String::from("item"), String::from("item3"))},
            Item{id: String::from("item4id"), name: String::from("item4name"), page: Page::new(String::from("item"), String::from("item4"))},
            Item{id: String::from("item5id"), name: String::from("item5name"), page: Page::new(String::from("item"), String::from("item5"))},
        ];

        let mut page = Page::new(String::from("page type"), String::from("page name"));
        page.items = items;
        page.selected = Some(0);

        App{
            page: page,
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

            let title = format!("{}: {}", app.page.page_type, app.page.name);
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
                .items(&app.page.items.iter().map(|i| { i.name.clone() }).collect::<Vec<_>>())
                .select(app.page.selected)
                .style(style)
                .highlight_style(style.fg(Color::LightGreen).modifier(Modifier::BOLD))
                .highlight_symbol("=>")
                .render(&mut f, primary_chunks[0]);

            match app.page.selected {
                Some(index) => {
                    let item = app.page.items[index].clone();

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
                        .render(&mut f, info_chunks[0]);

                    let item_list = app.page.items[index].page.items.iter().map(|i| {
                        Text::styled(
                            format!("{}", i.name),
                            app.item_secondary_style,
                        )
                    });

                    List::new(item_list)
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
                    match app.page.previous {
                        Some(prev_page) => {
                            app.page = *prev_page;
                        }
                        None => { break; }
                    }
                }
                Key::Left => {
                    app.page.selected = None;
                }
                Key::Down => {
                    app.page.selected = if let Some(selected) = app.page.selected {
                        if selected >= app.page.items.len() - 1 {
                            Some(0)
                        } else {
                            Some(selected + 1)
                        }
                    } else {
                        Some(0)
                    };
                }
                Key::Up => {
                    app.page.selected = if let Some(selected) = app.page.selected {
                        if selected > 0 {
                            Some(selected - 1)
                        } else {
                            Some(app.page.items.len() - 1)
                        }
                    } else {
                        Some(0)
                    }
                }
                Key::Char('a') => {
                    'input: loop {
                        draw_add_menu(&mut terminal, &app)?;

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
                                    match app.page.selected {
                                        Some(index) => {
                                            let input: String = app.input.drain(..).collect();
                                            let id = input.clone();
                                            let name = input.clone();
                                            let item = Item::new(String::from("Item"), id, name);
                                            app.page.items[index].page.items.push(item);
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
                Key::Char('e') => {
                    match app.page.selected {
                        Some(index) => {
                            let page = app.page.clone();
                            let mut next_page = app.page.items[index].page.clone();

                            if next_page.items.len() > 0 {
                                next_page.previous = Some(Box::new(page));
                                app.page = next_page;
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

fn draw_add_menu(terminal: &mut Term, app: &App) -> Result<(), failure::Error> {
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

        match app.page.selected {
            Some(index) => {
                let list = app.page.items[index].page
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
    Ok(())
}
