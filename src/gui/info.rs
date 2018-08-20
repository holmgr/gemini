use tui::{backend::MouseBackend, layout::Rect, Terminal};
use tui::{
    layout::{Direction, Group, Size},
    style::{Alignment, Color, Style},
    widgets::{Block, Paragraph, Widget},
};

const TITLE_ART: &str = include_str!("../../res/title.txt");

/// Draws a information page with the given information.
pub fn draw_info_page(term: &mut Terminal<MouseBackend>, area: Rect, information: &str) {
    let page_area = Rect::new((area.width - 80) / 2, (area.height - 40) / 2, 80, 40);
    Group::default()
        .direction(Direction::Vertical)
        .sizes(&[Size::Percent(50), Size::Percent(30)])
        .render(term, &page_area, |term, chunks| {
            Paragraph::default()
                .block(Block::default())
                .style(Style::default().fg(Color::Green))
                .alignment(Alignment::Center)
                .text(TITLE_ART)
                .render(term, &chunks[0]);
            Paragraph::default()
                .block(Block::default())
                .style(Style::default().fg(Color::Yellow))
                .alignment(Alignment::Center)
                .text(information)
                .render(term, &chunks[1]);
        });
}
