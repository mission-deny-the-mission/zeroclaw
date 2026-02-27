//! SecGen Flag Validator Tool
//!
//! This tool validates CTF flags against the SecGen datastore and tracks
//! scenario completion for student progress tracking.
//!
//! # Usage
//!
//! ```json
//! {
//!   "name": "secgen_flag_validator",
//!   "arguments": {
//!     "flag": "SEC GEN{abc123}",
//!     "scenario_id": "hacker_vs_hackerbot_1",
//!     "username": "student1"
//!   }
//! }
//! ```
//!
//! # Response
//!
//! Success:
//! ```json
//! {
//!   "success": true,
//!   "output": "Flag validated successfully! +100 points",
//!   "error": null
//! }
//! ```
//!
//! Failure:
//! ```json
//! {
//!   "success": false,
//!   "output": "",
//!   "error": "Invalid flag. Please try again."
//! }
//! ```

use crate::tools::traits::{Tool, ToolResult};
use anyhow::{Context, Result};
use async_trait::async_trait;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use serde_json::json;
use std::collections::HashMap;
use std::fs::{self, OpenOptions};
use std::io::Write;
use std::path::PathBuf;

/// Default SecGen datastore path
const DEFAULT_DATASTORE_PATH: &str = "/var/lib/secgen/datastore.json";

/// Default progress tracking path
const DEFAULT_PROGRESS_PATH: &str = "/var/lib/zeroclaw/progress.json";

/// SecGen flag validator tool
pub struct SecGenFlagValidatorTool {
    datastore_path: PathBuf,
    progress_path: PathBuf,
    enabled: bool,
}

/// Flag validation parameters
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FlagValidation {
    /// Flag value to validate
    pub flag: String,
    /// Scenario identifier
    pub scenario_id: String,
    /// Student username (optional, for progress tracking)
    pub username: Option<String>,
}

/// Progress tracking entry
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProgressEntry {
    pub username: String,
    pub scenario_id: String,
    pub flag_id: String,
    pub validated_at: DateTime<Utc>,
    pub points: u32,
}

/// Progress tracking data
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct ProgressData {
    pub entries: Vec<ProgressEntry>,
    pub total_points: HashMap<String, u32>,
}

impl SecGenFlagValidatorTool {
    pub fn new(
        datastore_path: Option<&str>,
        progress_path: Option<&str>,
        enabled: bool,
    ) -> Self {
        let datastore = datastore_path
            .map(PathBuf::from)
            .unwrap_or_else(|| PathBuf::from(DEFAULT_DATASTORE_PATH));

        let progress = progress_path
            .map(PathBuf::from)
            .unwrap_or_else(|| PathBuf::from(DEFAULT_PROGRESS_PATH));

        Self {
            datastore_path: datastore,
            progress_path: progress,
            enabled,
        }
    }

    /// Read the SecGen datastore JSON file
    fn read_datastore(&self) -> Result<HashMap<String, serde_json::Value>> {
        if !self.enabled {
            anyhow::bail!("SecGen flag validator is disabled");
        }

        if !self.datastore_path.exists() {
            anyhow::bail!(
                "SecGen datastore not found at {}",
                self.datastore_path.display()
            );
        }

        let content = fs::read_to_string(&self.datastore_path)
            .with_context(|| format!("Failed to read datastore at {:?}", self.datastore_path))?;

        let data: HashMap<String, serde_json::Value> = serde_json::from_str(&content)
            .with_context(|| "Failed to parse SecGen datastore JSON")?;

        Ok(data)
    }

    /// Read progress tracking data
    fn read_progress(&self) -> Result<ProgressData> {
        if !self.progress_path.exists() {
            return Ok(ProgressData::default());
        }

        let content = fs::read_to_string(&self.progress_path)
            .with_context(|| format!("Failed to read progress file at {:?}", self.progress_path))?;

        let data: ProgressData = serde_json::from_str(&content)
            .with_context(|| "Failed to parse progress JSON")?;

        Ok(data)
    }

    /// Write progress tracking data
    fn write_progress(&self, data: &ProgressData) -> Result<()> {
        // Ensure directory exists
        if let Some(parent) = self.progress_path.parent() {
            fs::create_dir_all(parent)
                .with_context(|| format!("Failed to create directory for {:?}", self.progress_path))?;
        }

        let content = serde_json::to_string_pretty(data)
            .with_context(|| "Failed to serialize progress data")?;

        let mut file = OpenOptions::new()
            .write(true)
            .create(true)
            .truncate(true)
            .open(&self.progress_path)
            .with_context(|| format!("Failed to open progress file at {:?}", self.progress_path))?;

        file.write_all(content.as_bytes())
            .with_context(|| "Failed to write progress file")?;

        Ok(())
    }

    /// Validate a flag against the SecGen datastore
    fn validate_flag(&self, validation: &FlagValidation) -> Result<FlagValidationResult> {
        let data = self.read_datastore()?;

        // Look for flags in datastore
        let flags_value = data.get("flags").with_context(|| {
            "No 'flags' key found in datastore. Ensure SecGen generated flags for this scenario."
        })?;

        let flags_array = flags_value
            .as_array()
            .with_context(|| "'flags' key is not an array")?;

        // Try to find matching flag
        let mut found_flag: Option<&serde_json::Value> = None;
        let mut flag_id: Option<String> = None;
        let mut points: u32 = 100; // Default points

        for (index, flag_value) in flags_array.iter().enumerate() {
            let flag_str = if let Some(s) = flag_value.as_str() {
                s.to_string()
            } else if let Some(obj) = flag_value.as_object() {
                // Handle object format: {"id": "flag_1", "value": "SEC GEN{...}", "points": 100}
                let id = obj
                    .get("id")
                    .and_then(|v| v.as_str())
                    .unwrap_or(&format!("flag_{}", index + 1))
                    .to_string();

                let value = obj
                    .get("value")
                    .and_then(|v| v.as_str())
                    .unwrap_or("")
                    .to_string();

                let p = obj
                    .get("points")
                    .and_then(|v| v.as_u64())
                    .unwrap_or(100) as u32;

                if value == validation.flag {
                    let flag_id_str = id.clone();
                    flag_id = Some(id);
                    points = p;
                    return Ok(FlagValidationResult {
                        valid: true,
                        flag_id: Some(flag_id_str),
                        points: p,
                        message: format!("Flag validated successfully! +{} points", p),
                        already_submitted: false,
                    });
                }
                continue;
            } else {
                flag_value.to_string()
            };

            if flag_str == validation.flag {
                found_flag = Some(flag_value);
                flag_id = Some(format!("flag_{}", index + 1));
                break;
            }
        }

        if found_flag.is_none() {
            return Ok(FlagValidationResult {
                valid: false,
                flag_id: None,
                points: 0,
                message: "Invalid flag. Please try again.".to_string(),
                already_submitted: false,
            });
        }

        // Check if already submitted by this user
        let already_submitted = if let Some(username) = &validation.username {
            let progress = self.read_progress()?;
            progress.entries.iter().any(|entry| {
                entry.username == *username
                    && entry.scenario_id == validation.scenario_id
                    && entry.flag_id == *flag_id.as_ref().unwrap_or(&String::new())
            })
        } else {
            false
        };

        if already_submitted {
            return Ok(FlagValidationResult {
                valid: true,
                flag_id,
                points: 0,
                message: "Flag already submitted by this user.".to_string(),
                already_submitted: true,
            });
        }

        // Record progress if username provided
        if let Some(username) = &validation.username {
            let mut progress = self.read_progress()?;

            progress.entries.push(ProgressEntry {
                username: username.clone(),
                scenario_id: validation.scenario_id.clone(),
                flag_id: flag_id.clone().unwrap_or_default(),
                validated_at: Utc::now(),
                points,
            });

            *progress
                .total_points
                .entry(username.clone())
                .or_insert(0) += points;

            self.write_progress(&progress)?;
        }

        Ok(FlagValidationResult {
            valid: true,
            flag_id,
            points,
            message: format!("Flag validated successfully! +{} points", points),
            already_submitted: false,
        })
    }
}

/// Flag validation result
#[derive(Debug, Clone)]
pub struct FlagValidationResult {
    pub valid: bool,
    pub flag_id: Option<String>,
    pub points: u32,
    pub message: String,
    pub already_submitted: bool,
}

#[async_trait]
impl Tool for SecGenFlagValidatorTool {
    fn name(&self) -> &str {
        "secgen_flag_validator"
    }

    fn description(&self) -> &str {
        "Validate CTF flags against SecGen datastore and track scenario completion. \
         Use this to verify student flag submissions and record progress."
    }

    fn parameters_schema(&self) -> serde_json::Value {
        json!({
            "type": "object",
            "properties": {
                "flag": {
                    "type": "string",
                    "description": "Flag value to validate (e.g., 'SEC GEN{abc123}')"
                },
                "scenario_id": {
                    "type": "string",
                    "description": "Scenario identifier (e.g., 'hacker_vs_hackerbot_1')"
                },
                "username": {
                    "type": "string",
                    "description": "Student username for progress tracking. Optional."
                }
            },
            "required": ["flag", "scenario_id"]
        })
    }

    async fn execute(&self, args: serde_json::Value) -> anyhow::Result<ToolResult> {
        // Parse arguments
        let validation: FlagValidation = serde_json::from_value(args)
            .with_context(|| "Failed to parse secgen_flag_validator arguments")?;

        // Validate arguments
        if validation.flag.is_empty() {
            return Ok(ToolResult {
                success: false,
                output: String::new(),
                error: Some("'flag' parameter cannot be empty".to_string()),
            });
        }

        if validation.scenario_id.is_empty() {
            return Ok(ToolResult {
                success: false,
                output: String::new(),
                error: Some("'scenario_id' parameter cannot be empty".to_string()),
            });
        }

        // Validate flag
        match self.validate_flag(&validation) {
            Ok(result) => {
                if result.valid && !result.already_submitted {
                    Ok(ToolResult {
                        success: true,
                        output: result.message,
                        error: None,
                    })
                } else if result.already_submitted {
                    Ok(ToolResult {
                        success: true,
                        output: result.message,
                        error: None,
                    })
                } else {
                    Ok(ToolResult {
                        success: false,
                        output: String::new(),
                        error: Some(result.message),
                    })
                }
            }
            Err(e) => Ok(ToolResult {
                success: false,
                output: String::new(),
                error: Some(format!("Flag validation failed: {}", e)),
            }),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs::File;
    use std::io::Write;
    use tempfile::TempDir;

    fn create_test_datastore(dir: &TempDir, content: &str) -> PathBuf {
        let path = dir.path().join("datastore.json");
        let mut file = File::create(&path).unwrap();
        file.write_all(content.as_bytes()).unwrap();
        path
    }

    #[test]
    fn test_validate_valid_flag() {
        let dir = TempDir::new().unwrap();
        let datastore_path = create_test_datastore(
            &dir,
            r#"{
                "flags": [
                    "SEC GEN{flag1}",
                    "SEC GEN{flag2}",
                    "SEC GEN{flag3}"
                ]
            }"#,
        );

        let tool = SecGenFlagValidatorTool::new(Some(datastore_path.to_str().unwrap()), None, true);

        let result = tool
            .validate_flag(&FlagValidation {
                flag: "SEC GEN{flag1}".to_string(),
                scenario_id: "test_scenario".to_string(),
                username: None,
            })
            .unwrap();

        assert!(result.valid);
        assert_eq!(result.points, 100);
        assert!(!result.already_submitted);
    }

    #[test]
    fn test_validate_invalid_flag() {
        let dir = TempDir::new().unwrap();
        let datastore_path = create_test_datastore(
            &dir,
            r#"{
                "flags": ["SEC GEN{flag1}"]
            }"#,
        );

        let tool = SecGenFlagValidatorTool::new(Some(datastore_path.to_str().unwrap()), None, true);

        let result = tool
            .validate_flag(&FlagValidation {
                flag: "SEC GEN{wrong}".to_string(),
                scenario_id: "test_scenario".to_string(),
                username: None,
            })
            .unwrap();

        assert!(!result.valid);
        assert!(result.message.contains("Invalid flag"));
    }

    #[test]
    fn test_validate_flag_object_format() {
        let dir = TempDir::new().unwrap();
        let datastore_path = create_test_datastore(
            &dir,
            r#"{
                "flags": [
                    {"id": "flag_1", "value": "SEC GEN{obj1}", "points": 150},
                    {"id": "flag_2", "value": "SEC GEN{obj2}", "points": 200}
                ]
            }"#,
        );

        let tool = SecGenFlagValidatorTool::new(Some(datastore_path.to_str().unwrap()), None, true);

        let result = tool
            .validate_flag(&FlagValidation {
                flag: "SEC GEN{obj1}".to_string(),
                scenario_id: "test_scenario".to_string(),
                username: None,
            })
            .unwrap();

        assert!(result.valid);
        assert_eq!(result.points, 150);
        assert_eq!(result.flag_id, Some("flag_1".to_string()));
    }
}
