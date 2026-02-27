//! SecGen Datastore Query Tool
//!
//! This tool allows ZeroClaw to query the SecGen datastore for randomized values
//! such as IP addresses, usernames, passwords, and flags.
//!
//! # Usage
//!
//! ```json
//! {
//!   "name": "secgen_datastore_query",
//!   "arguments": {
//!     "key": "IP_addresses",
//!     "index": 0,
//!     "field": null
//!   }
//! }
//! ```
//!
//! # Examples
//!
//! Query IP address:
//! ```
//! secgen_datastore_query --key IP_addresses --index 0
//! ```
//!
//! Query account username:
//! ```
//! secgen_datastore_query --key accounts --index 0 --field username
//! ```

use crate::tools::traits::{Tool, ToolResult};
use anyhow::{Context, Result};
use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use serde_json::json;
use std::collections::HashMap;
use std::fs;
use std::path::PathBuf;

/// Default SecGen datastore path
const DEFAULT_DATASTORE_PATH: &str = "/var/lib/secgen/datastore.json";

/// SecGen datastore query tool
pub struct SecGenDatastoreQueryTool {
    datastore_path: PathBuf,
    enabled: bool,
}

/// Query parameters for datastore access
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DatastoreQuery {
    /// Datastore key to query (e.g., "IP_addresses", "accounts", "flags")
    pub key: String,
    /// Array index (0-based), optional
    pub index: Option<usize>,
    /// Field name for object access (e.g., "username", "password"), optional
    pub field: Option<String>,
}

impl SecGenDatastoreQueryTool {
    pub fn new(datastore_path: Option<&str>, enabled: bool) -> Self {
        let path = datastore_path
            .map(PathBuf::from)
            .unwrap_or_else(|| PathBuf::from(DEFAULT_DATASTORE_PATH));

        Self {
            datastore_path: path,
            enabled,
        }
    }

    /// Read the SecGen datastore JSON file
    fn read_datastore(&self) -> Result<HashMap<String, serde_json::Value>> {
        if !self.enabled {
            anyhow::bail!("SecGen datastore query tool is disabled");
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

    /// Query the datastore with the given parameters
    fn query(&self, query: &DatastoreQuery) -> Result<String> {
        let data = self.read_datastore()?;

        let value = data.get(&query.key).with_context(|| {
            format!(
                "Key '{}' not found in datastore. Available keys: {:?}",
                query.key,
                data.keys().collect::<Vec<_>>()
            )
        })?;

        // Handle array access with index
        let result = if let Some(index) = query.index {
            let array = value
                .as_array()
                .with_context(|| format!("Key '{}' is not an array", query.key))?;

            let item = array.get(index).with_context(|| {
                format!(
                    "Index {} out of bounds for array '{}' (length: {})",
                    index,
                    query.key,
                    array.len()
                )
            })?;

            // Handle field access on object
            if let Some(field) = &query.field {
                let obj = item
                    .as_object()
                    .with_context(|| format!("Array item at index {} is not an object", index))?;

                obj.get(field)
                    .with_context(|| {
                        format!(
                            "Field '{}' not found in object at index {}. Available fields: {:?}",
                            field,
                            index,
                            obj.keys().collect::<Vec<_>>()
                        )
                    })?
                    .clone()
            } else {
                item.clone()
            }
        } else if let Some(field) = &query.field {
            // Handle field access on first object in array
            let array = value
                .as_array()
                .with_context(|| format!("Key '{}' is not an array", query.key))?;

            let first_item = array.first().with_context(|| {
                format!("Array '{}' is empty", query.key)
            })?;

            let obj = first_item
                .as_object()
                .with_context(|| "First array item is not an object")?;

            obj.get(field)
                .with_context(|| {
                    format!(
                        "Field '{}' not found in object. Available fields: {:?}",
                        field,
                        obj.keys().collect::<Vec<_>>()
                    )
                })?
                .clone()
        } else {
            value.clone()
        };

        // Convert result to string
        let result_str = if result.is_string() {
            result.as_str().unwrap().to_string()
        } else {
            result.to_string()
        };

        Ok(result_str)
    }
}

#[async_trait]
impl Tool for SecGenDatastoreQueryTool {
    fn name(&self) -> &str {
        "secgen_datastore_query"
    }

    fn description(&self) -> &str {
        "Query SecGen datastore for randomized values (IPs, usernames, passwords, flags). \
         Use this to access scenario-specific data generated by SecGen."
    }

    fn parameters_schema(&self) -> serde_json::Value {
        json!({
            "type": "object",
            "properties": {
                "key": {
                    "type": "string",
                    "description": "Datastore key to query (e.g., 'IP_addresses', 'accounts', 'flags')"
                },
                "index": {
                    "type": "integer",
                    "description": "Array index (0-based). Optional.",
                    "minimum": 0
                },
                "field": {
                    "type": "string",
                    "description": "Field name for object access (e.g., 'username', 'password'). Optional."
                }
            },
            "required": ["key"]
        })
    }

    async fn execute(&self, args: serde_json::Value) -> anyhow::Result<ToolResult> {
        // Parse arguments
        let query: DatastoreQuery = serde_json::from_value(args)
            .with_context(|| "Failed to parse secgen_datastore_query arguments")?;

        // Validate arguments
        if query.key.is_empty() {
            return Ok(ToolResult {
                success: false,
                output: String::new(),
                error: Some("'key' parameter cannot be empty".to_string()),
            });
        }

        // Execute query
        match self.query(&query) {
            Ok(result) => Ok(ToolResult {
                success: true,
                output: result,
                error: None,
            }),
            Err(e) => Ok(ToolResult {
                success: false,
                output: String::new(),
                error: Some(format!("Datastore query failed: {}", e)),
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
    fn test_query_ip_addresses() {
        let dir = TempDir::new().unwrap();
        let datastore_path = create_test_datastore(
            &dir,
            r#"{
                "IP_addresses": ["172.16.0.2", "172.16.0.3", "172.16.0.4"],
                "accounts": [
                    {"username": "user1", "password": "pass1"},
                    {"username": "user2", "password": "pass2"}
                ]
            }"#,
        );

        let tool = SecGenDatastoreQueryTool::new(Some(datastore_path.to_str().unwrap()), true);

        // Query first IP
        let result = tool
            .query(&DatastoreQuery {
                key: "IP_addresses".to_string(),
                index: Some(0),
                field: None,
            })
            .unwrap();
        assert_eq!(result, "172.16.0.2");

        // Query second IP
        let result = tool
            .query(&DatastoreQuery {
                key: "IP_addresses".to_string(),
                index: Some(1),
                field: None,
            })
            .unwrap();
        assert_eq!(result, "172.16.0.3");
    }

    #[test]
    fn test_query_account_field() {
        let dir = TempDir::new().unwrap();
        let datastore_path = create_test_datastore(
            &dir,
            r#"{
                "accounts": [
                    {"username": "user1", "password": "pass1"},
                    {"username": "user2", "password": "pass2"}
                ]
            }"#,
        );

        let tool = SecGenDatastoreQueryTool::new(Some(datastore_path.to_str().unwrap()), true);

        // Query username
        let result = tool
            .query(&DatastoreQuery {
                key: "accounts".to_string(),
                index: Some(0),
                field: Some("username".to_string()),
            })
            .unwrap();
        assert_eq!(result, "user1");

        // Query password
        let result = tool
            .query(&DatastoreQuery {
                key: "accounts".to_string(),
                index: Some(1),
                field: Some("password".to_string()),
            })
            .unwrap();
        assert_eq!(result, "pass2");
    }

    #[test]
    fn test_query_nonexistent_key() {
        let dir = TempDir::new().unwrap();
        let datastore_path = create_test_datastore(&dir, r#"{"IP_addresses": ["172.16.0.2"]}"#);

        let tool = SecGenDatastoreQueryTool::new(Some(datastore_path.to_str().unwrap()), true);

        let result = tool.query(&DatastoreQuery {
            key: "nonexistent".to_string(),
            index: None,
            field: None,
        });

        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("not found in datastore"));
    }
}
