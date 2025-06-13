use crate::app::TodoItem;
use directories::BaseDirs;
use std::fs::File;
use std::io;
use std::io::prelude::*;

pub enum Error {
    IO(std::io::Error),
    Serde(serde_json::Error),
}

impl From<io::Error> for Error {
    fn from(e: io::Error) -> Self {
        Error::IO(e)
    }
}

impl From<serde_json::Error> for Error {
    fn from(e: serde_json::Error) -> Self {
        Error::Serde(e)
    }
}

pub fn read_todos() -> Result<Vec<TodoItem>, Error> {
    let base_dir = match BaseDirs::new() {
        Some(val) => val,
        None => return Err(io::Error::other("No home directory found.").into()),
    };
    let data_dir = base_dir.data_dir();
    let file_path = data_dir.join("todo").join("todos.json");

    let mut file = File::open(file_path)?;
    let mut as_string = String::new();
    file.read_to_string(&mut as_string)?;

    Ok(serde_json::from_str(&as_string)?)
}

pub fn write_todos(todos: &Vec<TodoItem>) -> Result<(), Error> {
    let base_dir = match BaseDirs::new() {
        Some(val) => val,
        None => return Err(io::Error::other("No home directory found.").into()),
    };
    let data_dir = base_dir.data_dir();
    let todo_dir = data_dir.join("todo");
    let file_path = todo_dir.join("todos.json");

    let json_string = serde_json::to_string(todos)?;

    std::fs::create_dir_all(todo_dir)?;
    let mut file = File::create(file_path)?;
    file.write_all(json_string.as_bytes())?;

    Ok(())
}
