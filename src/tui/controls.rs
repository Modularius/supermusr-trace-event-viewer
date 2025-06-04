
pub(crate) struct Controls{
    changed: bool,
    focus: Focus
}

impl Controls {
    pub(crate) fn new() -> Self {
        App{
            changed: true,
        }
    }
}

impl Component for Controls {
    fn changed(&self) -> bool {
        self.changed
    }

    fn acknowledge_change(&mut self) {
        self.changed = false;
    }

    fn handle_key_press(&mut self, key: KeyEvent) {
        if key == KeyEvent::new(KeyCode::Up, KeyModifiers::NONE) {

        } else if key == KeyEvent::new(KeyCode::Down, KeyModifiers::NONE) {

        } else if key == KeyEvent::new(KeyCode::Tab, KeyModifiers::NONE) {
            self.focus = match self.focus {
                Focus::Setup => Focus::Controls,
                Focus::Controls => Focus::Results,
                Focus::Results => Focus::Setup,
            };
            self.changed = true;
        } else {
        }
    }

    fn update(&mut self) {
    }

    fn render(&self, frame: &mut Frame<CrosstermBackend<Stdout>>, area: Rect) {
        let (setup, display) = {
            let chunk = Layout::default()
                .direction(Direction::Vertical)
                .constraints([Constraint::Length(4), Constraint::Min(0)])
                .split(area);
                (chunk[0], chunk[1])
        };

        {
            let block = Block::new()
                .title(Title::default().alignment(Alignment::Center).content("Setup"))
                .borders(Borders::ALL)
                .border_style(Style::new().fg(match self.focus { Focus::Setup => Color::Blue, _ => Color::Black}))
                .style(Style::new().bg(Color::Gray));
            
            frame.render_widget(block, setup);
        }
    }
}