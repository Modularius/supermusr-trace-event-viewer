use std::io::Stdout;

use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};
use ratatui::{
    layout::Rect,
    prelude::CrosstermBackend,
    style::{Color, Style},
    widgets::{List, ListItem},
    Frame,
};

use crate::tui::{traits::{Component}, ComponentStyle, TuiComponent};

pub(crate) struct Results {
    changed: bool,
    index: Option<usize>,
    list: Vec<String>,
}

impl Results {
    pub(crate) fn new() -> TuiComponent<Self> {
        TuiComponent::new(
            Results {
                changed: true,
                index: None,
                list: Default::default(),
            },
            ComponentStyle::selectable(),
        )
        .with_name("results")
    }
}

impl Component for Results {
    fn changed(&self) -> bool {
        self.changed
    }

    fn acknowledge_change(&mut self) {
        self.changed = false;
    }
    
    fn give_focus(&mut self) {}
    
    fn acknowledge_focus(&mut self) {}


    fn handle_key_press(&mut self, key: KeyEvent) {
        if key == KeyEvent::new(KeyCode::Up, KeyModifiers::NONE) {
            self.changed = true;
        } else if key == KeyEvent::new(KeyCode::Down, KeyModifiers::NONE) {
            self.changed = true;
        } else {
        }
    }

    fn update(&mut self) {}

    fn render(&self, frame: &mut Frame<CrosstermBackend<Stdout>>, area: Rect) {
        let list = //List::new(self.list.iter().map(|trace|trace.metadata().expect("").collect::<Vec<_>>()))
        List::new(self.list.iter().map(|trace|ListItem::new(trace.as_str())).collect::<Vec<_>>())
            .highlight_style(Style::new().bg(Color::Cyan))
            .highlight_symbol(">");

        frame.render_widget(list, area);
    }

    fn help(&self) -> &'static str {
        "Use [Tab] to switch, [Up/Down] Arrows to navigate, [Enter] to view trace, and [s] to Save."
    }
}