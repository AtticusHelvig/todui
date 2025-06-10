use color_eyre::eyre::Result;
use ratatui::crossterm::event::{self, Event, KeyCode, KeyEvent};
use ratatui::layout::{Constraint, Flex, Layout, Rect};
use ratatui::style::{Color, Modifier, Style, Stylize};
use ratatui::text::Line;
use ratatui::widgets::{Block, BorderType, Borders, List, ListItem, ListState, Paragraph, Wrap};
use ratatui::{DefaultTerminal, Frame};
use tui_input::Input;
use tui_input::backend::crossterm::EventHandler;

const SELECTED_STYLE: Style = Style::new()
    .bg(Color::DarkGray)
    .add_modifier(Modifier::BOLD);

/// Holds current application state
pub struct App {
    todo_list: TodoList,
    editing_index: Option<usize>,
    view: View,
    input: Input,
    cursor_pos: Option<(u16, u16)>,
    focus: Option<Focus>,
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

#[derive(Clone)]
pub enum EditMode {
    Normal,
    Insert,
}

#[derive(Clone)]
pub enum Focus {
    Todo,
    Info,
}

impl App {
    pub fn new() -> Self {
        Self {
            exit: false,
            view: View::List,
            input: Input::default(),
            cursor_pos: None,
            focus: None,
            edit_mode: None,
            editing_index: None,
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
        self.render(frame);
    }

    fn handle_events(&mut self) -> Result<()> {
        if let Some(key) = event::read()?.as_key_press_event() {
            self.handle_key_event(key);
        }
        return Ok(());
    }

    fn handle_key_event(&mut self, key: KeyEvent) {
        match self.view {
            View::List => self.handle_list_key_event(key),
            View::Edit => self.handle_edit_key_event(key),
        }
    }

    fn handle_list_key_event(&mut self, key: KeyEvent) {
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
                KeyCode::Char('j') => self.focus_down(),
                KeyCode::Char('k') => self.focus_up(),
                _ => {}
            },
            EditMode::Insert => match key.code {
                KeyCode::Esc => self.edit_mode = Some(EditMode::Normal),
                _ => {
                    self.input.handle_event(&Event::Key(key));
                }
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
        self.todo_list
            .items
            .push(TodoItem::new(Status::Todo, "", ""));
        self.input.reset();
        self.todo_list.state.select_last();
        self.editing_index = Some(self.todo_list.items.len() - 1);
        self.switch_view(View::Edit);
        self.edit_mode = Some(EditMode::Insert);
    }

    fn switch_view(&mut self, view: View) {
        match view {
            View::List => {
                self.view = View::List;
                self.edit_mode = None;
                self.focus = None;
            }
            View::Edit => {
                self.view = View::Edit;
                self.focus = Some(Focus::Todo);
            }
        }
    }

    fn switch_focus(&mut self, focus: Focus) {
        let selected_item = self
            .todo_list
            .items
            .get(
                self.editing_index
                    .expect("Expected a ListItem in Edit View."),
            )
            .expect("Expected a valid ListItem in Edit View.");

        match focus {
            Focus::Todo => {
                self.input = Input::new(selected_item.todo.clone());
            }
            Focus::Info => {
                self.input = Input::new(selected_item.info.clone());
            }
        }
        self.focus = Some(focus);
    }

    fn focus_down(&mut self) {
        let selected_item = self
            .todo_list
            .items
            .get_mut(
                self.editing_index
                    .expect("Expected a ListItem in Edit View."),
            )
            .expect("Expected a valid ListItem in Edit View.");

        match self.focus.clone() {
            Some(focus) => {
                let below = match focus {
                    Focus::Todo => {
                        selected_item.todo = self.input.value().to_string();
                        Focus::Info
                    }
                    Focus::Info => Focus::Info,
                };
                self.switch_focus(below);
            }
            None => {}
        }
    }

    fn focus_up(&mut self) {
        let selected_item = self
            .todo_list
            .items
            .get_mut(
                self.editing_index
                    .expect("Expected a ListItem in Edit View."),
            )
            .expect("Expected a valid ListItem in Edit View.");

        match self.focus.clone() {
            Some(focus) => {
                let above = match focus {
                    Focus::Todo => Focus::Todo,
                    Focus::Info => {
                        selected_item.info = self.input.value().to_string();
                        Focus::Todo
                    }
                };
                self.switch_focus(above);
            }
            None => {}
        }
    }
}

// Rendering Logic
impl App {
    fn render(&mut self, f: &mut Frame) {
        match self.view {
            View::List => self.render_list_view(f),
            View::Edit => self.render_edit_view(f),
        }
    }

    fn render_list_view(&mut self, f: &mut Frame) {
        let [border_area] = Layout::vertical([Constraint::Fill(1)])
            .margin(1)
            .areas(f.area());
        let [inner_area] = Layout::vertical([Constraint::Fill(1)])
            .horizontal_margin(2)
            .vertical_margin(1)
            .areas(border_area);

        f.render_widget(
            Block::bordered()
                .title(Line::raw(" TODO ").centered())
                .border_type(BorderType::Rounded)
                .fg(Color::White),
            border_area,
        );

        let list = List::new(self.todo_list.items.iter().map(|x| ListItem::from(x)))
            .highlight_style(SELECTED_STYLE);
        f.render_stateful_widget(list, inner_area, &mut self.todo_list.state);
    }

    fn render_edit_view(&mut self, f: &mut Frame) {
        let focus = self
            .focus
            .clone()
            .expect("Expected a focus while in edit view.");
        // Outer border
        let bordered_area = centered_area(f.area(), 40, 15);
        f.render_widget(
            Block::bordered()
                .border_type(BorderType::Rounded)
                .fg(Color::White),
            bordered_area,
        );

        let [
            _header_area,
            todo_area,
            separator_area,
            info_area,
            footer_area,
        ] = Layout::vertical([
            Constraint::Length(1),
            Constraint::Length(1),
            Constraint::Length(1),
            Constraint::Fill(1),
            Constraint::Length(1),
        ])
        .horizontal_margin(1)
        .areas(bordered_area);

        f.render_widget(
            Block::bordered()
                .borders(Borders::BOTTOM)
                .border_type(BorderType::Plain)
                .fg(Color::White),
            separator_area,
        );

        // Todo entry area
        if matches!(focus, Focus::Todo) {
            if self.input.value().len() > todo_area.width as usize - 1 {
                let mut truncated = self.input.value().to_string();
                truncated.truncate(todo_area.width as usize - 1);
                self.input = self.input.clone().with_value(truncated.to_string());
            }
            f.render_widget(Paragraph::new(self.input.value()), todo_area);
        } else {
            f.render_widget(
                Paragraph::new(
                    self.todo_list
                        .items
                        .get(
                            self.editing_index
                                .expect("Expected a selected TodoItem in Edit View."),
                        )
                        .expect("Expected a valid TodoItem while in Edit View.")
                        .todo
                        .clone(),
                ),
                todo_area,
            );
        }

        if matches!(focus, Focus::Info) {
            if self.input.value().len() > (info_area.width * info_area.height - 1) as usize {
                let mut truncated = self.input.value().to_string();
                truncated.truncate((info_area.width * info_area.height - 1) as usize);
                self.input = self.input.clone().with_value(truncated.to_string());
            }
            f.render_widget(
                Paragraph::new(self.input.value()).wrap(Wrap { trim: false }),
                info_area,
            );
        } else {
            f.render_widget(
                Paragraph::new(
                    self.todo_list
                        .items
                        .get(
                            self.editing_index
                                .expect("Expected a selected TodoItem in Edit View."),
                        )
                        .expect("Expected a valid TodoItem while in Edit View.")
                        .info
                        .clone(),
                )
                .wrap(Wrap { trim: false }),
                info_area,
            );
        }

        // Footer area
        let editor_mode = match self.edit_mode.as_ref().expect("Expected an editor mode.") {
            EditMode::Normal => " NORMAL Mode ",
            EditMode::Insert => " INSERT Mode ",
        };
        f.render_widget(Paragraph::new(editor_mode), footer_area);

        // Render cursor
        match self.focus.clone().expect("Expected a focus.") {
            Focus::Todo => self.render_cursor(f, todo_area),
            Focus::Info => self.render_cursor(f, info_area),
        }
    }

    fn render_cursor(&mut self, f: &mut Frame, area: Rect) {
        self.cursor_pos = match self.edit_mode.clone().expect("Expected an editor mode.") {
            EditMode::Insert => {
                let x = self.input.visual_cursor() as u16 % area.width + area.x;
                let y =
                    (self.input.visual_cursor() as u16 / area.width).min(area.height - 1) + area.y;
                Some((x, y))
            }
            _ => self.cursor_pos,
        };
        match self.cursor_pos {
            Some(pos) => f.set_cursor_position(pos),
            None => {}
        }
    }
}

impl TodoItem {
    fn new(status: Status, todo: &str, info: &str) -> Self {
        Self {
            status,
            todo: String::from(todo),
            info: String::from(info),
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
