use crate::hosts::{resolve_identity_file, Host};

pub const LABELS: [&str; 5] = [
    "Name",
    "Hostname / IP",
    "Username",
    "SSH port",
    "SSH private key path (optional)",
];

pub struct HostDraft {
    pub values: [String; 5],
    pub selected: usize,
    pub cursor: usize,
    pub error: Option<String>,
}

impl Default for HostDraft {
    fn default() -> Self {
        Self {
            values: [
                String::new(),
                String::new(),
                std::env::var("USER").unwrap_or_default(),
                "22".into(),
                String::new(),
            ],
            selected: 0,
            cursor: 0,
            error: None,
        }
    }
}

impl HostDraft {
    pub fn continue_from_address(&mut self) -> Result<(), String> {
        let hostname = self.values[1].trim();
        if let Err(error) = validate_hostname(hostname) {
            self.error = Some(error.clone());
            return Err(error);
        }
        if self.values[0].trim().is_empty() {
            self.values[0] = hostname.to_owned();
        }
        self.error = None;
        self.select(0);
        Ok(())
    }

    pub fn select(&mut self, index: usize) {
        if index < self.values.len() {
            self.selected = index;
            self.cursor = self.values[index].len();
        }
    }

    pub fn next(&mut self, reverse: bool) {
        let count = self.values.len();
        self.select((self.selected + if reverse { count - 1 } else { 1 }) % count);
    }

    pub fn insert(&mut self, text: &str) {
        if text.chars().any(char::is_control) || self.values[self.selected].len() + text.len() > 255
        {
            self.error = Some("Input must be a single line (max 255 bytes).".into());
            return;
        }
        self.values[self.selected].insert_str(self.cursor, text);
        self.cursor += text.len();
        self.error = None;
    }

    pub fn backspace(&mut self) {
        if let Some(index) = self.values[self.selected][..self.cursor]
            .char_indices()
            .last()
            .map(|(index, _)| index)
        {
            self.values[self.selected].drain(index..self.cursor);
            self.cursor = index;
            self.error = None;
        }
    }

    pub fn delete(&mut self) {
        let value = &mut self.values[self.selected];
        if let Some(ch) = value[self.cursor..].chars().next() {
            value.drain(self.cursor..self.cursor + ch.len_utf8());
            self.error = None;
        }
    }

    pub fn move_cursor(&mut self, right: bool) {
        let value = &self.values[self.selected];
        if right {
            if let Some(ch) = value[self.cursor..].chars().next() {
                self.cursor += ch.len_utf8();
            }
        } else if let Some((index, _)) = value[..self.cursor].char_indices().last() {
            self.cursor = index;
        }
    }

    pub fn display(&self, index: usize) -> String {
        let value = &self.values[index];
        if self.selected == index {
            format!("{}│{}", &value[..self.cursor], &value[self.cursor..])
        } else {
            value.clone()
        }
    }

    pub fn validate(&self) -> Result<Host, String> {
        let [name, hostname, username, port, identity_file] =
            self.values.each_ref().map(|v| v.trim());
        if name.is_empty() {
            return Err("Name is required.".into());
        }
        validate_hostname(hostname)?;
        if username.is_empty()
            || username
                .chars()
                .any(|c| c.is_whitespace() || c.is_control() || c == '@')
        {
            return Err("Enter a valid SSH username.".into());
        }
        let port: u16 = port
            .parse()
            .map_err(|_| "SSH port must be between 1 and 65535.".to_owned())?;
        if port == 0 {
            return Err("SSH port must be between 1 and 65535.".into());
        }
        if !identity_file.is_empty() {
            resolve_identity_file(identity_file).map_err(|error| error.to_string())?;
        }
        Ok(Host {
            name: name.into(),
            hostname: hostname.into(),
            username: username.into(),
            port,
            identity_file: (!identity_file.is_empty()).then(|| identity_file.to_owned()),
        })
    }
}

fn validate_hostname(hostname: &str) -> Result<(), String> {
    if hostname.is_empty()
        || hostname.starts_with('-')
        || hostname
            .chars()
            .any(|c| c.is_whitespace() || c.is_control() || c == '@' || c == '/')
    {
        return Err("Enter a valid hostname or IP address.".into());
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn valid_host_and_bad_values() {
        let mut draft = HostDraft {
            values: [
                " Production ".into(),
                "example.com".into(),
                "admin".into(),
                "2222".into(),
                String::new(),
            ],
            ..HostDraft::default()
        };
        let host = draft.validate().unwrap();
        assert_eq!((host.name.as_str(), host.port), ("Production", 2222));
        draft.values[1] = "-oProxyCommand=bad".into();
        assert!(draft.validate().is_err());
        draft.values[1] = "example.com".into();
        draft.values[3] = "0".into();
        assert!(draft.validate().is_err());
        draft.values[3] = "65536".into();
        assert!(draft.validate().is_err());
        draft.values[3] = "22".into();
        draft.values[2] = "a@b".into();
        assert!(draft.validate().is_err());
        draft.values[2] = "admin".into();
        draft.values[4] = "missing-relative-key".into();
        assert!(draft.validate().is_err());
        assert!(host.identity_file.is_none());
    }

    #[test]
    fn editing_keeps_utf8_cursor_boundaries() {
        let mut draft = HostDraft::default();
        draft.insert("éx");
        draft.move_cursor(false);
        draft.insert("中");
        assert_eq!(draft.values[0], "é中x");
        draft.backspace();
        assert_eq!(draft.values[0], "éx");
        draft.delete();
        assert_eq!(draft.values[0], "é");
        draft.next(false);
        assert_eq!(draft.selected, 1);
        draft.insert("bad\nline");
        assert!(draft.values[1].is_empty());
    }

    #[test]
    fn validates_optional_existing_key_path() {
        let base = std::env::temp_dir().join(format!(
            "terminur-form-key-{}-{}",
            std::process::id(),
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_nanos()
        ));
        std::fs::create_dir_all(&base).unwrap();
        let key = base.join("id_example");
        std::fs::write(&key, "dummy file for path validation").unwrap();
        let draft = HostDraft {
            values: [
                "Test".into(),
                "example.com".into(),
                "me".into(),
                "22".into(),
                key.to_string_lossy().into(),
            ],
            ..HostDraft::default()
        };
        assert_eq!(
            draft.validate().unwrap().identity_file.as_deref(),
            key.to_str()
        );
        std::fs::remove_dir_all(base).unwrap();
    }

    #[test]
    fn address_step_requires_hostname_and_prefills_name() {
        let mut draft = HostDraft::default();
        draft.select(1);
        assert!(draft.continue_from_address().is_err());
        assert_eq!(draft.selected, 1);
        draft.insert("user@server");
        assert!(draft.continue_from_address().is_err());
        draft.values[1] = "example.com".into();
        assert!(draft.continue_from_address().is_ok());
        assert_eq!(draft.values[0], "example.com");
        assert_eq!(draft.selected, 0);
        assert!(draft.error.is_none());
        assert_eq!(draft.validate().unwrap().hostname, "example.com");
    }
}
