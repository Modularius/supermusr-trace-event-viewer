use supermusr_common::{Channel, DigitizerId};

enum Focus {
    TimestampFrom,
    TimestampTo,
    Channels,
    Digitisers,
}

pub(crate) struct Setup{
    changed: bool,
    focus: Focus,
    from: DateTime<Utc>,
    to: DateTime<Utc>,
    channels: Vec<Channel>,
    digitisers: Vec<DigitizerId>
}

impl Setup {
    pub(crate) fn new(consumer: &BaseConsumer, topics: &Topics) -> Self {
        App{
            changed: true,
            focus: Default::default(),
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

    fn handle_key_press(&mut self, key: KeyEvent) {
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
        
    }
}