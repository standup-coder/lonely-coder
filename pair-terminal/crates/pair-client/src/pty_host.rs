use anyhow::Result;
use portable_pty::{native_pty_system, CommandBuilder, PtySize};
use std::io::{Read, Write};

pub struct PtyHost {
    reader: Box<dyn Read + Send>,
    writer: Box<dyn Write + Send>,
    child: Box<dyn portable_pty::Child>,
    master: Box<dyn portable_pty::MasterPty>,
    cols: u16,
    rows: u16,
}

impl PtyHost {
    pub fn spawn(command: &[String], cols: u16, rows: u16) -> Result<Self> {
        let pty_system = native_pty_system();
        let pair = pty_system.openpty(PtySize {
            rows: rows.saturating_sub(1),
            cols,
            pixel_width: 0,
            pixel_height: 0,
        })?;

        let cmd = if command.is_empty() || (command.len() == 1 && command[0].is_empty()) {
            let shell = std::env::var("SHELL").unwrap_or_else(|_| "/bin/bash".to_string());
            CommandBuilder::new(shell)
        } else {
            let mut cmd = CommandBuilder::new(&command[0]);
            for arg in &command[1..] {
                cmd.arg(arg);
            }
            cmd
        };

        let child = pair.slave.spawn_command(cmd)?;
        drop(pair.slave);

        let reader = pair.master.try_clone_reader()?;
        let writer = pair.master.take_writer()?;
        let master = pair.master;

        Ok(Self {
            reader,
            writer,
            child,
            master,
            cols,
            rows,
        })
    }

    pub fn read_output(&mut self, buf: &mut [u8]) -> Result<usize> {
        let n = self.reader.read(buf)?;
        Ok(n)
    }

    pub fn write_input(&mut self, data: &[u8]) -> Result<()> {
        self.writer.write_all(data)?;
        self.writer.flush()?;
        Ok(())
    }

    pub fn resize(&mut self, cols: u16, rows: u16) -> Result<()> {
        self.cols = cols;
        self.rows = rows;
        self.master.resize(PtySize {
            rows: rows.saturating_sub(1),
            cols,
            pixel_width: 0,
            pixel_height: 0,
        })?;
        Ok(())
    }

    pub fn cols(&self) -> u16 {
        self.cols
    }

    pub fn rows(&self) -> u16 {
        self.rows
    }
}

impl Drop for PtyHost {
    fn drop(&mut self) {
        let _ = self.child.kill();
        let _ = self.child.wait();
    }
}
