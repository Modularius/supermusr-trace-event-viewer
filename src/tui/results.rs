use crate::data::{DigitiserMetadata, DigitiserTrace};


pub(crate) struct Results<'a> {
    changed: bool,
    index: usize,
    cache: &'a Cache,
}

impl Results {
    pub(crate) fn new() -> Self {
        App{
            changed: true,
        }
    }
}

impl Component for Setup {
    fn changed(&self) -> bool {
        self.changed
    }

    fn acknowledge_change(&mut self) {
        self.changed = false;
    }

    fn handle_key_press(&mut self, key: KeyEvent) -> bool {
        if key == KeyEvent::new(KeyCode::Up, KeyModifiers::NONE) {

        } else if key == KeyEvent::new(KeyCode::Down, KeyModifiers::NONE) {

        } else if key == KeyEvent::new(KeyCode::Tab, KeyModifiers::NONE) {
            self.changed = true;
        } else {
        }

        self.changed
    }

    fn update(&mut self) {
    }

    fn render(&self, frame: &mut Frame<CrosstermBackend<Stdout>>, area: Rect) {
        let list = List::new(self.digitiser_traces.map(|trace|trace.metadata().expect("").collect::<Vec<_>>()))
            .highlight_style(Style::new().bg(Color::Cyan))
            .highlight_symbol(">");

            self.results.render(frame, block.inner(results));
            frame.render_widget(list, results);
    }
}