pub(crate) struct Graph<'a> {
    changed: bool,
    trace: Option<&'a DigitiserTrace>,
}

impl<'a> Graph<'a> {
    pub(crate) fn new() -> Self {
        App{
            changed: true,
        }
    }
}

impl<'a> Component for Graph<'a> {
    fn changed(&self) -> bool {
        self.changed
    }

    fn acknowledge_change(&mut self) {
        self.changed = false;
    }

    fn handle_key_press(&mut self, key: KeyEvent) {
        false
    }

    fn update(&mut self) {
    }

    fn render(&self, frame: &mut Frame<CrosstermBackend<Stdout>>, area: Rect) {
        
    }
}