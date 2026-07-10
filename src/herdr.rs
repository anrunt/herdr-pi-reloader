use std::{time::Duration};

use serde::{Deserialize};
use shell_escape::unix::escape;
use tokio::time::{self, timeout};

#[derive(Debug)]
pub struct ReloadSummary {
    reloaded: usize,
    skipped_non_pi: usize,
    skipped_unsafe_status: usize,
    skipped_invalid_agent_data: usize,
    failed: usize
}

#[derive(Debug)]
pub struct ResetCandidatesSummary {
   candidates: usize,
   skipped_non_pi: usize,
   skipped_unsafe_status: usize,
   skipped_invalid_agent_data: usize,
   skipped_missing_session: usize,
   skipped_invalid_session: usize,
}

#[derive(Deserialize)]
struct AgentSession {
    agent: String,
    kind: String,
    value: String
}

#[derive(Deserialize)]
pub struct AgentInfo {
    agent: String,
    agent_status: String,
    pane_id: String,
    agent_session: Option<AgentSession>
}
#[derive(Debug)]
pub struct ResetCandidate {
    pane_id: String,
    session_path: String
}

#[derive(Deserialize)]
struct AgentListResult {
    agents: Vec<AgentInfo>
}

#[derive(Deserialize)]
struct AgentListResponse {
    result: AgentListResult
}

pub async fn run_in_pane(herdr_path: &str, pane_id: &str, command: &str) -> Result<(), String> {
    match tokio::process::Command::new(herdr_path).args(["pane", "run", pane_id, command]).output().await {
        Ok(output) => {
            if output.status.success() {
                println!("Successfully executed command: {command} on pane_id: {pane_id}");
                Ok(())
            } else {
                let std_error = String::from_utf8_lossy(&output.stderr);
                let std_out = String::from_utf8_lossy(&output.stdout);

                let error_message = format!("std_err: {std_error} - std_out: {std_out}");
                let error_str = format!("Error in executing command: {}: on pane: {} - error: {}", command, pane_id, error_message);
                Err(error_str)
            }
        },
        Err(error) => {
            let error_str = format!("Error in executing command: {}: on pane: {} - error: {}", command, pane_id, error);
            Err(error_str)
        }
    }

}

pub async fn get_agent_list(herdr_path: &str) -> Result<Vec<AgentInfo>, String> {
    match tokio::process::Command::new(herdr_path).args(["agent", "list"]).output().await {
        Ok(output) => {
            if output.status.success() {
                let output_str = String::from_utf8_lossy(&output.stdout);

                let json_result: Result<AgentListResponse, serde_json::Error> =
                    serde_json::from_str(&output_str);

                match json_result {
                    Ok(value) => {
                        let agents = value.result.agents;
                        Ok(agents)
                    },
                    Err(error) => {
                        let error_text = format!("Error with parsing json: {}", error);
                        Err(error_text)
                    }
                }
            } else {
                let error_str = String::from_utf8_lossy(&output.stderr).to_string();
                let error_text = format!("Herdr output error: {}", error_str);
                Err(error_text)
            }

        },
        Err(error) => {
            let error_str = error.to_string();
            let error_text = format!("Failed to run herdr agent list: {}", error_str);
            Err(error_text)
        }
    }
}

pub fn get_reset_candidates(agent_list: &[AgentInfo]) -> (Vec<ResetCandidate>, ResetCandidatesSummary) {
    let mut reset_candidates_summary = ResetCandidatesSummary {
        candidates: 0,
        skipped_non_pi: 0,
        skipped_unsafe_status: 0,
        skipped_invalid_agent_data: 0,
        skipped_missing_session: 0,
        skipped_invalid_session: 0
    };

    let mut reset_candidates: Vec<ResetCandidate> = Vec::new();

    for value in agent_list {
        if value.agent.is_empty() {
            reset_candidates_summary.skipped_invalid_agent_data += 1;
            continue;
        }

        if value.pane_id.is_empty() {
            reset_candidates_summary.skipped_invalid_agent_data += 1;
            continue;
        }

        if value.agent_status.is_empty() {
            reset_candidates_summary.skipped_invalid_agent_data += 1;
            continue;
        }

        if value.agent != "pi" {
            reset_candidates_summary.skipped_non_pi += 1;
            continue;
        }

        if value.agent_status != "done" && value.agent_status != "idle" {
            reset_candidates_summary.skipped_unsafe_status += 1;
            continue;
        }

        match &value.agent_session {
            Some(session) =>{
                if &session.agent != "pi" {
                    reset_candidates_summary.skipped_invalid_session += 1;
                    continue;
                }

                if &session.kind != "path" {
                    reset_candidates_summary.skipped_invalid_session += 1;
                    continue;
                }

                if session.value.is_empty() {
                    reset_candidates_summary.skipped_invalid_session += 1;
                    continue;
                }

                let reset_candidate = ResetCandidate {
                    pane_id: value.pane_id.clone(),
                    session_path: session.value.clone()
                };

                reset_candidates.push(reset_candidate);
                reset_candidates_summary.candidates += 1;
            },
            None => {
                reset_candidates_summary.skipped_missing_session += 1;
                continue;
            }
        }

    }

    (reset_candidates, reset_candidates_summary)
}

async fn wait_until_pi_exits(herdr_path: &str, pane_id: &str) -> Result<(), String> {
    let res = timeout(Duration::from_secs(15), async {
        loop {
            let agents = get_agent_list(herdr_path).await;

            match agents {
                Ok(agents) => {
                    let found_pi = agents.iter().any(|value| value.agent == "pi" && value.pane_id == pane_id);

                    if found_pi {
                        let sleep_time = time::Duration::from_millis(500);
                        tokio::time::sleep(sleep_time).await;
                    } else {
                        return Ok(());
                    }

                },
                Err(error) => {
                    return Err(error);
                }
            }
        }
    }).await;

    match res {
        Ok(agent_result) => agent_result,
        Err(error) => {
            let error_str = format!("Error, timeout reached for pane_id: {} - error: {}", pane_id, error);
            Err(error_str)
        },
    }
}

pub async fn reset_one_candidate(herdr_path: &str, candidate: &ResetCandidate) -> Result<(), String> {
    println!("Resetting pane: {}", candidate.pane_id);

    let quit_result = run_in_pane(herdr_path, candidate.pane_id.as_str(), "/quit").await;

    match quit_result {
        Ok(_) => (),
        Err(error) => {
            let error_str = format!("Error with resetting pane: {} - error: {}", candidate.pane_id, error);
            return Err(error_str);
        }
    }

    let wait_result = wait_until_pi_exits(herdr_path, &candidate.pane_id).await;

    match wait_result {
        Ok(_) => (),
        Err(error) => {
            let error_str = format!("Error with exiting pi on pane: {} - error: {}", candidate.pane_id, error);
            return Err(error_str);
        }
    }

    let safe_session_path = escape(std::borrow::Cow::Borrowed(&candidate.session_path));
    let start_command = format!("pi --session {}", safe_session_path);

    let start_result = run_in_pane(herdr_path, candidate.pane_id.as_str(), &start_command).await;

    match start_result {
        Ok(_) => Ok(()),
        Err(error) => {
            let error_str = format!("Error with starting pi on pane: {} - error: {}", candidate.pane_id, error);
            Err(error_str)
        }
    }
}

pub async fn reload_all_pi(herdr_path: &str, agents: &[AgentInfo]) {
    let mut reload_summary = ReloadSummary {
        reloaded: 0,
        skipped_non_pi: 0,
        skipped_unsafe_status: 0,
        skipped_invalid_agent_data: 0,
        failed: 0,
    };

    for (index, agent) in agents.iter().enumerate() {
        println!("Agent nr: {}", index);

        let pane_id = agent.pane_id.as_str();
        let agent_name = agent.agent.as_str();
        let agent_status = agent.agent_status.as_str();

        let required_fields = [
            ("pane_id", pane_id),
            ("agent_name", agent_name),
            ("agent_status", agent_status),
        ];

        let mut invalid_agent_data = false;
        for (name, value) in required_fields {
            if value.is_empty() {
                println!("Missing {} value!", name);
                invalid_agent_data = true;
            }
        }

        if invalid_agent_data {
            reload_summary.skipped_invalid_agent_data += 1;
            continue;
        }

        if agent_name != "pi" {
            reload_summary.skipped_non_pi += 1;
            println!("Not pi - skipping");
            continue;
        }

        if agent_status == "done" || agent_status == "idle" {
            println!("Reloading pi on pane: {}", pane_id);

            let reload_pane_status = run_in_pane(herdr_path, pane_id, "/reload").await;

            match reload_pane_status {
                Ok(_) => reload_summary.reloaded += 1,
                Err(error) => {
                    reload_summary.failed += 1;
                    println!("{}", error);
                }
            }
        } else {
            reload_summary.skipped_unsafe_status += 1;
            println!("Agent on pane: {} is {} - skipping", pane_id, agent_status);
            continue;
        }
    }
    println!("{:#?}", reload_summary);
}
