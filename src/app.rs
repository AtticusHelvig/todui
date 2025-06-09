use color_eyre::eyre::Result;
use ratatui::crossterm::event::{self, Event, KeyCode, KeyEvent, KeyEventKind};
use ratatui::prelude::{Buffer, Constraint, Layout, Rect, Stylize};
use ratatui::style::{Color, Modifier, Style};
use ratatui::text::Line;
use ratatui::widgets::{Block, BorderType, List, ListItem, ListState, StatefulWidget, Widget};
use ratatui::{DefaultTerminal, Frame};

const SELECTED_STYLE: Style = Style::new()
    .bg(Color::DarkGray)
    .add_modifier(Modifier::BOLD);

/// Holds current application state
pub struct App {
    todo_list: TodoList,
    exit: bool,
}

pub struct TodoItem {
    status: Status,
    todo: String,
    info: String,
}

pub struct TodoList {
    items: Vec<TodoItem>,
    state: ListState,
}

pub enum Status {
    Todo,
    Completed,
}

impl App {
    pub fn new() -> Self {
        Self {
            exit: false,
            todo_list: TodoList::from_iter([
                (
                    Status::Todo,
                    "Get a list on the screen.",
                    "Seems as though I succeeded.",
                ),
                (
                    Status::Todo,
                    "Get a list on the screen. 2",
                    "Seems as though I succeeded.",
                ),
            ]),
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
            KeyCode::Char('j') => self.todo_list.state.select_next(),
            KeyCode::Char('k') => self.todo_list.state.select_previous(),
            KeyCode::Char('g') => self.todo_list.state.select_first(),
            KeyCode::Char('G') => self.todo_list.state.select_last(),
            KeyCode::Char('x') => self.toggle_status(),
            _ => {}
        }
    }

    pub fn exit(&mut self) {
        self.exit = true;
    }

    pub fn toggle_status(&mut self) {
        if let Some(i) = self.todo_list.state.selected() {
            self.todo_list.items[i].status = match self.todo_list.items[i].status {
                Status::Todo => Status::Completed,
                Status::Completed => Status::Todo,
            }
        }
    }
}

impl Widget for &mut App {
    fn render(self, area: Rect, buf: &mut Buffer) {
        let [border_area] = Layout::vertical([Constraint::Fill(1)])
            .margin(1)
            .areas(area);
        let [inner_area] = Layout::vertical([Constraint::Fill(1)])
            .horizontal_margin(2)
            .vertical_margin(1)
            .areas(border_area);

        Block::bordered()
            .title(Line::raw(" TODO ").centered())
            .border_type(BorderType::Rounded)
            .fg(Color::White)
            .render(border_area, buf);

        let list = List::new(self.todo_list.items.iter().map(|x| ListItem::from(x)))
            .highlight_style(SELECTED_STYLE);
        StatefulWidget::render(list, inner_area, buf, &mut self.todo_list.state);
    }
}

impl TodoItem {
    fn new(status: Status, todo: &'static str, info: &'static str) -> Self {
        Self {
            status,
            todo: String::from(todo),
            info: String::from(info),
        }
    }
}

impl TodoList {
    fn new() -> TodoList {
        Self {
            items: Vec::new(),
            state: ListState::default(),
        }
    }
}

impl FromIterator<(Status, &'static str, &'static str)> for TodoList {
    fn from_iter<I: IntoIterator<Item = (Status, &'static str, &'static str)>>(iter: I) -> Self {
        let items = iter
            .into_iter()
            .map(|(status, todo, info)| TodoItem::new(status, todo, info))
            .collect();
        let state = ListState::default();
        Self { items, state }
    }
}

impl From<&TodoItem> for ListItem<'_> {
    fn from(value: &TodoItem) -> Self {
        let text = match value.status {
            Status::Todo => {
                format!("☐ {}", value.todo)
            }
            Status::Completed => {
                format!("✓ {}", value.todo)
            }
        };
        ListItem::new(text)
    }
}
