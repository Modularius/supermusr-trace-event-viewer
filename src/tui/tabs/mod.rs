use ratatui::widgets::Widget;

mod search;
mod intro;

struct Tab<P : Page> {
    name: String,
    page: P,
}

trait Page : Widget {
}