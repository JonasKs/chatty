use std::sync::{Arc, RwLock};

use bytes::Bytes;
use color_eyre::eyre::Result;
use tokio::sync::mpsc::Sender;
use tui::{init, restore, setup_pty, Tui};

mod errors;
mod tui;

struct Size {
    cols: u16,
    rows: u16,
}

struct AppState {
    size: Size,
}

#[tokio::main]
async fn main() -> Result<()> {
    let mut terminal = init()?;
    let rx = setup_pty(terminal)?;

    restore(terminal)?;
    Ok(())
}

async fn run(
    terminal: &mut Tui,
    parser: Arc<RwLock<vt100::Parser>>,
    sender: Sender<Bytes>,
) -> Result<()> {
    Ok(())
}
