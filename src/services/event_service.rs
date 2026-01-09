use std::io;

use crossterm::event::{
    Event as CrosstermEvent, EventStream, KeyCode, KeyEvent, KeyModifiers, MouseEventKind,
};
use futures::StreamExt;
use tokio::{sync::mpsc, time};

#[derive(Debug)]
pub enum Event {
    Tick,
    Key(KeyEvent),
    ChangeMode,
    Quit,
    AIStreamResponse(String),
    AIReasoning(bool),
    // columns, rows
    Resize(u16, u16),
    ScrollUp,
    ScrollDown,
    ScrollUpTerminal,
    ScrollDownTerminal,
}

pub struct EventService {
    crossterm_events: EventStream,
    event_receiver: mpsc::UnboundedReceiver<Event>,
}

impl EventService {
    pub fn new(event_receiver: mpsc::UnboundedReceiver<Event>) -> Self {
        Self {
            crossterm_events: EventStream::new(),
            event_receiver,
        }
    }

    pub async fn next(&mut self) -> io::Result<Event> {
        loop {
            let received_event = tokio::select! {
                event_from_receiver = self.event_receiver.recv() => event_from_receiver,
                crossterm_event = self.crossterm_events.next() => match crossterm_event {
                    Some(Ok(input)) => self.handle_crossterm_event(input),
                    Some(Err(err)) => {println!("{}", err); None},
                    None => {println!("none event"); None},
                },
                _ = time::sleep(time::Duration::from_millis(10)) => Some(Event::Tick),
            };

            if let Some(event) = received_event {
                return Ok(event);
            }
        }
    }

    fn handle_crossterm_event(&self, event: CrosstermEvent) -> Option<Event> {
        match event {
            CrosstermEvent::Mouse(mouse) => {
                if mouse.kind == crossterm::event::MouseEventKind::ScrollUp {
                    Some(Event::ScrollUp)
                } else if mouse.kind == crossterm::event::MouseEventKind::ScrollDown {
                    Some(Event::ScrollDown)
                } else {
                    None
                }
            }
            CrosstermEvent::Key(key) => {
                if key.kind == crossterm::event::KeyEventKind::Press {
                    match key.code {
                        KeyCode::Char('q') if key.modifiers.contains(KeyModifiers::CONTROL) => {
                            tracing::info!("Quitting app : {:?}", key);
                            Some(Event::Quit)
                        }
                        KeyCode::Char('b') if key.modifiers.contains(KeyModifiers::CONTROL) => {
                            tracing::info!("Changing mode : {:?}", key);
                            Some(Event::ChangeMode)
                        }
                        KeyCode::Char('u') if key.modifiers.contains(KeyModifiers::CONTROL) => {
                            tracing::info!("Scroll up : {:?}", key);
                            Some(Event::ScrollUpTerminal)
                        }
                        KeyCode::Char('d') if key.modifiers.contains(KeyModifiers::CONTROL) => {
                            tracing::info!("Scroll down : {:?}", key);
                            Some(Event::ScrollDownTerminal)
                        }
                        KeyCode::Char('U') if key.modifiers.contains(KeyModifiers::CONTROL) => {
                            tracing::info!("Scroll up terminal : {:?}", key);
                            Some(Event::ScrollUpTerminal)
                        }
                        KeyCode::Char('D') if key.modifiers.contains(KeyModifiers::CONTROL) => {
                            tracing::info!("Scroll down terminal : {:?}", key);
                            Some(Event::ScrollDownTerminal)
                        }
                        _ => {
                            tracing::info!("key event {:?}", key);
                            Some(Event::Key(key))
                        }
                    }
                } else {
                    None
                }
            }
            CrosstermEvent::Resize(columns, rows) => Some(Event::Resize(columns, rows)),
            _ => None,
        }
    }
}
