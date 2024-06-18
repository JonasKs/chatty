use ratatui::backend::CrosstermBackend;
use ratatui::widgets::Widget;
use ratatui::Terminal;
use ratatui::{buffer::Buffer, layout::Rect, style::Style};
use std::io::Stderr;
use std::{
    io::{BufWriter, Write},
    sync::{Arc, RwLock},
};
use tui_term::widget::PseudoTerminal;

use bytes::Bytes;
use portable_pty::{CommandBuilder, NativePtySystem, PtySize, PtySystem};
use tokio::{sync::mpsc::channel, sync::mpsc::Sender, task};

pub struct TerminalWidget {
    pub parser: Arc<RwLock<vt100::Parser>>,
    pub sender_to_terminal: Sender<Bytes>,
}

impl Widget for TerminalWidget {
    fn render(self, area: Rect, buf: &mut Buffer) {
        let greeting = format!("Hello, widget!");
        buf.set_string(area.x, area.y, greeting, Style::default());
    }
}

impl TerminalWidget {
    pub fn new_psuedo_terminal(self) -> PseudoTerminal<&'a vt100::Screen> {
        PseudoTerminal::new(self.parser.read().unwrap().screen())
    }

    pub fn new(terminal: &Terminal<CrosstermBackend<Stderr>>) -> TerminalWidget {
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

        let (sender_to_terminal, mut receiver) = channel::<Bytes>(32);
        let mut writer = BufWriter::new(pair.master.take_writer().unwrap());

        // Drop writer on purpose
        tokio::spawn(async move {
            while let Some(bytes) = receiver.recv().await {
                writer.write_all(&bytes).unwrap();
                writer.flush().unwrap();
            }
            drop(pair.master);
        });

        Self {
            sender_to_terminal,
            parser,
        }
    }
}
