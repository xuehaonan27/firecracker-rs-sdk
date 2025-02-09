use std::path::PathBuf;

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, PartialOrd, Ord, Hash, Default)]
pub enum LogLevel {
    #[serde(rename = "Error")]
    Error,
    #[serde(rename = "Warning")]
    Warning,
    #[default]
    #[serde(rename = "Info")]
    Info,
    #[serde(rename = "Debug")]
    Debug,
    #[serde(rename = "Trace")]
    Trace,
    #[serde(rename = "Off")]
    Off,
}

/// Describes the configuration option for the logging capability.
/// logger can only be constructed once
/// and cannot update after configuration
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Logger {
    /// Set the level. The possible values are case-insensitive.
    /// Enum: [Error Warning Info Debug]
    #[serde(rename = "level", skip_serializing_if = "Option::is_none")]
    pub level: Option<LogLevel>,

    /// Path to the named pipe or file for the human readable log output.
    /// Required: true
    #[serde(rename = "log_path")]
    pub log_path: PathBuf,

    /// Whether or not to output the level in the logs.
    #[serde(rename = "show_level", skip_serializing_if = "Option::is_none")]
    pub show_level: Option<bool>,

    /// Whether or not to include the file path and line number of the log's origin.
    #[serde(rename = "show_log_origin", skip_serializing_if = "Option::is_none")]
    pub show_log_origin: Option<bool>,

    /// The module path to filter log messages by. example: api_server::request
    #[serde(rename = "module", skip_serializing_if = "Option::is_none")]
    pub module: Option<String>,
}
