use std::io;

use crossterm::event::{Event as CrosstermEvent, EventStream, KeyCode, KeyEvent, KeyModifiers};
use futures::StreamExt;
use tokio::{sync::mpsc, time};

pub enum Event {
    Tick,
    Key(KeyEvent),
    ChangeMode,
    Quit,
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
                event = self.event_receiver.recv() => event,
                event = self.crossterm_events.next() => match event {
                    Some(Ok(input)) => self.handle_crossterm_event(input),
                    Some(Err(_)) => None,
                    None => None,
                },
                _ = time::sleep(time::Duration::from_millis(500)) => Some(Event::Tick),
            };

            if let Some(event) = received_event {
                return Ok(event);
            }
        }
    }

    fn handle_crossterm_event(&self, event: CrosstermEvent) -> Option<Event> {
        match event {
            CrosstermEvent::Key(key) => {
                if key.kind == crossterm::event::KeyEventKind::Press {
                    match key.code {
                        KeyCode::Tab => Some(Event::ChangeMode),
                        KeyCode::Char('q') if key.modifiers.contains(KeyModifiers::CONTROL) => {
                            Some(Event::Quit)
                        }
                        _ => Some(Event::Key(key)),
                    }
                } else {
                    None
                }
            }
            _ => None,
        }
    }
}
