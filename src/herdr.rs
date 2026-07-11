use serde::{Deserialize};

#[derive(Deserialize)]
pub(crate) struct AgentSession {
    pub(crate) agent: String,
    pub(crate) kind: String,
    pub(crate) value: String
}

#[derive(Deserialize)]
pub struct AgentInfo {
    pub(crate) agent: String,
    pub(crate) agent_status: String,
    pub(crate) pane_id: String,
    pub(crate) agent_session: Option<AgentSession>
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
