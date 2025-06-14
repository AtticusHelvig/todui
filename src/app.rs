use crate::data;
use crate::widget::{InputField, Wrap};
use color_eyre::eyre::Result;
use ratatui::crossterm::event::{self, Event, KeyCode, KeyEvent};
use ratatui::layout::{Constraint, Flex, Layout, Rect};
use ratatui::style::{Color, Modifier, Style, Stylize};
use ratatui::text::Line;
use ratatui::widgets::{Block, BorderType, Borders, List, ListItem, ListState, Paragraph};
use ratatui::{DefaultTerminal, Frame};
use serde::{Deserialize, Serialize};
use tui_input::Input;
use tui_input::backend::crossterm::EventHandler;

const SELECTED_STYLE: Style = Style::new()
    .bg(Color::DarkGray)
    .add_modifier(Modifier::BOLD);

/// Holds current application state
#[derive(Default)]
pub struct App {
    todo_list: TodoList,
    editing_index: Option<usize>,
    view: View,
    input: Input,
    focus: Option<Focus>,
    edit_mode: Option<EditMode>,
    exit: bool,
}

/// Represents a task to be done
#[derive(Serialize, Deserialize, Debug)]
pub struct TodoItem {
    status: Status,
    todo: String,
    info: String,
}

/// Wrapper around a Vec of TodoItems and the ListState (for the List Widget)
#[derive(Default)]
pub struct TodoList {
    items: Vec<TodoItem>,
    state: ListState,
}

/// Represents whether a TodoItem is done or not
#[derive(Serialize, Deserialize, Debug, Default)]
pub enum Status {
    #[default]
    Todo,
    Completed,
}

/// Represents a "page" of the app
#[derive(Default)]
pub enum View {
    #[default]
    List,
    Edit,
}

/// Represents a vim-like editor mode
#[derive(Clone)]
pub enum EditMode {
    Normal,
    Insert,
}

/// Represents the currently selected input field
#[derive(Clone)]
pub enum Focus {
    Todo,
    Info,
}

impl App {
    /// Handles main application loop
    pub fn run(&mut self, terminal: &mut DefaultTerminal) -> Result<()> {
        // Read todos from file
        if let Ok(list) = data::read_todos() {
            self.todo_list.items = list;
        }
        while !self.exit {
            // Rendering
            terminal.draw(|frame| self.render(frame))?;
            // Input handling
            self.handle_events()?;
        }
        return Ok(());
    }

    /// Handles all input events from user (discards non-key events)
    fn handle_events(&mut self) -> Result<()> {
        if let Some(key) = event::read()?.as_key_press_event() {
            self.handle_key_event(key);
        }
        return Ok(());
    }

    /// Handles keyboard inputs from user
    fn handle_key_event(&mut self, key: KeyEvent) {
        match self.view {
            View::List => self.handle_list_key_event(key),
            View::Edit => self.handle_edit_key_event(key),
        }
    }

    /// Responsible for handling keyboard input in List View
    fn handle_list_key_event(&mut self, key: KeyEvent) {
        match key.code {
            KeyCode::Char('q') => self.exit(),
            KeyCode::Char('j') => self.todo_list.state.select_next(),
            KeyCode::Char('k') => self.todo_list.state.select_previous(),
            KeyCode::Char('g') => self.todo_list.state.select_first(),
            KeyCode::Char('G') => self.todo_list.state.select_last(),
            KeyCode::Char('x') => self.toggle_status(),
            KeyCode::Char('d') => self.delete_entry(),
            KeyCode::Char('a') => self.add_entry(),
            _ => {}
        }
    }

    /// Responsible for handling keyboard input in Edit View
    fn handle_edit_key_event(&mut self, key: KeyEvent) {
        let edit_mode = self.edit_mode.as_ref().expect("Expected an editor mode.");
        match edit_mode {
            EditMode::Normal => match key.code {
                KeyCode::Char('q') => self.switch_view(View::List),
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

    /// Marks the app for closure
    fn exit(&mut self) {
        self.exit = true;
        _ = data::write_todos(&self.todo_list.items);
    }

    /// Toggles a TodoItem from Todo to Complete or vice-versa
    fn toggle_status(&mut self) {
        if let Some(i) = self.todo_list.state.selected() {
            self.todo_list.items[i].status = match self.todo_list.items[i].status {
                Status::Todo => Status::Completed,
                Status::Completed => Status::Todo,
            }
        }
    }

    /// Deletes the currently selected TodoItem
    fn delete_entry(&mut self) {
        if let Some(index) = self.todo_list.state.selected() {
            self.todo_list.items.remove(index);
        }
    }

    /// Adds a new TodoItem to the list and enters Edit View
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

    /// Sets the application view
    fn switch_view(&mut self, view: View) {
        // Do any necessary cleanup
        match self.view {
            View::Edit => self.save_input(),
            _ => {}
        }
        // Do any necessary setup
        match view {
            View::List => {
                self.edit_mode = None;
                self.focus = None;
            }
            View::Edit => {
                self.focus = Some(Focus::Todo);
            }
        }
        self.view = view;
    }

    /// Switches to desired 'Focus' (input field)
    fn switch_focus(&mut self, focus: Focus) {
        self.save_input();

        let err = "Expected a selected ListItem in Edit View.";
        let index = self.editing_index.expect(err);
        let selected_item = self.todo_list.items.get(index).expect(err);

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

    /// Graphically switches to the Focus below the current one
    fn focus_down(&mut self) {
        if let Some(focus) = &self.focus {
            let below = match focus {
                Focus::Todo => Focus::Info,
                Focus::Info => Focus::Info,
            };
            self.switch_focus(below);
        }
    }

    /// Graphically switches to the Focus above the current one
    fn focus_up(&mut self) {
        if let Some(focus) = &self.focus {
            let above = match focus {
                Focus::Todo => Focus::Todo,
                Focus::Info => Focus::Todo,
            };
            self.switch_focus(above);
        }
    }

    /// Saves the Input into the TodoItem
    fn save_input(&mut self) {
        let err = "Expected a selected ListItem while saving.";
        let index = self.editing_index.expect(err);
        let selected_item = self.todo_list.items.get_mut(index).expect(err);
        let input = self.input.value().to_string();

        if let Some(focus) = &self.focus {
            match focus {
                Focus::Todo => selected_item.todo = input,
                Focus::Info => selected_item.info = input,
            }
        }
    }
}

// Rendering Logic
impl App {
    /// Renders the application to a given Frame
    fn render(&mut self, f: &mut Frame) {
        match self.view {
            View::List => self.render_list_view(f),
            View::Edit => self.render_edit_view(f),
        }
    }

    /// Renders the application in List View
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

    /// Renders the application in Edit View
    fn render_edit_view(&mut self, f: &mut Frame) {
        let err = "Expected a focus while in edit view.";
        let focus = self.focus.clone().expect(err);
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

        // Handle the focused area
        let input_field = InputField::new(self.input.value().to_string(), Wrap::Word);
        match focus {
            Focus::Todo => f.render_widget(&input_field, todo_area),
            Focus::Info => f.render_widget(&input_field, info_area),
        }

        // Handle the non focused areas
        if !matches!(focus, Focus::Todo) {
            let err = "Expected a selected ListItem in Edit View.";
            let index = self.editing_index.expect(err);
            let selected_item = self.todo_list.items.get(index).expect(err);
            let text = selected_item.todo.clone();
            f.render_widget(&InputField::new(text, Wrap::Word), todo_area);
        }

        if !matches!(focus, Focus::Info) {
            let err = "Expected a selected ListItem in Edit View.";
            let index = self.editing_index.expect(err);
            let selected_item = self.todo_list.items.get(index).expect(err);
            let text = selected_item.info.clone();
            f.render_widget(&InputField::new(text, Wrap::Word), info_area);
        }

        // Footer area
        let editor_mode = match self.edit_mode.as_ref().expect("Expected an editor mode.") {
            EditMode::Normal => " NORMAL Mode ",
            EditMode::Insert => " INSERT Mode ",
        };
        f.render_widget(Paragraph::new(editor_mode), footer_area);

        // Render cursor
        match self.focus.clone().expect("Expected a focus.") {
            Focus::Todo => render_cursor(
                f,
                input_field.get_cursor_at(todo_area, self.input.value().len()),
            ),
            Focus::Info => render_cursor(
                f,
                input_field.get_cursor_at(info_area, self.input.value().len()),
            ),
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

/// Renders the cursor as needed
fn render_cursor(f: &mut Frame, pos: (u16, u16)) {
    f.set_cursor_position(pos)
}

/// Helper function to make a centered area of any size
fn centered_area(area: Rect, x: u16, y: u16) -> Rect {
    let vertical = Layout::vertical([Constraint::Length(y)]).flex(Flex::Center);
    let horizontal = Layout::horizontal([Constraint::Length(x)]).flex(Flex::Center);
    let [area] = vertical.areas(area);
    let [area] = horizontal.areas(area);
    area
}
