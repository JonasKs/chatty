use std::{
    fs,
    path::PathBuf,
    sync::{Arc, Mutex as StdMutex},
};
use tokio::sync::Mutex;

use tracing::Level;
use tracing_subscriber::FmtSubscriber;

pub enum Mode {
    Terminal,
    Chat,
}

pub struct AppState {
    pub running: bool,
    pub current_mode: Mode,
    pub tick: i64,
    pub ai_response: String,
    pub terminal_context: Arc<Mutex<String>>,
    pub user_chat_to_send_to_gpt: String,
}

impl AppState {
    pub fn new(terminal_context: Arc<Mutex<String>>) -> Self {
        let s = Self {
            running: true,
            current_mode: Mode::Terminal,
            terminal_context,
            ai_response: "".to_string(),
            tick: 0,
            user_chat_to_send_to_gpt: "".to_string(),
        };
        s.init_panic_hook();
        s
    }

    pub fn change_mode(&mut self) {
        match self.current_mode {
            Mode::Chat => self.current_mode = Mode::Terminal,
            Mode::Terminal => self.current_mode = Mode::Chat,
        }
    }

    pub fn tick(&mut self) {
        self.tick += 1;
    }

    pub fn quit(&mut self) {
        self.running = false;
    }
}

/// Private, init log stuff
impl AppState {
    fn init_panic_hook(&self) {
        let log_file_path = self.get_log_file_path();
        let log_file = fs::File::create(log_file_path).unwrap();
        let subscriber = FmtSubscriber::builder()
            // all spans/events with a level higher than TRACE (e.g, debug, info, warn, etc.)
            // will be written to output path.
            .with_max_level(Level::TRACE)
            .with_writer(StdMutex::new(log_file))
            .with_thread_ids(true)
            .with_ansi(true)
            .with_line_number(true);

        let subscriber = subscriber.finish();
        tracing::subscriber::set_global_default(subscriber)
            .expect("setting default subscriber failed");

        // Set the panic hook to log panic information before panicking
        std::panic::set_hook(Box::new(|panic| {
            let original_hook = std::panic::take_hook();
            tracing::error!("Panic Error: {}", panic);
            crossterm::terminal::disable_raw_mode().expect("Could not disable raw mode");
            crossterm::execute!(std::io::stdout(), crossterm::terminal::LeaveAlternateScreen)
                .expect("Could not leave the alternate screen");

            original_hook(panic);
        }));
        tracing::debug!("Set panic hook")
    }
    fn get_log_file_path(&self) -> PathBuf {
        let log_file_name = "terminal-ai-ops.log".to_string();
        #[cfg(any(target_os = "macos", target_os = "linux"))]
        {
            PathBuf::from("/tmp/").join(log_file_name)
        }
        #[cfg(target_os = "windows")]
        {
            let temp_dir = env::var("TEMP").unwrap_or_else(|_| "C:\\Windows\\Temp".to_string());
            PathBuf::from(temp_dir).join(log_file_name)
        }
    }
}
