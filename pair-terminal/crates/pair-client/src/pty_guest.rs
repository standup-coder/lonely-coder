use anyhow::Result;
use std::io::{Read, Write};

pub struct PtyGuest;

impl PtyGuest {
    pub fn render_output(data: &[u8]) -> Result<()> {
        let mut stdout = std::io::stdout().lock();
        stdout.write_all(data)?;
        stdout.flush()?;
        Ok(())
    }

    pub fn read_local_input(buf: &mut [u8]) -> Result<usize> {
        let stdin = std::io::stdin();
        let mut handle = stdin.lock();
        let n = handle.read(buf)?;
        Ok(n)
    }
}
