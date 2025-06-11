use crossterm::event::{KeyCode, KeyEvent};
use ratatui::{layout::{Constraint, Direction, Layout, Rect}, Frame};

use crate::{
    messages::{EventList, Trace}, tui::{ComponentStyle, FocusableComponent, Graph, InputComponent, ParentalFocusComponent, TuiComponent, TuiComponentBuilder}, Component
};

pub(crate) struct Display {
    graph: TuiComponent<Graph>,
}

impl Display {
    pub(crate) fn new() -> TuiComponent<Self> {
        TuiComponentBuilder::new(ComponentStyle::selectable()).build(Self {
            graph: Graph::new(),
        })
    }

    pub(crate) fn select(&mut self, trace_data: &Trace, event_data: Option<&EventList>) {
        self.graph.set(trace_data, event_data)
    }
}

impl ParentalFocusComponent for Display {
    fn propagate_parental_focus(&mut self, focus: bool) {
        self.graph.propagate_parental_focus(focus);
    }
}

impl Component for Display {
    fn render(&self, frame: &mut Frame, area: Rect) {
        let (status, results) = {
            let chunk = Layout::default()
                .direction(Direction::Vertical)
                .constraints([Constraint::Length(4), Constraint::Min(16)])
                .split(area);
            (chunk[0], chunk[1])
        };

        self.graph.render(frame, area);
    }
}

impl InputComponent for Display {
    fn handle_key_press(&mut self, key: KeyEvent) {
        if key.code == KeyCode::Char('+') {
            
        } else if key.code == KeyCode::Char('-') {
            
        } else if key.code == KeyCode::Up {
            
        } else if key.code == KeyCode::Down {
            
        } else if key.code == KeyCode::Left {
            
        } else if key.code == KeyCode::Right {
            
        }
    }
}

impl FocusableComponent for Display {
    fn set_focus(&mut self, focus: bool) {
        self.propagate_parental_focus(focus);
    }
}