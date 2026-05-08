use crossterm::event::{self, Event, KeyCode, KeyEvent, KeyModifiers};
use crossterm::terminal;
use std::io::{self, Write};

pub struct StatusBar {
    mode: String,
    peer: String,
    connected: bool,
    start_time: std::time::Instant,
    show_help: bool,
    show_chat: bool,
    chat_input: String,
}

impl StatusBar {
    pub fn new(mode: &str, peer: &str) -> Self {
        Self {
            mode: mode.to_string(),
            peer: peer.to_string(),
            connected: true,
            start_time: std::time::Instant::now(),
            show_help: false,
            show_chat: false,
            chat_input: String::new(),
        }
    }

    pub fn set_connected(&mut self, connected: bool) {
        self.connected = connected;
    }

    pub fn toggle_help(&mut self) {
        self.show_help = !self.show_help;
    }

    pub fn toggle_chat(&mut self) {
        self.show_chat = !self.show_chat;
    }

    pub fn render(&self) -> anyhow::Result<()> {
        let mut stdout = io::stdout().lock();

        let elapsed = self.start_time.elapsed();
        let minutes = elapsed.as_secs() / 60;
        let seconds = elapsed.as_secs() % 60;

        let status = if self.connected { "🟢" } else { "🔴" };
        let mode_display = match self.mode.as_str() {
            "driver" => "DRIVER",
            "navigator" => "NAVIGATOR",
            _ => "COLLAB",
        };

        if self.show_help {
            let help_lines = vec![
                "╭─ pair-terminal ─────────────────────────╮",
                "│ [c] chat    [d] driver    [n] navigator │",
                "│ [m] collab  [r] record   [q] quit       │",
                "│ [s] snapshot [h] toggle help             │",
                "│ [Esc] close this panel                   │",
                "╰──────────────────────────────────────────╯",
            ];
            for line in help_lines {
                terminal::disable_raw_mode()?;
                println!("\x1b[s{}", line);
                terminal::enable_raw_mode()?;
            }
            terminal::disable_raw_mode()?;
            print!("\x1b[u");
            terminal::enable_raw_mode()?;
        } else if self.show_chat {
            let chat_lines = vec![
                "╭─ Chat ───────────────────────────────────╮",
                "│ Type message, Enter to send, Esc to close │",
                "╰──────────────────────────────────────────╯",
            ];
            for line in chat_lines {
                terminal::disable_raw_mode()?;
                println!("\x1b[s{}", line);
                terminal::enable_raw_mode()?;
            }
            terminal::disable_raw_mode()?;
            print!("\x1b[u");
            terminal::enable_raw_mode()?;
        } else {
            terminal::disable_raw_mode()?;
            print!(
                "\r\x1b[K── pair: {} · {} · {:02}:{:02} ── Ctrl+T help ──",
                status, mode_display, minutes, seconds
            );
            terminal::enable_raw_mode()?;
        }

        stdout.flush()?;
        Ok(())
    }

    pub async fn poll_event(&mut self) -> anyhow::Result<TuiEvent> {
        if event::poll(std::time::Duration::from_millis(100))? {
            if let Event::Key(key) = event::read()? {
                if key.modifiers.contains(KeyModifiers::CONTROL) && key.code == KeyCode::Char('t') {
                    return Ok(TuiEvent::ToggleHelp);
                }

                if self.show_help {
                    match key.code {
                        KeyCode::Char('c') => {
                            self.show_help = false;
                            self.show_chat = true;
                            return Ok(TuiEvent::OpenChat);
                        }
                        KeyCode::Char('d') => return Ok(TuiEvent::SetMode("driver".to_string())),
                        KeyCode::Char('n') => return Ok(TuiEvent::SetMode("navigator".to_string())),
                        KeyCode::Char('m') => return Ok(TuiEvent::SetMode("collab".to_string())),
                        KeyCode::Char('r') => return Ok(TuiEvent::ToggleRecord),
                        KeyCode::Char('q') => return Ok(TuiEvent::Quit),
                        KeyCode::Char('s') => return Ok(TuiEvent::Snapshot),
                        KeyCode::Char('h') | KeyCode::Esc => {
                            self.show_help = false;
                            return Ok(TuiEvent::Nop);
                        }
                        _ => return Ok(TuiEvent::Nop),
                    }
                }

                if self.show_chat {
                    match key.code {
                        KeyCode::Esc => {
                            self.show_chat = false;
                            self.chat_input.clear();
                            return Ok(TuiEvent::Nop);
                        }
                        KeyCode::Enter => {
                            let msg = self.chat_input.clone();
                            self.chat_input.clear();
                            return Ok(TuiEvent::ChatMessage(msg));
                        }
                        KeyCode::Backspace => {
                            self.chat_input.pop();
                            return Ok(TuiEvent::Nop);
                        }
                        KeyCode::Char(c) => {
                            self.chat_input.push(c);
                            return Ok(TuiEvent::Nop);
                        }
                        _ => return Ok(TuiEvent::Nop),
                    }
                }
            }
        }
        Ok(TuiEvent::Nop)
    }
}

#[derive(Debug)]
pub enum TuiEvent {
    ToggleHelp,
    OpenChat,
    SetMode(String),
    ToggleRecord,
    Quit,
    Snapshot,
    ChatMessage(String),
    Nop,
}
