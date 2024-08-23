use ratatui::{
    buffer::Buffer,
    layout::Rect,
    style::Color,
    widgets::{Block, Borders, Paragraph, Widget},
};

use crate::app_state::Message;

pub struct Chat<'a> {
    messages: &'a Vec<Message>,
    pub scroll: u16,
    pub block: Option<Block<'a>>,
}

impl<'a> Chat<'a> {
    pub fn new(messages: &'a Vec<Message>) -> Self {
        Self {
            messages,
            scroll: 0,
            block: None,
        }
    }
    pub fn scroll(mut self, scroll: u16) -> Self {
        self.scroll = scroll;
        self
    }

    pub fn block(mut self, block: Block<'a>) -> Self {
        self.block = Some(block);
        self
    }
}

impl<'a> Widget for &mut Chat<'a> {
    fn render(self, area: Rect, buf: &mut Buffer) {
        let mut y = area.y + 1;
        let mut remaining_height = area.height - 1;
        let mut width = area.width;
        let mut x = area.x;
        if self.block.is_some() {
            self.block.render(area, buf);
            width -= 2;
            x += 1;
        }

        let mut total_height_of_all_messages = 0;
        for message in self.messages.iter() {
            let (_, message_height) = message.paragraph(width);
            total_height_of_all_messages += message_height;
        }

        for message in self.messages.iter() {
            let (paragraph, message_height) = message.paragraph(width);
            tracing::info!(
                "y: {}, message_height: {}, remaining_height: {}",
                y,
                message_height,
                remaining_height
            );
            // If the message height is greater than the remaining height, render partially
            if message_height as u16 >= remaining_height {
                let partial_message_area = Rect {
                    x,
                    y: y.saturating_sub(self.scroll),
                    width,
                    height: remaining_height.saturating_sub(1), // Render only the remaining visible part
                };
                paragraph
                    .style(Color::Yellow)
                    .render(partial_message_area, buf);
                break; // Stop after rendering the partial message
            } else {
                let message_area = Rect {
                    x,
                    y: y.saturating_sub(self.scroll),
                    width,
                    height: message_height as u16,
                };
                paragraph.render(message_area, buf);

                // let block = Block::default().borders(Borders::ALL);
                // block.render(block_rect, buf);
                // Update Y position and remaining height
                y += message_height as u16;
                remaining_height -= message_height as u16;
            }
        }
    }
}
