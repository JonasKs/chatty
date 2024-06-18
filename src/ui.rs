use ratatui::{
    layout::{Alignment, Constraint, Direction, Layout},
    style::{Color, Style},
    widgets::{Block, BorderType, Paragraph},
    Frame,
};

use crate::{app::App, widgets::terminal::TerminalWidget};

/// Renders the user interface widgets.
pub fn render(app: &mut App, frame: &mut Frame) {
    let root_box = Layout::default()
        .direction(Direction::Vertical)
        .constraints(vec![Constraint::Fill(1), Constraint::Min(1)])
        .split(frame.size());

    let outer_layout = Layout::default()
        .direction(Direction::Horizontal)
        .constraints(vec![Constraint::Percentage(60), Constraint::Percentage(40)])
        .split(root_box[0]);

    frame.render_widget(app.terminal_widget, outer_layout[0])
}
