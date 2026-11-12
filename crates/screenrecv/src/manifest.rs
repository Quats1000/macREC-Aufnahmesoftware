//! Manifest (`info.json`) und Protokoll (`log.jsonl`) auf dem USB-Stick.

use std::fs::OpenOptions;
use std::io::Write;
use std::path::Path;

use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Clone, Debug, Default)]
pub struct Manifest {
    pub app: String,
    pub bundle_id: String,
    pub version: String,
    pub name: String,
    pub class: String,
    pub test: String,
    pub date: String,
    pub started_at: String,
    pub ended_at: Option<String>,
    pub screenshots: u32,
    pub last_screenshot_at: Option<String>,
    pub storage_root: String,
    pub device: String,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct LogEntry {
    pub index: u32,
    pub timestamp: String,
    pub file: String,
    pub bytes: u64,
    pub width: u32,
    pub height: u32,
}

impl Manifest {
    pub fn write(&self, dir: &Path) -> std::io::Result<()> {
        let json = serde_json::to_string_pretty(self)?;
        std::fs::write(dir.join(crate::storage::INFO_FILE), json)
    }
}

pub fn append_log(dir: &Path, entry: &LogEntry) -> std::io::Result<()> {
    let mut file = OpenOptions::new()
        .create(true)
        .append(true)
        .open(dir.join(crate::storage::LOG_FILE))?;
    let line = serde_json::to_string(entry)?;
    writeln!(file, "{line}")
}

/// Name des Geraets (fuer das Manifest).
pub fn device_name() -> String {
    #[cfg(windows)]
    {
        std::env::var("COMPUTERNAME").unwrap_or_else(|_| "unknown".to_owned())
    }
    #[cfg(not(windows))]
    {
        std::env::var("HOSTNAME").unwrap_or_else(|_| "unknown".to_owned())
    }
}
