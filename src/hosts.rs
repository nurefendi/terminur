use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::{fs, path::PathBuf};

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Host {
    pub name: String,
    pub hostname: String,
    pub username: String,
    pub port: u16,
}

pub fn path() -> PathBuf {
    dirs::config_dir()
        .unwrap_or_else(|| PathBuf::from("."))
        .join("rustterm")
        .join("hosts.json")
}

pub fn load() -> Vec<Host> {
    fs::read_to_string(path())
        .ok()
        .and_then(|s| serde_json::from_str(&s).ok())
        .unwrap_or_else(|| {
            vec![Host {
                name: "Localhost".into(),
                hostname: "localhost".into(),
                username: std::env::var("USER").unwrap_or_else(|_| "user".into()),
                port: 22,
            }]
        })
}

#[expect(dead_code, reason = "Used by the upcoming GPUI host editor")]
pub fn save(hosts: &[Host]) -> Result<()> {
    let file = path();
    if let Some(parent) = file.parent() {
        fs::create_dir_all(parent)?;
    }
    fs::write(file, serde_json::to_vec_pretty(hosts)?)?;
    Ok(())
}
