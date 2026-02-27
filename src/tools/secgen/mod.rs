//! SecGen integration tools for ZeroClaw
//!
//! This module provides tools for integrating ZeroClaw with SecGen-generated scenarios:
//! - [`secgen_datastore_query`]: Query SecGen datastore for randomized values
//! - [`secgen_flag_validator`]: Validate CTF flags and track progress

pub mod secgen_datastore_query;
pub mod secgen_flag_validator;

pub use secgen_datastore_query::SecGenDatastoreQueryTool;
pub use secgen_flag_validator::SecGenFlagValidatorTool;
