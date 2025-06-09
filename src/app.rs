use color_eyre::eyre::Result;
use ratatui::crossterm::event::{self, Event, KeyCode, KeyEvent, KeyEventKind};
use ratatui::prelude::{Buffer, Constraint, Layout, Rect, Stylize};
use ratatui::style::Color;
use ratatui::widgets::{Block, BorderType, List, ListItem, ListState, StatefulWidget, Widget};
use ratatui::{DefaultTerminal, Frame};

/// Holds current application state
pub struct App {
    todo_list: TodoList,
    exit: bool,
}

pub struct TodoList {
    items: Vec<TodoItem>,
    state: ListState,
}

#[derive(Debug)]
pub struct TodoItem {
    is_done: bool,
    description: String,
}

impl App {
    pub fn new() -> App {
        Self {
            todo_list: TodoList {
                items: Vec::new(),
                state: ListState::default(),
            },
            exit: false,
        }
    }

    pub fn run(&mut self, terminal: &mut DefaultTerminal) -> Result<()> {
        while !self.exit {
            // Rendering
            terminal.draw(|frame| self.draw(frame))?;
            // Input handling
            self.handle_events()?;
        }
        return Ok(());
    }

    pub fn draw(&mut self, frame: &mut Frame) {
        frame.render_widget(self, frame.area());
    }

    pub fn handle_events(&mut self) -> Result<()> {
        match event::read()? {
            Event::Key(key_event) if key_event.kind == KeyEventKind::Press => {
                self.handle_key_event(key_event);
            }
            _ => {}
        }
        return Ok(());
    }

    pub fn handle_key_event(&mut self, key: KeyEvent) {
        match key.code {
            KeyCode::Char('q') => self.exit(),
            _ => {}
        }
    }

    pub fn exit(&mut self) {
        self.exit = true;
    }
}

impl Widget for &mut App {
    fn render(self, area: Rect, buf: &mut Buffer) {
        let [border_area] = Layout::vertical([Constraint::Fill(1)])
            .margin(1)
            .areas(area);
        let [inner_area] = Layout::vertical([Constraint::Fill(1)])
            .margin(1)
            .areas(border_area);

        Block::bordered()
            .border_type(BorderType::Rounded)
            .fg(Color::Yellow)
            .render(border_area, buf);

        let list = List::new(
            self.todo_list
                .items
                .iter()
                .map(|x| ListItem::from(x.description.clone())),
        );
        StatefulWidget::render(list, border_area, buf, &mut self.todo_list.state);
    }
}
