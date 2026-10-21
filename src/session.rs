use crate::hosts::{resolve_identity_file, Host};
use anyhow::{Context, Result};
use portable_pty::{native_pty_system, CommandBuilder, MasterPty, PtySize};
use std::{
    io::{Read, Write},
    sync::{Arc, Mutex},
    thread,
};

/// An interactive SSH process attached to a pseudo-terminal.
pub struct Session {
    master: Box<dyn MasterPty + Send>,
    writer: Box<dyn Write + Send>,
    child: Box<dyn portable_pty::Child + Send + Sync>,
}

impl Session {
    pub fn connect_host(
        host: &Host,
        rows: u16,
        cols: u16,
        on_data: impl FnMut(Vec<u8>) + Send + 'static,
    ) -> Result<Self> {
        Self::spawn(ssh_command(host)?, rows, cols, on_data)
    }

    /// Starts an interactive login shell. Only used to exercise the PTY
    /// plumbing in tests; the app itself targets SSH hosts.
    #[cfg(test)]
    pub fn connect_shell(
        shell: &str,
        rows: u16,
        cols: u16,
        on_data: impl FnMut(Vec<u8>) + Send + 'static,
    ) -> Result<Self> {
        let mut cmd = CommandBuilder::new(shell);
        cmd.arg("-l");
        if let Some(home) = dirs::home_dir() {
            cmd.cwd(home);
        }
        Self::spawn(cmd, rows, cols, on_data)
    }

    fn spawn(
        mut cmd: CommandBuilder,
        rows: u16,
        cols: u16,
        mut on_data: impl FnMut(Vec<u8>) + Send + 'static,
    ) -> Result<Self> {
        cmd.env("TERM", "xterm-256color");
        cmd.env("COLORTERM", "truecolor");
        let pty = native_pty_system().openpty(PtySize {
            rows,
            cols,
            pixel_width: 0,
            pixel_height: 0,
        })?;
        let child = pty
            .slave
            .spawn_command(cmd)
            .context("Could not start terminal process")?;
        drop(pty.slave);
        let mut reader = pty.master.try_clone_reader()?;
        let writer = pty.master.take_writer()?;
        thread::spawn(move || {
            let mut buf = [0u8; 8192];
            loop {
                match reader.read(&mut buf) {
                    Ok(0) | Err(_) => break,
                    Ok(n) => on_data(buf[..n].to_vec()),
                }
            }
        });
        Ok(Self {
            master: pty.master,
            writer,
            child,
        })
    }

    pub fn send(&mut self, bytes: &[u8]) -> Result<()> {
        self.writer.write_all(bytes)?;
        self.writer.flush()?;
        Ok(())
    }

    /// Tells the kernel and the remote shell that the window changed size.
    pub fn resize(&self, rows: u16, cols: u16) -> Result<()> {
        self.master.resize(PtySize {
            rows,
            cols,
            pixel_width: 0,
            pixel_height: 0,
        })?;
        Ok(())
    }

    /// Nonblocking process check; spawning ssh is not proof of authentication.
    pub fn try_wait(&mut self) -> Result<Option<portable_pty::ExitStatus>> {
        Ok(self.child.try_wait()?)
    }
}

fn ssh_command(host: &Host) -> Result<CommandBuilder> {
    let mut cmd = CommandBuilder::new("/usr/bin/ssh");
    cmd.arg("-p");
    cmd.arg(host.port.to_string());
    cmd.arg("-o");
    cmd.arg("StrictHostKeyChecking=ask");
    cmd.arg("-o");
    cmd.arg("ServerAliveInterval=30");
    if let Some(path) = &host.identity_file {
        cmd.arg("-i");
        cmd.arg(resolve_identity_file(path)?);
        cmd.arg("-o");
        cmd.arg("IdentitiesOnly=yes");
    }
    cmd.arg("--");
    cmd.arg(format!("{}@{}", host.username, host.hostname));
    Ok(cmd)
}

impl Drop for Session {
    fn drop(&mut self) {
        // Switching targets must not leave the previous shell running.
        let _ = self.child.kill();
    }
}

pub type SharedSession = Arc<Mutex<Session>>;

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::mpsc;

    #[test]
    fn ssh_uses_key_as_single_argument_only_when_selected() {
        let mut host = Host {
            name: "Test".into(),
            hostname: "example.com".into(),
            username: "me".into(),
            port: 2222,
            identity_file: None,
        };
        let argv: Vec<_> = ssh_command(&host)
            .unwrap()
            .get_argv()
            .iter()
            .map(|arg| arg.to_string_lossy().into_owned())
            .collect();
        assert!(!argv
            .iter()
            .any(|arg| arg == "-i" || arg == "IdentitiesOnly=yes"));

        let key = std::env::temp_dir().join(format!(
            "terminur-test-key-with spaces-{}-{}",
            std::process::id(),
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_nanos()
        ));
        std::fs::write(&key, "test path only; not a private key").unwrap();
        host.identity_file = Some(key.to_string_lossy().into());
        let argv: Vec<_> = ssh_command(&host)
            .unwrap()
            .get_argv()
            .iter()
            .map(|arg| arg.to_string_lossy().into_owned())
            .collect();
        let index = argv.iter().position(|arg| arg == "-i").unwrap();
        assert_eq!(argv[index + 1], key.to_string_lossy());
        assert!(argv.iter().any(|arg| arg == "IdentitiesOnly=yes"));
        assert_eq!(argv.last().unwrap(), "me@example.com");
        std::fs::remove_file(&key).unwrap();
        assert!(ssh_command(&host).is_err());
    }

    #[test]
    fn local_shell_produces_output_and_resizes() {
        let (tx, rx) = mpsc::channel::<Vec<u8>>();
        let mut session = Session::connect_shell("/bin/sh", 24, 80, move |bytes| {
            let _ = tx.send(bytes);
        })
        .expect("spawn local shell");
        session
            .send(b"printf 'terminur-ok\\n'\n")
            .expect("write to pty");
        let deadline = std::time::Instant::now() + std::time::Duration::from_secs(5);
        let mut seen = String::new();
        while std::time::Instant::now() < deadline {
            match rx.recv_timeout(std::time::Duration::from_millis(200)) {
                Ok(bytes) => {
                    seen.push_str(&String::from_utf8_lossy(&bytes));
                    if seen.contains("terminur-ok") {
                        break;
                    }
                }
                Err(_) => continue,
            }
        }
        assert!(seen.contains("terminur-ok"), "shell output was: {seen:?}");
        session.resize(30, 100).expect("resize pty");
        session.send(b"exit\n").expect("stop test shell");
        let deadline = std::time::Instant::now() + std::time::Duration::from_secs(5);
        while std::time::Instant::now() < deadline {
            if let Some(status) = session.try_wait().expect("poll child") {
                assert!(status.success(), "shell exited with: {status:?}");
                return;
            }
            std::thread::sleep(std::time::Duration::from_millis(10));
        }
        panic!("shell did not exit");
    }
}
