use std::{env, process::Command};

use serde_json::Value;

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
        Ok(output) => {
            if output.status.success() {
                let output_str = String::from_utf8_lossy(&output.stdout);
                println!("Herdr output: {}", output_str);

                let json_result: Result<Value, serde_json::Error> =
                    serde_json::from_str(&output_str);

                match json_result {
                    Ok(value) => {
                        if let Some(agents) = value["result"]["agents"].as_array() {
                            for (index, agent) in agents.iter().enumerate() {
                                println!("Agent nr: {}", index);

                                // unwrap-or will probably get deleted because we need this values
                                // and we want to throw error if some of the are missing instead of
                                // replacing missing value with <missing>
                                let pane_id = agent.get("pane_id").and_then(|v| v.as_str()).unwrap_or("<missing>");
                                println!("Pane_id: {}", pane_id);

                                let agent_name = agent.get("agent").and_then(|v| v.as_str()).unwrap_or("<missing>");
                                println!("Agent_name: {}", agent_name);

                                let agent_status = agent.get("agent_status").and_then(|v| v.as_str()).unwrap_or("<missing>");
                                println!("Agent_status: {}", agent_status);
                            }
                        } else {
                            println!("Error - agents not array");
                        }
                    },
                    Err(error) => {
                        println!("Error with parsing json: {}", error);
                    }
                }
            } else {
                println!(
                    "Herdr output error: {}",
                    String::from_utf8_lossy(&output.stderr)
                );
            }
        }
        Err(error) => println!("Unexpected error: {}", error),
    }
}
