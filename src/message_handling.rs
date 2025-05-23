fn unpack_message<'a, F, G, M>(
    m: &'a BorrowedMessage<'a>,
    topic: &str,
    id: F,
    unpack: G,
) -> Option<M>
where
    F: Fn(&'a [u8]) -> bool,
    G: Fn(&'a [u8]) -> Result<M, flatbuffers::InvalidFlatbuffer>,
{
    if m.topic() == topic {
        if let Some(payload) = m.payload() {
            if id(payload) {
                return unpack(payload).ok();
            }
        }
    }
    None
}

fn unpack_trace_message<'a>(
    m: &'a BorrowedMessage<'a>,
    topic: &str,
) -> Option<DigitizerAnalogTraceMessage<'a>> {
    unpack_message(
        m,
        topic,
        digitizer_analog_trace_message_buffer_has_identifier,
        root_as_digitizer_analog_trace_message,
    )
}

fn unpack_event_list_message<'a>(
    m: &'a BorrowedMessage<'a>,
    topic: &str,
) -> Option<DigitizerEventListMessage<'a>> {
    unpack_message(
        m,
        topic,
        digitizer_event_list_message_buffer_has_identifier,
        root_as_digitizer_event_list_message,
    )
}

impl CollectMode {
    pub(crate) fn process_message(&self, cache: &mut Cache, message: &BorrowedMessage) {
        match self.collect {
            Self::Traces => {
                if let Some(msg) = unpack_trace_message(&m, args.trace_topic.as_str()) {
                    cache.push_trace(&msg);
                }
            }
            Self::Events => {
                if let Some(msg) =
                    unpack_event_list_message(&m, args.digitiser_event_topic.as_str())
                {
                    cache.push_events(&msg);
                }
            }
            Self::All => {
                if let Some(msg) = unpack_trace_message(&m, args.trace_topic.as_str()) {
                    cache.push_trace(&msg);
                } else if let Some(msg) =
                    unpack_event_list_message(&m, args.digitiser_event_topic.as_str())
                {
                    cache.push_event_list_to_trace(&msg);
                }
            }
        }
    }

    pub(crate) fn get_count(&self, cache: &Cache) -> usize {
        match self {
            Self::Traces => cache.get_num_traces(),
            Self::Events => cache.get_num_events(),
            Self::All => cache.get_num_traces_with_events()
        }
    }
}