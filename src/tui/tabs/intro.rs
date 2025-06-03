use ratatui::{buffer::Buffer, layout::{Constraint, Direction, Layout, Rect}, widgets::Widget};

use crate::tui::tabs::Page;

struct IntroTab {
}

impl Widget for IntroTab {
    fn render(self, area: Rect, buf: &mut Buffer) {
        
    }
}

impl Page for IntroTab {
}