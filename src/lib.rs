use std::fs::File;
use std::path::Path;
use std::io::Error;
use uuid::Uuid;
use serde::{Serialize, Deserialize};

pub struct Config {
    pub path: String
}

#[derive(Debug)]
struct TodoError(String);

impl std::fmt::Display for TodoError {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "There is an error: {}", self.0)
    }
}

impl std::error::Error for TodoError {}

pub fn run(config: Config) -> Result<(), Box<dyn std::error::Error>> {
    let mut todos = FileProvider::read_todos(&config.path)?;

    loop {
        let unfinished = get_unfinished_todos(&todos)?;

        let options = list_options(unfinished.len());

        let msg = format!("You have {} incomplete todos in your todo list", unfinished.len());

        let response_option = get_option(&options, Some(msg.as_str()));

        match response_option {
            0 => match add_todo(&mut todos) {
                Ok(todos) => {
                    FileProvider::write_todos(todos, &config.path)?;
                    println!("Todo added");
                },
                Err(e) => println!("{:?}", e)
            },
            1 => {
                let todo_id = get_todo_to_complete(&unfinished)?;
                complete_todo(&todo_id, &mut todos).unwrap();
            },
            _ => break
        }

        FileProvider::write_todos(&todos, &config.path)?;
    }
    
    Ok(())
}

fn get_option<T: std::fmt::Display>(options: &Vec<T>, msg: Option<&str>) -> usize {
    loop {
        let mut message = String::from("\n====================================\n");
        message.push_str(msg.unwrap_or("Select Option:"));
        message.push_str("\n====================================\n");
        let mut ops = vec![message];
        for (idx, option) in options.iter().enumerate() {
            ops.push(format!("{}: {}", idx+1, option));
        }
    
        let option = prompt(ops.join("\n").as_str());

        match option.parse::<usize>() {
            Ok(num) if num <= options.len() => return num-1,
            Ok(_) => println!("Invalid option selected, try again."),
            Err(_) => println!("Invalid option selected, try again.")
        }
    }
}

trait DataProvider {
    fn read_todos(path: &String) -> Result<Vec<Todo>, Box<dyn std::error::Error>>;

    fn write_todos(ts: &Vec<Todo>, path: &String) -> Result<(), Box<dyn std::error::Error>>;
}

struct FileProvider {}

impl DataProvider for FileProvider {
    fn read_todos(path: &String) -> Result<Vec<Todo>, Box<dyn std::error::Error>> {
        if !Path::new(path).exists() {
            File::create(path)?;
        }

        let file_content = std::fs::read_to_string(path).unwrap();

        let ts = serde_json::from_str(&file_content).unwrap_or(Vec::new());

        Ok(ts)
    }

    fn write_todos(ts: &Vec<Todo>, path: &String) -> Result<(), Box<dyn std::error::Error>> {
        std::fs::write(path, serde_json::to_string(&ts)?)?;
        Ok(())
    }
}

#[derive(Debug, Serialize, Deserialize, PartialEq, Clone)]
enum Status {
    Incomplete,
    Done
}

#[derive(Debug, Serialize, Deserialize, Clone)]
struct Todo {
    id: String,
    todo: String,
    status: Status
}

impl Todo {
    fn new(todo: String) -> Result<Self, &'static str> {
        if todo.len() == 0 {
            return Err("todo cannot be empty");
        }

        Ok(Self {
            id: Uuid::new_v4().to_string(),
            todo,
            status: Status::Incomplete
        })
    }
}

fn add_todo(todos: &mut Vec<Todo>) -> Result<&Vec<Todo>, Box<dyn std::error::Error>> {
    let response = prompt("Enter todo: ");

    let new_todo = Todo::new(response)?;
    // let mut todos = get_todos()?;
    todos.push(new_todo);
    // let result = write_file(&todos)?;
    Ok(todos)
}

fn complete_todo(id: &str, todos: &mut Vec<Todo>) -> Result<bool, Box<dyn std::error::Error>> {
    for t in todos {
        if t.id == id { 
            t.status = Status::Done;
            return Ok(true);
        } else { 
            t.status = Status::Incomplete;
        }
    }

    Err(Box::new(TodoError(String::from("todo not found"))))
}

fn get_unfinished_todos(ts: &Vec<Todo>) -> Result<Vec<Todo>, Error> {
    let mut unfinished: Vec<Todo> = Vec::new();

    for t in ts {
        if let Status::Incomplete = t.status {
            unfinished.push(t.clone());
        }
    }

    Ok(unfinished)
}

fn get_todo_to_complete(unfinished_todos: &Vec<Todo>) -> Result<String, Box<dyn std::error::Error>> {
    let todo_names: Vec<&str> = unfinished_todos.iter().map(|t| t.todo.as_str()).collect();

    let option = get_option(&todo_names, Some("Pick a todo to mark as complete"));

    Ok(unfinished_todos.get(option).unwrap().id.clone())
}

fn prompt(message: &str) -> String {
    println!("{}", message);
    println!("Press 0 to exit");
    let mut response = String::new();

    std::io::stdin()
        .read_line(&mut response)
        .expect("Error: could not read response");

    let input = response.trim();
    if input == "0" {
        std::process::exit(1);
    }

    input.to_string()
}

fn list_options(todo_count: usize) -> Vec<&'static str> {
    let options = vec!["add a new todo", "complete a todo"];
    if todo_count == 0 {
        vec![options[0]]
    }else {
        options
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn nonexistent_file() {
        let path = "/tmp/notfound";
        let file = FileProvider::read_todos(&String::from(path)).unwrap();
        
        assert!(Path::new(path).exists());
        assert_eq!(0, file.len());
    }

    #[test]
    fn list_options_zero() {
        assert_eq!(1, list_options(0).len());
    }

    #[test]
    fn list_options_one() {
        assert_eq!(2, list_options(1).len());
    }

    #[test]
    fn count_unfinished_todos() {
        let todos = vec![
            Todo {
                id: String::from("1"),
                todo: String::from("todo one"),
                status: Status::Done
            },
            Todo {
                id: String::from("2"),
                todo: String::from("todo two"),
                status: Status::Incomplete
            },
            Todo {
                id: String::from("3"),
                todo: String::from("todo three"),
                status: Status::Incomplete
            }
        ];

        assert_eq!(2, get_unfinished_todos(&todos).unwrap().len());
    }
}