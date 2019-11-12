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
use crate::list::{List, Item};
use crate::options::{Options, PageOptions};

type Term = Terminal<TermionBackend<AlternateScreen<MouseTerminal<termion::raw::RawTerminal<std::io::Stdout>>>>>;

pub struct App{
    pub lists: Vec<List>,
    current: usize,
    pub options: Options,
    depth: usize,
    on_save: Box<dyn Fn(Vec<List>) -> Option<String>>,
    running: bool,
    notification: Option<String>,
}

impl App{
    pub fn new(root_list: List) -> App {
        App{
            lists: vec!(root_list),
            current: 0,
            options: Options::new(),
            depth: 0,
            on_save: Box::new(|_: Vec<List>| None),
            running: false,
            notification: None,
        }
    }

    pub fn register_save_handler(&mut self, on_save: Box<dyn Fn(Vec<List>) -> Option<String>>) {
        self.on_save = on_save
    }

    pub fn save(&mut self) {
        self.notification = (self.on_save)(self.lists.clone());
    }

    pub fn stop(&mut self) {
        self.running = false;
    }

    pub fn add_list(&mut self, name: String) -> usize {
        let mut list = List::new(name);
        list.previous = Some(self.current);
        self.lists.push(list);

        return self.lists.len() - 1;
    }

    fn get_current_page_options(&self) -> PageOptions {
        if self.options.page_options.len() > self.depth {
            self.options.page_options[self.depth].clone()
        } else {
            PageOptions::new(format!("{}", self.depth))
        }
    }

    fn get_current_list(&self) -> &List {
        &self.lists[self.current]
    }

    fn get_selected_item(&self) -> Option<&Item> {
        self.lists[self.current].get_selected_item()
    }

    fn get_list_for_selected_item(&self) -> Option<&List> {
        match self.lists[self.current].get_selected_item() {
            Some(selected) => {
                match selected.list_index {
                    Some(selected_item_list_index) => {
                        return Some(&self.lists[selected_item_list_index]);
                    }
                    None => { return None; }

                }
            }
            None => { return None; }
        }
    }

    fn add_list_item(&mut self, name: String, id: String) {
        match self.lists[self.current].get_selected_item() {
            Some(selected_item) => {
                match selected_item.list_index {
                    Some(_) => {}
                    None => {
                        let index = self.add_list(name.clone());
                        let list = &mut self.lists[self.current];

                        list.set_selected_item_list_index(Some(index));
                    }
                }

                let item = Item::new(id, name.clone());
                match self.lists[self.current].get_selected_item() {
                    Some(selected_item) => {
                        match selected_item.list_index {
                            Some(list_index) => {
                                self.lists[list_index].items.push(item);
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

    fn can_go_back(&self) -> bool {
        match self.lists[self.current].previous {
            Some(_) => {
                true
            }
            None => {
                false
            }
        }
    }

    fn close_current_list(&mut self) {
        self.notification = None;

        match self.lists[self.current].previous {
            Some(previous_index) => {
                self.current = previous_index;
                self.depth = self.depth - 1;

                // if list that was just closed doesn't have any items,
                // remove list_index from item that owns the list
                match self.lists[self.current].get_selected_item() {
                    Some(selected_item) => {
                        match selected_item.list_index {
                            Some(index) => {
                                if self.lists[index].items.len() == 0 {
                                    self.lists[self.current].set_selected_item_list_index(None);
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

    fn open_selected_item_list(&mut self) {
        self.notification = None;

        if !self.get_current_page_options().disable_edit {
            match self.lists[self.current].get_selected_item() {
                Some(selected_item) => {
                    match selected_item.list_index {
                        Some(index) => {
                            self.current = index;
                            self.depth = self.depth + 1;

                            if self.lists[index].items.len() > 0 {
                                self.lists[index].set_selected_item_index(Some(0));
                            }
                        }
                        None => {}
                    }
                }
                None => {}
            }
        }
    }

    fn delete_selected_item(&mut self) {
        if !self.get_current_page_options().disable_delete {
            match self.lists[self.current].get_selected_item() {
                Some(_) => {
                    self.lists[self.current].set_selected_item_list_index(None);
                    self.lists[self.current].remove_selected_item();

                    if self.lists[self.current].items.len() == 0 {
                        self.close_current_list();
                    }
                }
                None => {}
            }
        }
    }

    pub fn run(&mut self) -> Result<(), failure::Error> {
        if self.lists.len() == 0 {
            println!("No root list found");
            return Ok(());
        }

        if self.lists[0].items.len() == 0 {
            println!("No items in root list");
            return Ok(());
        }

        self.running = true;

        // Terminal initialization
        let stdout = io::stdout().into_raw_mode()?;
        let stdout = MouseTerminal::from(stdout);
        let stdout = AlternateScreen::from(stdout);
        let backend = TermionBackend::new(stdout);
        let mut terminal = Terminal::new(backend)?;
        terminal.hide_cursor()?;

        let events = Events::new();

        'main: loop {
            let page_options = self.get_current_page_options();

            if !self.running {
                break 'main;
            }

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

                let list = self.get_current_list();
                let mut title = format!("{}: {}", page_options.title, list.name);
                let mut title_color = Color::Blue;

                match self.notification.clone() {
                    Some(notice) => {
                        title = format!("{} | {}", notice, title);
                        title_color = Color::Red;
                    }
                    None => {}
                }

                Paragraph::new([
                    Text::styled(
                        title,
                        Style::default().fg(title_color).modifier(Modifier::BOLD),
                    )].iter())
                    .block(block.clone())
                    .alignment(Alignment::Center)
                    .render(&mut f, wrapper_chunks[0]);

                let body_chunks = Layout::default()
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
                    .split(body_chunks[1]);

                let style = Style::default().fg(Color::Black).bg(Color::White);
                SelectableList::default()
                    .block(Block::default().borders(Borders::ALL).title(page_options.menu_box_title.as_str()))
                    .items(&list.items.iter().map(|i| { i.name.clone() }).collect::<Vec<_>>())
                    .select(list.get_selected_item_index())
                    .style(style)
                    .highlight_style(style.fg(Color::LightGreen).modifier(Modifier::BOLD))
                    .highlight_symbol("=>")
                    .render(&mut f, body_chunks[0]);

                match self.get_selected_item() {
                    Some(item) => {
                        let mut usage = vec![
                            "ctrl-c: exit",
                        ];

                        let save_description = format!(
                            "W: {}",
                            page_options.save_command_description.clone()
                        );

                        if !page_options.disable_save {
                            usage.push(save_description.as_str());
                        }

                        if self.can_go_back() {
                            usage.push("b: back to previous page");
                        }

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
                            .block(Block::default().borders(Borders::ALL).title("Navigation"))
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

                        match self.get_list_for_selected_item() {
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

            match events.next()? {
                Event::Input(input) => match input {
                    Key::Ctrl('c') => {
                        break;
                    }
                    Key::Left => {
                        self.close_current_list();
                    }
                    Key::Down => {
                        (&mut self.lists[self.current]).decrement_selected();
                    }
                    Key::Up => {
                        (&mut self.lists[self.current]).increment_selected();
                    }
                    Key::Char('W') => {
                        self.save();
                    }
                    Key::Char('b') => {
                        self.close_current_list();
                    }
                    Key::Char('a') => {
                        if !page_options.disable_add {
                            let mut user_input: String = String::new();

                            'input: loop {
                                if !self.running {
                                    break 'main;
                                }

                                draw_add_menu(&mut terminal, &self, user_input.clone())?;

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

                                                self.add_list_item(id, name);
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
                        self.open_selected_item_list();
                    }
                    Key::Char('d') => {
                        self.delete_selected_item();
                    }
                    _ => {}
                },
            }
        }
        Ok(())
    }

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
            "ctrl-s: save and return to previous",
        ];

        let usage_info = usage.iter().map(|i| {
            Text::styled(
                format!("{}", i),
                Style::default().fg(Color::Green),
            )
        });

        TuiList::new(usage_info)
            .block(Block::default().borders(Borders::ALL).title("Navigation"))
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

        match app.get_list_for_selected_item() {
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
