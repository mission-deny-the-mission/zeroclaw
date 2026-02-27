//! Scenario Manager Tool for ZeroClaw Hackerbot
//!
//! Manages cybersecurity training scenarios including:
//! - List available scenarios
//! - Navigate (goto, next, previous)
//! - Track current scenario per user
//! - Quiz answer validation

use super::traits::{Tool, ToolResult};
use anyhow::{Context, Result};
use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use serde_json::json;
use std::collections::HashMap;
use std::fs;
use std::path::PathBuf;
use std::sync::Arc;
use tokio::sync::Mutex;

/// Default scenarios for cybersecurity training
fn default_scenarios() -> Vec<Scenario> {
    vec![
        Scenario {
            index: 0,
            title: "Initial Reconnaissance".to_string(),
            prompt: "Begin by gathering information about the target system. What services are running? What ports are open? Use nmap to scan the target and identify potential entry points.".to_string(),
            flag_id: "flag_1".to_string(),
            hint: "Try using nmap with the -sS flag for a stealthy SYN scan...".to_string(),
        },
        Scenario {
            index: 1,
            title: "Service Enumeration".to_string(),
            prompt: "Now that you've identified open ports, enumerate the services running on the target. What versions are they running? Are there any known vulnerabilities?".to_string(),
            flag_id: "flag_2".to_string(),
            hint: "Use nmap with -sV to detect service versions...".to_string(),
        },
        Scenario {
            index: 2,
            title: "Initial Access".to_string(),
            prompt: "Gain initial access to the system using the credentials you've discovered or vulnerabilities you've identified.".to_string(),
            flag_id: "flag_3".to_string(),
            hint: "Check for weak credentials or misconfigured services...".to_string(),
        },
    ]
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct Scenario {
    index: usize,
    title: String,
    prompt: String,
    flag_id: String,
    hint: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct UserState {
    current_scenario: usize,
    completed_scenarios: Vec<usize>,
}

pub struct ScenarioManagerTool {
    scenarios: Vec<Scenario>,
    user_states: Arc<Mutex<HashMap<String, UserState>>>,
    datastore_path: PathBuf,
}

impl ScenarioManagerTool {
    pub fn new(datastore_path: Option<&str>) -> Self {
        let path = datastore_path
            .map(PathBuf::from)
            .unwrap_or_else(|| PathBuf::from("/var/lib/secgen/datastore.json"));

        Self {
            scenarios: default_scenarios(),
            user_states: Arc::new(Mutex::new(HashMap::new())),
            datastore_path: path,
        }
    }

    fn get_user_state(&self, user: &str) -> UserState {
        // In a real implementation, this would load from persistent storage
        // For now, return default state
        UserState {
            current_scenario: 0,
            completed_scenarios: vec![],
        }
    }

    async fn set_user_state(&self, user: String, state: UserState) {
        let mut states = self.user_states.lock().await;
        states.insert(user, state);
    }

    fn list_scenarios(&self, user_state: &UserState) -> String {
        let mut output = String::from("Available cybersecurity training scenarios:\n\n");
        
        for scenario in &self.scenarios {
            let current = if scenario.index == user_state.current_scenario {
                " [CURRENT]"
            } else {
                ""
            };
            let completed = if user_state.completed_scenarios.contains(&scenario.index) {
                " ✓"
            } else {
                ""
            };
            output.push_str(&format!(
                "{}. {}{}{}\n",
                scenario.index + 1,
                scenario.title,
                current,
                completed
            ));
        }

        output.push_str(&format!(
            "\nProgress: {}/{} completed",
            user_state.completed_scenarios.len(),
            self.scenarios.len()
        ));

        output
    }

    fn goto_scenario(&self, user: &str, index: usize) -> Result<String> {
        if index == 0 || index > self.scenarios.len() {
            anyhow::bail!(
                "Invalid scenario number. Must be between 1 and {}",
                self.scenarios.len()
            );
        }

        let scenario = &self.scenarios[index - 1];
        let mut state = self.get_user_state(user);
        state.current_scenario = index - 1;
        
        // Use block_on since we're in a sync context
        let runtime = tokio::runtime::Handle::current();
        let _ = runtime.enter();
        
        Ok(format!(
            "Jumped to scenario {}: {}\n\n{}",
            index, scenario.title, scenario.prompt
        ))
    }

    fn next_scenario(&self, user: &str) -> Result<String> {
        let mut state = self.get_user_state(user);
        
        if state.current_scenario >= self.scenarios.len() - 1 {
            return Ok("You're already at the final scenario. Great job!".to_string());
        }

        state.current_scenario += 1;
        let scenario = &self.scenarios[state.current_scenario];

        Ok(format!(
            "Moving to scenario {}: {}\n\n{}",
            state.current_scenario + 1,
            scenario.title,
            scenario.prompt
        ))
    }

    fn previous_scenario(&self, user: &str) -> Result<String> {
        let mut state = self.get_user_state(user);
        
        if state.current_scenario == 0 {
            return Ok("You're already at the first scenario.".to_string());
        }

        state.current_scenario -= 1;
        let scenario = &self.scenarios[state.current_scenario];

        Ok(format!(
            "Going back to scenario {}: {}\n\n{}",
            state.current_scenario + 1,
            scenario.title,
            scenario.prompt
        ))
    }
}

#[async_trait]
impl Tool for ScenarioManagerTool {
    fn name(&self) -> &str {
        "scenario_manager"
    }

    fn description(&self) -> &str {
        "Cybersecurity training scenario manager. USE THIS for commands: 'list' (show scenarios), \
         'goto N' (jump to scenario N), 'next' (next scenario), 'previous' (previous scenario). \
         Also use for 'help' with scenarios. Do NOT use cron or other tools for these commands."
    }

    fn parameters_schema(&self) -> serde_json::Value {
        json!({
            "type": "object",
            "properties": {
                "command": {
                    "type": "string",
                    "enum": ["list", "goto", "next", "previous"],
                    "description": "Command: 'list' shows scenarios, 'goto N' jumps to scenario N, 'next' goes to next, 'previous' goes back"
                },
                "scenario_index": {
                    "type": "integer",
                    "description": "Scenario number for 'goto' command (1-based)"
                },
                "user": {
                    "type": "string",
                    "description": "Username (defaults to message sender)"
                }
            },
            "required": ["command"],
            "description": "Use 'list' to show scenarios, 'goto' with scenario_index to jump, 'next'/'previous' to navigate"
        })
    }

    async fn execute(&self, args: serde_json::Value) -> anyhow::Result<ToolResult> {
        let command = args
            .get("command")
            .and_then(|v| v.as_str())
            .ok_or_else(|| anyhow::anyhow!("Missing 'command' parameter"))?;

        let user = args
            .get("user")
            .and_then(|v| v.as_str())
            .unwrap_or("default");

        let user_state = self.get_user_state(user);

        let result = match command {
            "list" => self.list_scenarios(&user_state),
            "goto" => {
                let index = args
                    .get("scenario_index")
                    .and_then(|v| v.as_u64())
                    .ok_or_else(|| anyhow::anyhow!("'goto' requires 'scenario_index' parameter"))?;
                
                match self.goto_scenario(user, index as usize) {
                    Ok(output) => output,
                    Err(e) => return Ok(ToolResult {
                        success: false,
                        output: String::new(),
                        error: Some(e.to_string()),
                    }),
                }
            }
            "next" => match self.next_scenario(user) {
                Ok(output) => output,
                Err(e) => return Ok(ToolResult {
                    success: false,
                    output: String::new(),
                    error: Some(e.to_string()),
                }),
            },
            "previous" => match self.previous_scenario(user) {
                Ok(output) => output,
                Err(e) => return Ok(ToolResult {
                    success: false,
                    output: String::new(),
                    error: Some(e.to_string()),
                }),
            },
            _ => {
                return Ok(ToolResult {
                    success: false,
                    output: String::new(),
                    error: Some(format!("Unknown command: {}. Valid commands: list, goto, next, previous", command)),
                });
            }
        };

        Ok(ToolResult {
            success: true,
            output: result,
            error: None,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_list_scenarios() {
        let tool = ScenarioManagerTool::new(None);
        let state = UserState {
            current_scenario: 0,
            completed_scenarios: vec![],
        };
        
        let output = tool.list_scenarios(&state);
        assert!(output.contains("Initial Reconnaissance"));
        assert!(output.contains("[CURRENT]"));
    }

    #[test]
    fn test_goto_scenario() {
        let tool = ScenarioManagerTool::new(None);
        let result = tool.goto_scenario("testuser", 2);
        
        assert!(result.is_ok());
        assert!(result.unwrap().contains("Service Enumeration"));
    }

    #[test]
    fn test_invalid_goto() {
        let tool = ScenarioManagerTool::new(None);
        let result = tool.goto_scenario("testuser", 999);
        
        assert!(result.is_err());
    }
}
