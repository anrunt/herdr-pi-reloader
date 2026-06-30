use std::{env, process::{Command}};

fn main() {
    let args: Vec<String> = env::args().skip(1).collect();

    if args.is_empty() {
        println!("Usage: cargo run reload");
        return;
    }

    if args[0] != "reload" {
        println!("Error: Unknown command");
        return;
    }

    println!("Correct command!");

    let herdr_path_result = env::var("HERDR_BIN_PATH");

    let herdr_path = match herdr_path_result {
      Ok(path) => path,
      Err(_error) => String::from("herdr"),
    };

    println!("Path: {}", herdr_path);

    match Command::new(herdr_path).args(["agent", "list"]).output() {
      Ok(output) =>  {
        if output.status.success() {
          println!("Herdr output: {}", String::from_utf8_lossy(&output.stdout));
        } else {
          println!("Herdr output error: {}", String::from_utf8_lossy(&output.stderr));
        }
      }
      Err(error) => println!("Unexpected error: {}", error)
    }
}
