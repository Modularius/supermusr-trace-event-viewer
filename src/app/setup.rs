use std::path::PathBuf;

use chrono::{NaiveDate, NaiveDateTime, NaiveTime, Utc};
use crossterm::event::KeyCode;
use ratatui::{
    layout::{Constraint, Direction, Layout, Rect, Spacing},
    Frame,
};
use strum::{EnumCount, EnumIter, IntoEnumIterator};
use supermusr_common::{Channel, DigitizerId};

use crate::{
    finder::{MessageFinder, SearchMode, SearchTarget},
    graphics::FileFormat,
    tui::{
        ComponentContainer, ComponentStyle, EditBox, FocusableComponent, InputComponent, ListBox,
        ParentalFocusComponent, TuiComponent, TuiComponentBuilder,
    },
    Component, Select, Timestamp,
};

#[derive(Default, Clone, EnumCount, EnumIter)]
pub(crate) enum Focus {
    #[default]
    SearchMode,
    Date,
    Time,
    Number,
    Channel,
    DigitiserId,
    NumPasses,
    MinStepSize,
    StepSizeMul,
    SavePath,
    Format,
    Width,
    Height,
}

pub(crate) struct Setup {
    focus: Focus,
    search_mode: TuiComponent<ListBox<SearchMode>>,
    date: TuiComponent<EditBox<NaiveDate>>,
    time: TuiComponent<EditBox<NaiveTime>>,
    number: TuiComponent<EditBox<usize>>,
    channel: TuiComponent<EditBox<Channel>>,
    digitiser_id: TuiComponent<EditBox<DigitizerId>>,
    num_passes: TuiComponent<EditBox<u32>>,
    min_step_size: TuiComponent<EditBox<i64>>,
    step_size_mul: TuiComponent<EditBox<i64>>,
    save_path: TuiComponent<EditBox<String>>,
    format: TuiComponent<EditBox<FileFormat>>,
    width: TuiComponent<EditBox<u32>>,
    height: TuiComponent<EditBox<u32>>,
}

impl Setup {
    pub(crate) fn new(select: &Select) -> TuiComponent<Self> {
        let comp = Self {
            focus: Default::default(),
            search_mode: ListBox::new(
                &SearchMode::iter().collect::<Vec<_>>(),
                Some("Search Mode"),
                Some(0),
            ),
            date: EditBox::new(select.timestamp.date_naive(), Some("Date (YYYY-MM-DD)")),
            time: EditBox::new(select.timestamp.time(), Some("Time (hh:mm:ss.f)")),
            number: EditBox::new(1, Some("Number to Collect")),
            channel: EditBox::new(1, Some("Channel to Seek")),
            digitiser_id: EditBox::new(4, Some("Digitiser Id to Seek")),
            num_passes: EditBox::new(select.step.num_step_passes, Some("Num Step Passes")),
            min_step_size: EditBox::new(select.step.min_step_size, Some("Min Step Size")),
            step_size_mul: EditBox::new(select.step.step_mul_coef, Some("Step Size Mul Coef")),
            save_path: EditBox::new("out".to_owned(), Some("Save Path")),
            format: EditBox::new(FileFormat::Svg, Some("Image Format")),
            width: EditBox::new(800, Some("Image Width")),
            height: EditBox::new(600, Some("Image Height")),
        };
        let mut setup = TuiComponentBuilder::new(ComponentStyle::default()).build(comp);
        setup.focused_component_mut().set_focus(true);
        setup
    }

    pub(crate) fn search<M: MessageFinder>(&self, message_finder: &mut M) {
        let timestamp = {
            let date = self.date.get();
            let time = self.time.get();
            Timestamp::from_naive_utc_and_offset(
                NaiveDateTime::new(date.clone(), time.clone()),
                Utc,
            )
        };
        let number = *self.number.get();
        let channel = *self.channel.get();
        let digitiser_id = *self.digitiser_id.get();
        if let Some(mode) = self.search_mode.get_value() {
            message_finder.init_search(SearchTarget {
                mode,
                timestamp,
                number,
                channels: vec![channel],
                digitiser_ids: vec![digitiser_id],
            });
        }
    }

    pub(crate) fn get_path(&self) -> PathBuf {
        PathBuf::from(self.save_path.get())
    }

    pub(crate) fn get_image_size(&self) -> (u32, u32) {
        (*self.width.get(), *self.height.get())
    }
}

impl Component for Setup {
    fn render(&self, frame: &mut Frame, area: Rect) {
        // Search Mode Division
        let (search_mode, area) = {
            let chunk = Layout::default()
                .direction(Direction::Horizontal)
                .constraints([Constraint::Length(32), Constraint::Min(0)])
                .spacing(Spacing::Space(6))
                .split(area);
            (chunk[0], chunk[1])
        };

        self.search_mode.render(frame, search_mode);

        // Top/Bottom Division
        let (top, bottom) = {
            let chunk = Layout::default()
                .direction(Direction::Vertical)
                .constraints([Constraint::Ratio(1, 2); 2])
                .split(area);
            (chunk[0], chunk[1])
        };

        //
        // Top Row
        //

        // Date and Time/Search Params Division
        let (datetime, search_params) = {
            let chunk = Layout::default()
                .direction(Direction::Horizontal)
                .constraints([Constraint::Ratio(1, 2); 2])
                .spacing(Spacing::Space(4))
                .split(top);
            (chunk[0], chunk[1])
        };

        // Date/Time Division
        let (date, time) = {
            let chunk = Layout::default()
                .direction(Direction::Horizontal)
                .constraints([Constraint::Ratio(1, 2); 2])
                .split(datetime);
            (chunk[0], chunk[1])
        };
        self.date.render(frame, date);
        self.time.render(frame, time);

        // Search Params Division
        let (number, channel, digitiser_id) = {
            let chunk = Layout::default()
                .direction(Direction::Horizontal)
                .constraints([Constraint::Ratio(1, 3); 3])
                .split(search_params);
            (chunk[0], chunk[1], chunk[2])
        };
        self.number.render(frame, number);
        self.channel.render(frame, channel);
        self.digitiser_id.render(frame, digitiser_id);

        //
        // Bottom Row
        //

        // Settings/Save Division
        let (search_settings, save_settings) = {
            let chunk = Layout::default()
                .direction(Direction::Horizontal)
                .constraints([Constraint::Ratio(1, 2); 2])
                .spacing(Spacing::Space(4))
                .split(bottom);
            (chunk[0], chunk[1])
        };

        // Search Settings Division
        let (num_passes, min_step_size, step_size_mul) = {
            let chunk = Layout::default()
                .direction(Direction::Horizontal)
                .constraints([Constraint::Ratio(1, 3); 3])
                .split(search_settings);
            (chunk[0], chunk[1], chunk[2])
        };
        self.num_passes.render(frame, num_passes);
        self.min_step_size.render(frame, min_step_size);
        self.step_size_mul.render(frame, step_size_mul);

        // Save Settings Division
        let (save_path, format, width, height) = {
            let chunk = Layout::default()
                .direction(Direction::Horizontal)
                .constraints([Constraint::Ratio(1, 4); 4])
                .split(save_settings);
            (chunk[0], chunk[1], chunk[2], chunk[3])
        };
        self.num_passes.render(frame, num_passes);
        self.min_step_size.render(frame, min_step_size);
        self.step_size_mul.render(frame, step_size_mul);

        self.save_path.render(frame, save_path);
        self.format.render(frame, format);
        self.width.render(frame, width);
        self.height.render(frame, height);
    }
}

impl ComponentContainer for Setup {
    type Focus = Focus;

    fn get_focused_component_mut(&mut self, focus: Focus) -> &mut dyn FocusableComponent {
        match focus {
            Focus::SearchMode => &mut self.search_mode,
            Focus::Date => &mut self.date,
            Focus::Time => &mut self.time,
            Focus::Number => &mut self.number,
            Focus::Channel => &mut self.channel,
            Focus::DigitiserId => &mut self.digitiser_id,
            Focus::NumPasses => &mut self.num_passes,
            Focus::MinStepSize => &mut self.min_step_size,
            Focus::StepSizeMul => &mut self.step_size_mul,
            Focus::SavePath => &mut self.save_path,
            Focus::Format => &mut self.format,
            Focus::Width => &mut self.width,
            Focus::Height => &mut self.height,
        }
    }

    fn get_focus(&self) -> Self::Focus {
        self.focus.clone()
    }

    fn set_focus(&mut self, focus: Self::Focus) {
        self.focus = focus;
    }
}

impl InputComponent for Setup {
    fn handle_key_press(&mut self, key: crossterm::event::KeyEvent) {
        if key.code == KeyCode::Right {
            self.set_focus_index(self.focus.clone() as isize + 1);
        } else if key.code == KeyCode::Left {
            self.set_focus_index(self.focus.clone() as isize - 1)
        } else {
            self.focused_component_mut().handle_key_press(key);
        }
    }
}

impl FocusableComponent for Setup {
    fn set_focus(&mut self, focus: bool) {
        self.propagate_parental_focus(focus);
    }
}

impl ParentalFocusComponent for Setup {
    fn propagate_parental_focus(&mut self, focus: bool) {
        self.date.propagate_parental_focus(focus);
        self.date.propagate_parental_focus(focus);
        self.number.propagate_parental_focus(focus);
        self.channel.propagate_parental_focus(focus);
        self.digitiser_id.propagate_parental_focus(focus);
        self.num_passes.propagate_parental_focus(focus);
        self.min_step_size.propagate_parental_focus(focus);
        self.step_size_mul.propagate_parental_focus(focus);
        self.save_path.propagate_parental_focus(focus);
        self.format.propagate_parental_focus(focus);
    }
}
