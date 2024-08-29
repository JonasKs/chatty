use ratatui::backend::CrosstermBackend;
use ratatui::Terminal;
use std::io::Stdout;
use std::{
    io::{BufWriter, Write},
    sync::Arc,
};
use tokio::sync::mpsc::{self, Sender};
use tokio::sync::{Mutex, RwLock};

use bytes::Bytes;
use portable_pty::{CommandBuilder, NativePtySystem, PtySize, PtySystem};
use tokio::task;

pub fn new(
    terminal: &Terminal<CrosstermBackend<Stdout>>,
    terminal_context: Arc<Mutex<String>>,
) -> (Arc<RwLock<vt100::Parser>>, Sender<Bytes>) {
    let pty_system = NativePtySystem::default();
    let cwd = std::env::current_dir().unwrap();
    let mut cmd = CommandBuilder::new_default_prog();
    cmd.cwd(cwd);

    let adjusted_width = (terminal.size().unwrap().width as f32 * 0.57).floor() as u16;
    let pair = pty_system
        .openpty(PtySize {
            rows: terminal.size().unwrap().height - 5,
            cols: adjusted_width,
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
    let adjusted_width = (terminal.size().unwrap().width as f32 * 0.57).floor() as u16;
    let parser = Arc::new(RwLock::new(vt100::Parser::new(
        terminal.size().unwrap().height - 5,
        adjusted_width,
        0,
    )));
    {
        let parser = parser.clone();
        task::spawn(async move {
            let mut buf = [0u8; 8192]; // Temporary buffer for each read operation

            loop {
                let size = reader.read(&mut buf).unwrap();

                if size == 0 {
                    break; // Exit loop when EOF is reached
                }

                // Process the current batch of data
                let mut parser = parser.write().await;
                parser.process(&buf[..size]);

                // Convert the newly read data to a String
                let new_data = String::from_utf8_lossy(&buf[..size]);

                // Append the new data to terminal_context
                let mut terminal_context = terminal_context.lock().await;
                terminal_context.push_str(&new_data);

                // Update terminal context with screen contents if needed
                // If you also want to update the screen contents, you can do it here
                // Example: append `parser.screen().contents()` to `terminal_context` as well
            }
        });
    }

    let (terminal_sender, mut terminal_receiver) = mpsc::channel::<Bytes>(32);
    let mut writer = BufWriter::new(pair.master.take_writer().unwrap());

    // Drop writer on purpose
    tokio::spawn(async move {
        while let Some(bytes) = terminal_receiver.recv().await {
            writer.write_all(&bytes).unwrap();
            writer.flush().unwrap();
        }
        drop(pair.master);
    });

    (parser, terminal_sender)
}
