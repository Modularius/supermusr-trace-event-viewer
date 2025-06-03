mod app;
mod tabs;

pub(crate) use app::App;
use ratatui::layout::Rect;

pub(crate) trait Component {
    fn handle_key_press(&mut self) {
    }

    fn update(&mut self) {
    }

    fn render(&self, area: Rect) {
        
    }
}