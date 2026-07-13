mod herdr;
mod pi;
mod render;
mod tui;

use std::{env};

use tokio::task::JoinHandle;

use crate::herdr::get_agent_list;
use crate::pi::{get_reset_candidates, reload_all_pi, reset_one_candidate};
use crate::tui::run;

#[tokio::main]
async fn main() {
    let args: Vec<String> = env::args().skip(1).collect();

    if args.is_empty() {
        println!("Usage: cargo run -- reload / reset / tui");
        return;
    }

    let command = &args[0];

    if command != "reload" && command != "reset" && command != "tui" {
        println!("Error: Unknown command");
        return;
    }

    if command == "tui" {
        let tui_result = run().await;

        match tui_result {
            Ok(_) => (),
            Err(error) => {
                eprintln!("Error: {}", error);
            }
        }

        return;
    }

    let herdr_path_result = env::var("HERDR_BIN_PATH");

    let herdr_path = match herdr_path_result {
        Ok(path) => path,
        Err(_error) => String::from("herdr"),
    };

    let agents = match get_agent_list(&herdr_path).await {
        Ok(value) => value,
        Err(error) => {
            println!("{}", error);
            return;
        }
    };

    if command == "reload" {
        let reload_summary = reload_all_pi(&herdr_path, &agents).await;
        println!("{:#?}", reload_summary);
        return;
    }

    if command == "reset" {
        let (candidates, summary) = get_reset_candidates(&agents);

        println!("Candidates: {:#?}", candidates);
        println!("Summary: {:#?}", summary);

        let reset_tasks: Vec<JoinHandle<Result<(), String>>> = candidates.into_iter().map(|v| {
            let herdr_path_c = herdr_path.clone();
            tokio::spawn(async move {
                let res = reset_one_candidate(&herdr_path_c, &v).await;
                return res;
            })
        }).collect();

        for task in reset_tasks {
            let task_res = task.await;
            match task_res {
                Ok(res) => match res {
                    Ok(_) => (),
                    Err(error) => println!("{}", error),
                },
                Err(error) => println!("Join error: {}", error),
            }
        }
    }
}
