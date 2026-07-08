mod herdr;

use std::{env};

use crate::herdr::{get_agent_list, get_reset_candidates, reload_all_pi};

#[tokio::main]
async fn main() {
    let args: Vec<String> = env::args().skip(1).collect();

    if args.is_empty() {
        println!("Usage: cargo run -- reload");
        return;
    }

    if args[0] != "reload" && args[0] != "reset" {
        println!("Error: Unknown command");
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

    if args[0] == "reload" {
        reload_all_pi(&herdr_path, &agents).await;
        return;
    }

    if args[0] == "reset" {
        let (candidates, summary) = get_reset_candidates(&agents);

        println!("Candidates: {:#?}", candidates);
        println!("Summary: {:#?}", summary);

        return;
    }
}
