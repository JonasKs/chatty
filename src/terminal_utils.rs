use ratatui::backend::CrosstermBackend;
use ratatui::Terminal;
use std::io::{Stderr, Stdout};
use std::{
    io::{BufWriter, Write},
    sync::Arc,
};
use tokio::sync::mpsc::{self, Sender, UnboundedReceiver, UnboundedSender};
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

    let pair = pty_system
        .openpty(PtySize {
            rows: terminal.size().unwrap().height,
            cols: terminal.size().unwrap().width,
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
    let parser = Arc::new(RwLock::new(vt100::Parser::new(
        terminal.size().unwrap().height,
        terminal.size().unwrap().width,
        0,
    )));
    {
        let parser = parser.clone();
        task::spawn(async move {
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
                    let mut parser = parser.write().await;
                    parser.process(&processed_buf);

                    let mut terminal_context = terminal_context.lock().await;
                    *terminal_context = parser.screen().contents();

                    // Clear the processed portion of the buffer
                    processed_buf.clear();
                }
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
