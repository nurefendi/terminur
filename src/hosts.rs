use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use std::{
    fs,
    io::Write,
    path::{Path, PathBuf},
    time::{SystemTime, UNIX_EPOCH},
};

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Host {
    pub name: String,
    pub hostname: String,
    pub username: String,
    pub port: u16,
    /// Path only: private key bytes and passphrase are never stored here.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub identity_file: Option<String>,
}

/// Resolve only absolute or ~/ paths; do not invoke a shell to expand input.
pub fn resolve_identity_file(raw: &str) -> Result<PathBuf> {
    resolve_identity_file_with_home(raw, dirs::home_dir().as_deref())
}

fn resolve_identity_file_with_home(raw: &str, home: Option<&Path>) -> Result<PathBuf> {
    let raw = raw.trim();
    anyhow::ensure!(
        !raw.is_empty() && !raw.chars().any(char::is_control),
        "Invalid SSH key path"
    );
    let path = if let Some(relative) = raw.strip_prefix("~/") {
        home.context("Cannot find home directory for SSH key")?
            .join(relative)
    } else {
        PathBuf::from(raw)
    };
    anyhow::ensure!(
        path.is_absolute(),
        "SSH key path must be absolute or start with ~/"
    );
    anyhow::ensure!(
        path.is_file(),
        "SSH key path must point to an existing file"
    );
    fs::File::open(&path).with_context(|| format!("Cannot read SSH key at {}", path.display()))?;
    Ok(path)
}

fn config_path(base: &Path) -> PathBuf {
    let current = base.join("terminur").join("hosts.json");
    let legacy = base.join("rustterm").join("hosts.json");
    if !current.exists() && legacy.exists() {
        legacy
    } else {
        current
    }
}

pub fn load() -> Vec<Host> {
    let base = dirs::config_dir().unwrap_or_else(|| PathBuf::from("."));
    load_from(&base)
}

fn load_from(base: &Path) -> Vec<Host> {
    fs::read_to_string(config_path(base))
        .ok()
        .and_then(|s| serde_json::from_str(&s).ok())
        .unwrap_or_default()
}

pub fn save(hosts: &[Host]) -> Result<()> {
    let base = dirs::config_dir().context("Cannot find configuration directory")?;
    save_to(&base, hosts)
}

fn save_to(base: &Path, hosts: &[Host]) -> Result<()> {
    let path = config_path(base);
    if path.exists() {
        let existing =
            fs::read(&path).with_context(|| format!("Cannot read {}", path.display()))?;
        serde_json::from_slice::<Vec<Host>>(&existing)
            .with_context(|| format!("Invalid hosts file {}; not overwriting", path.display()))?;
    }
    let parent = path.parent().context("Invalid hosts path")?;
    fs::create_dir_all(parent)?;
    let nonce = SystemTime::now().duration_since(UNIX_EPOCH)?.as_nanos();
    let tmp = parent.join(format!(".hosts-{}-{nonce}.tmp", std::process::id()));
    let result = (|| -> Result<()> {
        let mut file = fs::OpenOptions::new()
            .write(true)
            .create_new(true)
            .open(&tmp)?;
        file.write_all(&serde_json::to_vec_pretty(hosts)?)?;
        file.sync_all()?;
        drop(file);
        fs::rename(&tmp, &path)?;
        Ok(())
    })();
    if result.is_err() {
        let _ = fs::remove_file(&tmp);
    }
    result.with_context(|| format!("Could not save host list to {}", path.display()))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn config_path_prefers_current_and_preserves_legacy() {
        let base = std::env::temp_dir().join(format!(
            "terminur-hosts-{}-{}",
            std::process::id(),
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_nanos()
        ));
        let current = base.join("terminur/hosts.json");
        let legacy = base.join("rustterm/hosts.json");
        assert_eq!(config_path(&base), current);
        assert!(load_from(&base).is_empty());
        fs::create_dir_all(legacy.parent().unwrap()).unwrap();
        let saved = r#"[{"name":"My SSH","hostname":"localhost","username":"me","port":22}]"#;
        fs::write(&legacy, saved).unwrap();
        assert_eq!(config_path(&base), legacy);
        assert_eq!(load_from(&base)[0].hostname, "localhost");
        assert!(load_from(&base)[0].identity_file.is_none());
        fs::create_dir_all(current.parent().unwrap()).unwrap();
        fs::write(&current, "invalid JSON").unwrap();
        assert_eq!(config_path(&base), current);
        assert!(load_from(&base).is_empty());
        assert_eq!(fs::read_to_string(&legacy).unwrap(), saved);
        fs::remove_file(current).unwrap();
        fs::remove_file(legacy).unwrap();
        fs::remove_dir(base.join("terminur")).unwrap();
        fs::remove_dir(base.join("rustterm")).unwrap();
        fs::remove_dir(base).unwrap();
    }

    #[test]
    fn save_roundtrip_and_reject_invalid_file() {
        let base = std::env::temp_dir().join(format!(
            "terminur-save-{}-{}",
            std::process::id(),
            SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap()
                .as_nanos()
        ));
        let first = Host {
            name: "Server".into(),
            hostname: "example.com".into(),
            username: "admin".into(),
            port: 22,
            identity_file: None,
        };
        save_to(&base, std::slice::from_ref(&first)).unwrap();
        assert_eq!(load_from(&base).len(), 1);
        let path = config_path(&base);
        fs::write(&path, "not json").unwrap();
        assert!(save_to(&base, &[first]).is_err());
        assert_eq!(fs::read_to_string(&path).unwrap(), "not json");
        fs::remove_dir_all(base).unwrap();
    }

    #[test]
    fn saving_legacy_host_keeps_existing_entries() {
        let base = std::env::temp_dir().join(format!(
            "terminur-legacy-save-{}-{}",
            std::process::id(),
            SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap()
                .as_nanos()
        ));
        let legacy = base.join("rustterm/hosts.json");
        fs::create_dir_all(legacy.parent().unwrap()).unwrap();
        fs::write(
            &legacy,
            r#"[{"name":"Old","hostname":"old.example","username":"me","port":22}]"#,
        )
        .unwrap();
        let mut hosts = load_from(&base);
        hosts.push(Host {
            name: "New".into(),
            hostname: "new.example".into(),
            username: "me".into(),
            port: 2222,
            identity_file: None,
        });
        save_to(&base, &hosts).unwrap();
        assert_eq!(load_from(&base).len(), 2);
        assert!(!base.join("terminur/hosts.json").exists());
        fs::remove_dir_all(base).unwrap();
    }

    #[test]
    fn key_path_checks_existing_file_without_storing_contents() {
        let base = std::env::temp_dir().join(format!(
            "terminur-identity-{}-{}",
            std::process::id(),
            SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap()
                .as_nanos()
        ));
        fs::create_dir_all(&base).unwrap();
        let key = base.join("sample key");
        fs::write(&key, "test-key-bytes-not-a-real-key").unwrap();
        assert_eq!(resolve_identity_file(key.to_str().unwrap()).unwrap(), key);
        assert_eq!(
            resolve_identity_file_with_home("~/sample key", Some(&base)).unwrap(),
            key
        );
        assert!(resolve_identity_file_with_home("~/sample key", None).is_err());
        assert!(resolve_identity_file("relative/key").is_err());
        assert!(resolve_identity_file(base.to_str().unwrap()).is_err());
        assert!(resolve_identity_file(&base.join("missing").to_string_lossy()).is_err());
        let host = Host {
            name: "Test".into(),
            hostname: "example.com".into(),
            username: "me".into(),
            port: 22,
            identity_file: Some(key.to_string_lossy().into()),
        };
        let json = serde_json::to_string(&host).unwrap();
        assert!(json.contains("identity_file"));
        assert!(!json.contains("test-key-bytes-not-a-real-key"));
        save_to(&base, &[host]).unwrap();
        assert_eq!(load_from(&base)[0].identity_file.as_deref(), key.to_str());
        fs::remove_dir_all(base).unwrap();
    }
}
