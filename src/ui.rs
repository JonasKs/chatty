use ratatui::{
    layout::{Alignment, Constraint, Direction, Layout},
    style::{Modifier, Style},
    widgets::{Block, Borders, Paragraph},
    Frame,
};
use tui_term::widget::PseudoTerminal;
use vt100::Screen;

use crate::app::App;

/// Renders the user interface widgets.
pub fn render(_app: &App, frame: &mut Frame, screen: &Screen) {
    let root_box = Layout::default()
        .direction(Direction::Vertical)
        .constraints(vec![Constraint::Fill(1), Constraint::Max(1)])
        .split(frame.size());

    let outer_layout = Layout::default()
        .direction(Direction::Horizontal)
        .constraints(vec![Constraint::Percentage(60), Constraint::Percentage(40)])
        .split(root_box[0]);

    let pseudo_terminal = PseudoTerminal::new(screen);
    frame.render_widget(
        pseudo_terminal.block(Block::default().borders(Borders::RIGHT)),
        outer_layout[0],
    );

    let chat = Paragraph::new("Hello world")
        .style(Style::default().add_modifier(Modifier::BOLD))
        .alignment(Alignment::Center);
    frame.render_widget(chat, outer_layout[1]);

    let explanation = "Press <CTRL>q to exit".to_string();
    let explanation = Paragraph::new(explanation)
        .style(Style::default().add_modifier(Modifier::BOLD | Modifier::REVERSED))
        .alignment(Alignment::Center);
    frame.render_widget(explanation, root_box[1]);
}
