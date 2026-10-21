use crate::hosts::Host;
use anyhow::{Context, Result};
use portable_pty::{native_pty_system, CommandBuilder, MasterPty, PtySize};
use std::{
    io::{Read, Write},
    sync::{Arc, Mutex},
    thread,
};

pub struct Session {
    #[expect(dead_code, reason = "Kept for PTY resizing support")]
    master: Box<dyn MasterPty + Send>,
    writer: Box<dyn Write + Send>,
    _child: Box<dyn portable_pty::Child + Send + Sync>,
}

impl Session {
    pub fn connect(host: &Host, on_data: impl Fn(Vec<u8>) + Send + 'static) -> Result<Self> {
        let pty = native_pty_system().openpty(PtySize {
            rows: 30,
            cols: 100,
            pixel_width: 0,
            pixel_height: 0,
        })?;
        let mut cmd = CommandBuilder::new("/usr/bin/ssh");
        cmd.arg("-p");
        cmd.arg(host.port.to_string());
        cmd.arg("-o");
        cmd.arg("StrictHostKeyChecking=ask");
        cmd.arg("--");
        cmd.arg(format!("{}@{}", host.username, host.hostname));
        let child = pty
            .slave
            .spawn_command(cmd)
            .context("Could not start ssh")?;
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
            _child: child,
        })
    }

    pub fn send(&mut self, bytes: &[u8]) -> Result<()> {
        self.writer.write_all(bytes)?;
        self.writer.flush()?;
        Ok(())
    }

    #[expect(dead_code, reason = "Terminal resizing UI is not wired up yet")]
    pub fn resize(&self, rows: u16, cols: u16) -> Result<()> {
        self.master.resize(PtySize {
            rows,
            cols,
            pixel_width: 0,
            pixel_height: 0,
        })?;
        Ok(())
    }
}

pub type SharedSession = Arc<Mutex<Session>>;
