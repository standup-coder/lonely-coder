use std::os::unix::io::AsRawFd;
use std::os::unix::io::RawFd;

pub struct RawModeGuard {
    original_termios: libc::termios,
    fd: RawFd,
    active: bool,
}

impl RawModeGuard {
    pub fn new() -> anyhow::Result<Self> {
        let fd = std::io::stdin().as_raw_fd();
        let original = unsafe { libc::tcgetattr(fd) };
        if original != 0 {
            anyhow::bail!("failed to get terminal attributes");
        }
        let mut raw = original;

        unsafe { libc::cfmakeraw(&mut raw) };

        if unsafe { libc::tcsetattr(fd, libc::TCSANOW, &raw) } != 0 {
            anyhow::bail!("failed to set terminal to raw mode");
        }

        Ok(Self {
            original_termios: original,
            fd,
            active: true,
        })
    }

    pub fn disable(&mut self) {
        if self.active {
            unsafe {
                libc::tcsetattr(self.fd, libc::TCSANOW, &self.original_termios);
            }
            self.active = false;
        }
    }
}

impl Drop for RawModeGuard {
    fn drop(&mut self) {
        self.disable();
    }
}

pub fn terminal_size() -> (u16, u16) {
    crossterm::terminal::size().unwrap_or((80, 24))
}
