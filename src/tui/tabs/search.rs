use ratatui::{buffer::Buffer, layout::{Constraint, Direction, Layout, Rect}, widgets::Widget};

use crate::tui::tabs::{Page, Tab};

#[derive(Default)]
enum SearchTabState {
    #[default]
    Setup,
    Results,
}

pub(crate) struct SearchTab {
    setup: Tab<SetupSearchTab>,
    results: Tab<ResultsTab>,
    stats: SearchTabState,
}

impl SearchTab {
    pub(crate) fn new() -> Self {
        Self {
            setup: Tab::new(),
            results: Tab::new(),
            stats: Default::default(),
        }
    }
}

impl<'a> Widget for SearchTab {
    fn render(self, area: Rect, buf: &mut Buffer) {
        let (tab_bar, tab_space) = {
            let chunks = Layout::default()
                .direction(Direction::Vertical)
                .constraints([Constraint::Length(1),Constraint::Min(0)])
                .split(area);
            (chunks[0], chunks[1])
        };
        let (search_tab, results_tab) = {
            let chunks = Layout::default()
                .direction(Direction::Horizontal)
                .constraints([Constraint::Percentage(50),Constraint::Percentage(50)])
                .split(tab_bar);
            (chunks[0],chunks[1])
        };
        match self.state {
            
        }

    }
}

impl Page for SearchTab {

}




pub(crate) struct SetupSearchTab {

}

impl Widget for SetupSearchTab {
    fn render(self, area: Rect, buf: &mut Buffer) {
        
    }
}

impl Page for SetupSearchTab {
}



pub(crate) struct ResultsTab {
}

impl Widget for ResultsTab {
    fn render(self, area: Rect, buf: &mut Buffer) {
        
    }
}

impl Page for ResultsTab {
}