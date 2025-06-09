use color_eyre::eyre::Result;
use ratatui::buffer::Buffer;
use ratatui::crossterm::event::{self, Event, KeyCode, KeyEvent, KeyEventKind};
use ratatui::layout::{Constraint, Flex, Layout, Rect};
use ratatui::style::{Color, Modifier, Style, Stylize};
use ratatui::text::Line;
use ratatui::widgets::{
    Block, BorderType, Borders, List, ListItem, ListState, Paragraph, StatefulWidget, Widget,
};
use ratatui::{DefaultTerminal, Frame};

const SELECTED_STYLE: Style = Style::new()
    .bg(Color::DarkGray)
    .add_modifier(Modifier::BOLD);

/// Holds current application state
pub struct App {
    todo_list: TodoList,
    view: View,
    edit_mode: Option<EditMode>,
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

pub enum View {
    List,
    Edit,
}

pub enum EditMode {
    Normal,
    Insert,
}

impl App {
    pub fn new() -> Self {
        Self {
            exit: false,
            view: View::List,
            edit_mode: None,
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

    fn draw(&mut self, frame: &mut Frame) {
        frame.render_widget(self, frame.area());
    }

    fn handle_events(&mut self) -> Result<()> {
        match event::read()? {
            Event::Key(key_event) if key_event.kind == KeyEventKind::Press => {
                self.handle_key_event(key_event);
            }
            _ => {}
        }
        return Ok(());
    }

    fn handle_key_event(&mut self, key: KeyEvent) {
        match self.view {
            View::List => self.handle_normal_key_event(key),
            View::Edit => self.handle_edit_key_event(key),
        }
    }

    fn handle_normal_key_event(&mut self, key: KeyEvent) {
        match key.code {
            KeyCode::Char('q') => self.exit(),
            KeyCode::Char('j') => self.todo_list.state.select_next(),
            KeyCode::Char('k') => self.todo_list.state.select_previous(),
            KeyCode::Char('g') => self.todo_list.state.select_first(),
            KeyCode::Char('G') => self.todo_list.state.select_last(),
            KeyCode::Char('x') => self.toggle_status(),
            KeyCode::Char('a') => self.add_entry(),
            _ => {}
        }
    }

    fn handle_edit_key_event(&mut self, key: KeyEvent) {
        let edit_mode = self.edit_mode.as_ref().expect("Expected an editor mode.");
        match edit_mode {
            EditMode::Normal => match key.code {
                KeyCode::Char('q') => self.view = View::List,
                KeyCode::Char('i') => self.edit_mode = Some(EditMode::Insert),
                _ => {}
            },
            EditMode::Insert => match key.code {
                KeyCode::Esc => self.edit_mode = Some(EditMode::Normal),
                _ => {}
            },
        }
    }

    fn exit(&mut self) {
        self.exit = true;
    }

    fn toggle_status(&mut self) {
        if let Some(i) = self.todo_list.state.selected() {
            self.todo_list.items[i].status = match self.todo_list.items[i].status {
                Status::Todo => Status::Completed,
                Status::Completed => Status::Todo,
            }
        }
    }

    fn add_entry(&mut self) {
        self.switch_view(View::Edit);
        self.edit_mode = Some(EditMode::Insert);
    }

    fn switch_view(&mut self, view: View) {
        match view {
            View::List => {
                self.view = View::List;
                self.edit_mode = None;
            }
            View::Edit => {
                self.view = View::Edit;
            }
        }
    }
}

// Rendering Logic
impl App {
    fn render_list_view(&mut self, area: Rect, buf: &mut Buffer) {
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

    fn render_edit_view(&mut self, area: Rect, buf: &mut Buffer) {
        // Outer border
        let bordered_area = centered_area(area, 40, 15);
        Block::bordered()
            .border_type(BorderType::Rounded)
            .fg(Color::White)
            .render(bordered_area, buf);

        let [header_area, todo_area, info_area, footer_area] = Layout::vertical([
            Constraint::Length(1),
            Constraint::Length(2),
            Constraint::Fill(1),
            Constraint::Length(1),
        ])
        .horizontal_margin(1)
        .areas(bordered_area);
        // Todo entry area
        Block::bordered()
            .borders(Borders::BOTTOM)
            .border_type(BorderType::Plain)
            .fg(Color::White)
            .render(todo_area, buf);
        // Footer area
        let editor_mode = match self.edit_mode.as_ref().expect("Expected an editor mode.") {
            EditMode::Normal => " NORMAL Mode ",
            EditMode::Insert => " INSERT Mode ",
        };
        Paragraph::new(editor_mode).render(footer_area, buf);
    }
}

impl Widget for &mut App {
    fn render(self, area: Rect, buf: &mut Buffer) {
        match self.view {
            View::List => self.render_list_view(area, buf),
            View::Edit => self.render_edit_view(area, buf),
        }
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

/// Helper function to make a centered area of any size
fn centered_area(area: Rect, x: u16, y: u16) -> Rect {
    let vertical = Layout::vertical([Constraint::Length(y)]).flex(Flex::Center);
    let horizontal = Layout::horizontal([Constraint::Length(x)]).flex(Flex::Center);
    let [area] = vertical.areas(area);
    let [area] = horizontal.areas(area);
    area
}
