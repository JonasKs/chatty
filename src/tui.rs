use std::{
    io::{self, stdout, BufWriter, Stdout, Write},
    sync::{Arc, RwLock},
};

use bytes::Bytes;
use color_eyre::eyre::Result;
use portable_pty::{CommandBuilder, NativePtySystem, PtySize, PtySystem};
use tokio::{sync::mpsc::channel, sync::mpsc::Receiver, sync::mpsc::Sender, task};

use super::Size;
use crossterm::{execute, terminal::*};
use ratatui::prelude::*;

/// A type alias for the terminal type used in this application
pub type Tui = Terminal<CrosstermBackend<Stdout>>;

/// Initialize the terminal
pub fn init() -> Result<Tui> {
    execute!(stdout(), EnterAlternateScreen)?;
    enable_raw_mode()?;
    let mut terminal = Terminal::new(CrosstermBackend::new(stdout()))?;
    Ok(terminal)
}

pub fn setup_pty(terminal: Tui) -> Result<Sender<Bytes>> {
    let size = Size {
        rows: terminal.size()?.height,
        cols: terminal.size()?.width,
    };

    let pty_system = NativePtySystem::default();
    let cwd = std::env::current_dir().unwrap();
    let mut cmd = CommandBuilder::new_default_prog();
    cmd.cwd(cwd);

    let pair = pty_system
        .openpty(PtySize {
            rows: size.rows,
            cols: size.cols,
            pixel_width: 0,
            pixel_height: 0,
        })
        .unwrap();
    // Wait for the child to complete
    task::spawn_blocking(move || {
        let mut child = pair.slave.spawn_command(cmd).unwrap();
        let _child_exit_status = child.wait().unwrap();
        drop(pair.slave);
    });

    let mut reader = pair.master.try_clone_reader().unwrap();
    let parser = Arc::new(RwLock::new(vt100::Parser::new(size.rows, size.cols, 0)));
    {
        let parser = parser.clone();
        task::spawn_blocking(move || {
            // Consume the output from the child
            // Can't read the full buffer, since that would wait for EOF
            let mut buf = [0u8; 8192];
            let mut processed_buf = Vec::new();
            loop {
                let size = reader.read(&mut buf).unwrap();
                if size == 0 {
                    break;
                }
                if size > 0 {
                    processed_buf.extend_from_slice(&buf[..size]);
                    let mut parser = parser.write().unwrap();
                    parser.process(&processed_buf);

                    // Clear the processed portion of the buffer
                    processed_buf.clear();
                }
            }
        });
    }

    let (tx, mut rx) = channel::<Bytes>(32);
    let mut writer = BufWriter::new(pair.master.take_writer().unwrap());

    // Drop writer on purpose
    tokio::spawn(async move {
        while let Some(bytes) = rx.recv().await {
            writer.write_all(&bytes).unwrap();
            writer.flush().unwrap();
        }
        drop(pair.master);
    });

    Ok(tx)
}

/// Restore the terminal to its original state
pub fn restore(mut terminal: Tui) -> Result<()> {
    execute!(stdout(), LeaveAlternateScreen)?;
    disable_raw_mode()?;
    terminal.show_cursor()?;
    Ok(())
}
