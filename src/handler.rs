use crate::app::{App, AppResult};
use bytes::Bytes;
use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};

/// Handles the key events and updates the state of [`App`].
pub async fn handle_key_events(key_event: KeyEvent, app: &mut App) -> AppResult<()> {
    match key_event.code {
        // Exit application on `ESC` or `q`
        KeyCode::Esc | KeyCode::Char('q') => {
            app.quit();
        }
        // Exit application on `Ctrl-C`
        KeyCode::Char('c') | KeyCode::Char('C') => {
            if key_event.modifiers == KeyModifiers::CONTROL {
                app.quit();
            }
        }
        KeyCode::Char(input) => app
            .sender_to_terminal
            .send(Bytes::from(input.to_string().into_bytes()))
            .await
            .unwrap(),
        KeyCode::Enter => {
            // Call async function with async loop
            // send result to output channel
            todo!()
        }
        // Other handlers you could add here.
        _ => {}
    }
    Ok(())
}
