use anyhow::Result;
use std::io::Write;
use std::path::PathBuf;
use std::time::Instant;

pub struct AsciiCastWriter {
    file: std::fs::File,
    start_time: Instant,
    width: u16,
    height: u16,
}

pub struct AsciiCastEvent {
    pub time: f64,
    pub event_type: AsciiCastEventType,
    pub data: String,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum AsciiCastEventType {
    Output,
    Input,
    Resize,
}

impl AsciiCastWriter {
    pub fn new(path: &str, width: u16, height: u16) -> Result<Self> {
        let mut file = std::fs::File::create(path)?;
        let header = serde_json::json!({
            "version": 2,
            "width": width,
            "height": height,
            "timestamp": chrono::Utc::now().timestamp(),
        });
        writeln!(file, "{}", header)?;
        Ok(Self {
            file,
            start_time: Instant::now(),
            width,
            height,
        })
    }

    pub fn write_output(&mut self, data: &[u8]) -> Result<()> {
        let elapsed = self.start_time.elapsed().as_secs_f64();
        let text = String::from_utf8_lossy(data).to_string();
        let event = format!(
            "[{:.6}, \"o\", {}]",
            elapsed,
            serde_json::to_string(&text)?
        );
        writeln!(self.file, "{}", event)?;
        self.file.flush()?;
        Ok(())
    }

    pub fn write_input(&mut self, data: &[u8]) -> Result<()> {
        let elapsed = self.start_time.elapsed().as_secs_f64();
        let text = String::from_utf8_lossy(data).to_string();
        let event = format!(
            "[{:.6}, \"i\", {}]",
            elapsed,
            serde_json::to_string(&text)?
        );
        writeln!(self.file, "{}", event)?;
        self.file.flush()?;
        Ok(())
    }

    pub fn write_resize(&mut self, cols: u16, rows: u16) -> Result<()> {
        let elapsed = self.start_time.elapsed().as_secs_f64();
        let event = format!("[{:.6}, \"r\", \"{}x{}\"]", elapsed, cols, rows);
        writeln!(self.file, "{}", event)?;
        self.file.flush()?;
        Ok(())
    }
}

pub struct AsciiCastReader {
    events: Vec<AsciiCastEvent>,
}

impl AsciiCastReader {
    pub fn from_file(path: &PathBuf) -> Result<Self> {
        let content = std::fs::read_to_string(path)?;
        let mut events = Vec::new();
        let mut lines = content.lines();

        if let Some(header_line) = lines.next() {
            let _header: serde_json::Value = serde_json::from_str(header_line)?;
        }

        for line in lines {
            let line = line.trim();
            if line.is_empty() {
                continue;
            }
            let parsed: serde_json::Value = serde_json::from_str(line)?;
            if let Some(arr) = parsed.as_array() {
                if arr.len() >= 3 {
                    let time = arr[0].as_f64().unwrap_or(0.0);
                    let event_type_str = arr[1].as_str().unwrap_or("o");
                    let data = arr[2].as_str().unwrap_or("").to_string();

                    let event_type = match event_type_str {
                        "o" => AsciiCastEventType::Output,
                        "i" => AsciiCastEventType::Input,
                        "r" => AsciiCastEventType::Resize,
                        _ => continue,
                    };

                    events.push(AsciiCastEvent {
                        time,
                        event_type,
                        data,
                    });
                }
            }
        }

        Ok(Self { events })
    }

    pub fn events(&self) -> &[AsciiCastEvent] {
        &self.events
    }
}

pub fn generate_share_path() -> String {
    let timestamp = chrono::Utc::now().format("%Y%m%d_%H%M%S");
    format!("pair_recording_{}.cast", timestamp)
}
