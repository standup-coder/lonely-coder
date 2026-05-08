use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::{broadcast, mpsc, RwLock};

#[derive(Debug, Clone)]
pub struct TerminalSession {
    pub terminal_id: String,
    pub host_tx: mpsc::Sender<ServerForwardMsg>,
    pub output_rx: broadcast::Receiver<String>,
    pub close_rx: tokio::sync::watch::Receiver<bool>,
}

#[derive(Debug, Clone)]
pub enum ServerForwardMsg {
    KeyInput(String),
    Resize { cols: u16, rows: u16 },
    Chat(String),
    SnapshotRequest,
}

pub struct SessionManager {
    terminals: RwLock<HashMap<String, TerminalHandle>>,
    guests: RwLock<HashMap<String, Vec<GuestHandle>>>,
}

struct TerminalHandle {
    terminal_id: String,
    host_user_id: String,
    output_tx: broadcast::Sender<String>,
    host_tx: mpsc::Sender<ServerForwardMsg>,
    close_tx: tokio::sync::watch::Sender<bool>,
    guest_count: Arc<std::sync::atomic::AtomicU32>,
}

struct GuestHandle {
    guest_id: String,
    user_id: String,
    output_rx: broadcast::Receiver<String>,
}

const MAX_TERMINALS: usize = 200;
const MAX_GUESTS_PER_TERMINAL: usize = 50;
const MAX_ROWS: u16 = 500;
const MAX_COLS: u16 = 500;

impl SessionManager {
    pub fn new() -> Self {
        Self {
            terminals: RwLock::new(HashMap::new()),
            guests: RwLock::new(HashMap::new()),
        }
    }

    pub async fn register_host(
        &self,
        terminal_id: String,
        user_id: String,
        host_tx: mpsc::Sender<ServerForwardMsg>,
    ) -> anyhow::Result<(broadcast::Receiver<String>, tokio::sync::watch::Receiver<bool>)> {
        let mut terminals = self.terminals.write().await;
        if terminals.len() >= MAX_TERMINALS {
            anyhow::bail!("server at capacity ({} terminals)", MAX_TERMINALS);
        }

        let (output_tx, output_rx) = broadcast::channel(256);
        let (close_tx, close_rx) = tokio::sync::watch::channel(false);

        terminals.insert(
            terminal_id.clone(),
            TerminalHandle {
                terminal_id: terminal_id.clone(),
                host_user_id: user_id,
                output_tx,
                host_tx,
                close_tx,
                guest_count: Arc::new(std::sync::atomic::AtomicU32::new(0)),
            },
        );

        Ok((output_rx, close_rx))
    }

    pub async fn register_guest(
        &self,
        terminal_id: &str,
        guest_id: String,
        user_id: String,
    ) -> anyhow::Result<broadcast::Receiver<String>> {
        let terminals = self.terminals.read().await;
        let handle = terminals.get(terminal_id)
            .ok_or_else(|| anyhow::anyhow!("terminal not found"))?;

        let count = handle.guest_count.load(std::sync::atomic::Ordering::Relaxed);
        if count >= MAX_GUESTS_PER_TERMINAL as u32 {
            anyhow::bail!("terminal at capacity ({} guests)", MAX_GUESTS_PER_TERMINAL);
        }

        let output_rx = handle.output_tx.subscribe();
        handle.guest_count.fetch_add(1, std::sync::atomic::Ordering::Relaxed);

        drop(terminals);

        let mut guests = self.guests.write().await;
        guests.entry(terminal_id.to_string())
            .or_insert_with(Vec::new)
            .push(GuestHandle {
                guest_id,
                user_id,
                output_rx: handle.output_tx.subscribe(),
            });

        Ok(output_rx)
    }

    pub async fn forward_to_host(&self, terminal_id: &str, msg: ServerForwardMsg) -> anyhow::Result<()> {
        let terminals = self.terminals.read().await;
        if let Some(handle) = terminals.get(terminal_id) {
            handle.host_tx.send(msg).await?;
            Ok(())
        } else {
            anyhow::bail!("terminal not found")
        }
    }

    pub async fn broadcast_output(&self, terminal_id: &str, data: String) -> anyhow::Result<()> {
        let terminals = self.terminals.read().await;
        if let Some(handle) = terminals.get(terminal_id) {
            let _ = handle.output_tx.send(data);
            Ok(())
        } else {
            anyhow::bail!("terminal not found")
        }
    }

    pub async fn remove_host(&self, terminal_id: &str) {
        let mut terminals = self.terminals.write().await;
        if let Some(handle) = terminals.remove(terminal_id) {
            let _ = handle.close_tx.send(true);
        }
        self.guests.write().await.remove(terminal_id);
    }

    pub async fn guest_count(&self, terminal_id: &str) -> u32 {
        let terminals = self.terminals.read().await;
        terminals.get(terminal_id)
            .map(|h| h.guest_count.load(std::sync::atomic::Ordering::Relaxed))
            .unwrap_or(0)
    }
}
