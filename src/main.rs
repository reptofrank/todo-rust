use std::env;
use todo_rust::{run, Config};

const PATH: &'static str = "todos.json";

fn main() {
    let path = match env::var("FILE_PATH") {
        Ok(val) => val,
        Err(_) => String::from(PATH)
    };

    match run(Config { path }) {
        Ok(_) => println!("done"),
        Err(e) => println!("{}", e)
    }
    
}
