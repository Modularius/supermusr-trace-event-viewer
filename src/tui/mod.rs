use ratatui::{buffer::Buffer, layout::Rect, widgets::Widget};

pub(crate) struct App {

}

impl App {
    pub(crate) fn new() -> Self {
        App{}
    }
}

impl Widget for App {
    fn render(self, area: Rect, buf: &mut Buffer) {
        
    }
}

trait Tab : Widget {
    pub(crate) fn new() -> Self {
        App{}
    }
}

struct IntroTab {
}

impl Widget for IntroTab {
    fn render(self, area: Rect, buf: &mut Buffer) {
        
    }
}

impl Tab for IntroTab {
    fn render(self, area: Rect, buf: &mut Buffer) {
        
    }
}