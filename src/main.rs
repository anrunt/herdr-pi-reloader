use std::{env, process::Command};

use serde_json::Value;
use serde::{Deserialize};

#[derive(Debug)]
struct ReloadSummary {
    reloaded: usize,
    skipped_non_pi: usize,
    skipped_unsafe_status: usize,
    skipped_invalid_agent_data: usize,
    failed: usize
}

#[derive(Deserialize)]
struct AgentInfo {
    agent: String,
    agent_status: String,
    pane_id: String
}

#[derive(Deserialize)]
struct AgentListResult {
    agents: Vec<AgentInfo>
}

fn reload_pane(herdr_path: &str, pane_id: &str) -> bool {
    match Command::new(herdr_path).args(["pane", "run", pane_id, "/reload"]).output() {
        Ok(output) => {
            if output.status.success() {
                println!("Successfully reloaded pane: {}", pane_id);
                return true;
            } else {
                let std_error = String::from_utf8_lossy(&output.stderr);
                let std_out = String::from_utf8_lossy(&output.stdout);
                println!("Pane run exited with error for pane: {} - status: {} - error: {} - output: {}", pane_id, output.status, std_error, std_out);
                return false;
            }
        },
        Err(error) => {
            println!("Error occurred when reloading pane: {} - error: {}", pane_id, error);
            return false;
        }
    }
}

fn main() {
    let args: Vec<String> = env::args().skip(1).collect();

    if args.is_empty() {
        println!("Usage: cargo run -- reload");
        return;
    }

    if args[0] != "reload" {
        println!("Error: Unknown command");
        return;
    }

    let herdr_path_result = env::var("HERDR_BIN_PATH");

    let herdr_path = match herdr_path_result {
        Ok(path) => path,
        Err(_error) => String::from("herdr"),
    };

    match Command::new(&herdr_path).args(["agent", "list"]).output() {
        Ok(output) => {
            if output.status.success() {
                let output_str = String::from_utf8_lossy(&output.stdout);

                let json_result: Result<Value, serde_json::Error> =
                    serde_json::from_str(&output_str);

                match json_result {
                    Ok(value) => {
                        if let Some(agents) = value["result"]["agents"].as_array() {
                            let mut reload_summary = ReloadSummary {
                                reloaded: 0,
                                skipped_non_pi: 0,
                                skipped_unsafe_status: 0,
                                skipped_invalid_agent_data: 0,
                                failed: 0
                            };

                            for (index, agent) in agents.iter().enumerate() {
                                println!("Agent nr: {}", index);

                                let pane_id_option = agent.get("pane_id").and_then(|v| v.as_str());
                                let agent_name_option = agent.get("agent").and_then(|v| v.as_str());
                                let agent_status_option = agent.get("agent_status").and_then(|v| v.as_str());

                                if let (Some(pane_id), Some(agent_name), Some(agent_status)) =
                                    (pane_id_option, agent_name_option, agent_status_option)
                                {
                                    if agent_name != "pi" {
                                        reload_summary.skipped_non_pi += 1;
                                        println!("Not pi - skipping");
                                        continue;
                                    }

                                    if agent_status == "done" || agent_status == "idle" {
                                        println!("Reloading pi on pane: {}", pane_id);

                                        let reload_pane_status = reload_pane(&herdr_path, pane_id);

                                        if reload_pane_status {
                                            reload_summary.reloaded += 1;
                                        } else {
                                            reload_summary.failed += 1;
                                        }

                                    } else {
                                        reload_summary.skipped_unsafe_status += 1;
                                        println!("Agent on pane: {} is {} - skipping", pane_id, agent_status);
                                        continue;
                                    }

                                } else {
                                    let required_fields = [
                                        ("pane_id", pane_id_option),
                                        ("agent_name", agent_name_option),
                                        ("agent_status", agent_status_option)
                                    ];

                                    for (field_name, field_value) in required_fields {
                                        if field_value.is_none() {
                                            println!("Missing {} value!", field_name);
                                        }    
                                    }

                                    reload_summary.skipped_invalid_agent_data += 1;
                                }
                            }
                            println!("{:#?}", reload_summary);
                        } else {
                            println!("Error - agents not array");
                        }
                    }
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
        Err(error) => println!("Failed to run herdr agent list: {}", error),
    }
}
