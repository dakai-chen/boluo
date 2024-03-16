use std::{borrow::Cow, fmt, time::Duration};

#[derive(Debug, Clone, PartialEq, Eq, Hash, Default)]
pub struct Event {
    comment: Option<Cow<'static, str>>,
    retry: Option<Duration>,
    id: Option<Cow<'static, str>>,
    event: Option<Cow<'static, str>>,
    data: Option<Cow<'static, str>>,
}

impl Event {
    pub fn builder() -> EventBuilder {
        EventBuilder::new()
    }
}

impl fmt::Display for Event {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if let Some(comment) = &self.comment {
            writeln!(f, ": {comment}")?;
        }
        if let Some(retry) = &self.retry {
            writeln!(f, "retry: {}", retry.as_millis())?;
        }
        if let Some(id) = &self.id {
            writeln!(f, "id: {id}")?;
        }
        if let Some(event) = &self.event {
            writeln!(f, "event: {event}")?;
        }
        if let Some(data) = &self.data {
            for line in data.lines() {
                writeln!(f, "data: {line}")?;
            }
        }
        writeln!(f)
    }
}

#[derive(Debug)]
pub struct EventBuilder {
    inner: Result<Event, EventValueError>,
}

impl EventBuilder {
    pub fn new() -> Self {
        Self {
            inner: Ok(Event::default()),
        }
    }

    pub fn comment(self, value: impl Into<Cow<'static, str>>) -> Self {
        self.and_then(|mut event| {
            Self::not_contains_newlines_or_carriage_returns(value).map(|value| {
                event.comment = Some(value);
                event
            })
        })
    }

    pub fn retry(self, value: Duration) -> Self {
        Self {
            inner: self.inner.map(|mut event| {
                event.retry = Some(value);
                event
            }),
        }
    }

    pub fn id(self, value: impl Into<Cow<'static, str>>) -> Self {
        self.and_then(|mut event| {
            Self::not_contains_newlines_or_carriage_returns(value).map(|value| {
                event.id = Some(value);
                event
            })
        })
    }

    pub fn event(self, value: impl Into<Cow<'static, str>>) -> Self {
        self.and_then(|mut event| {
            Self::not_contains_newlines_or_carriage_returns(value).map(|value| {
                event.event = Some(value);
                event
            })
        })
    }

    pub fn data(self, value: impl Into<Cow<'static, str>>) -> Self {
        self.and_then(|mut event| {
            let value: Cow<'static, str> = value.into();
            Self::not_contains_carriage_returns(value).map(|value| {
                event.data = Some(value);
                event
            })
        })
    }

    pub fn build(self) -> Result<Event, EventValueError> {
        self.inner
    }

    fn and_then<F>(self, func: F) -> Self
    where
        F: FnOnce(Event) -> Result<Event, EventValueError>,
    {
        Self {
            inner: self.inner.and_then(func),
        }
    }

    fn not_contains_newlines_or_carriage_returns(
        value: impl Into<Cow<'static, str>>,
    ) -> Result<Cow<'static, str>, EventValueError> {
        let value = value.into();
        if memchr::memchr2(b'\r', b'\n', value.as_bytes()).is_none() {
            Ok(value)
        } else {
            Err(EventValueError)
        }
    }

    fn not_contains_carriage_returns(
        value: impl Into<Cow<'static, str>>,
    ) -> Result<Cow<'static, str>, EventValueError> {
        let value = value.into();
        if memchr::memchr(b'\r', value.as_bytes()).is_none() {
            Ok(value)
        } else {
            Err(EventValueError)
        }
    }
}

#[derive(Debug, Clone, Copy)]
pub struct EventValueError;

impl std::fmt::Display for EventValueError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "SSE event value cannot contain newlines or carriage returns"
        )
    }
}

impl std::error::Error for EventValueError {}

#[cfg(test)]
mod tests {
    use std::time::Duration;

    use super::Event;

    #[test]
    fn comment() {
        let event = Event::builder().comment("xx").build().unwrap();
        assert_eq!(format!("{event}"), ": xx\n\n");
    }

    #[test]
    fn retry() {
        let event = Event::builder()
            .retry(Duration::from_secs(1))
            .build()
            .unwrap();
        assert_eq!(format!("{event}"), "retry: 1000\n\n");
    }

    #[test]
    fn id() {
        let event = Event::builder().id("1").build().unwrap();
        assert_eq!(format!("{event}"), "id: 1\n\n");
    }

    #[test]
    fn event() {
        let event = Event::builder().event("message").build().unwrap();
        assert_eq!(format!("{event}"), "event: message\n\n");
    }

    #[test]
    fn data() {
        let event = Event::builder().data("hello\nworld\n").build().unwrap();
        assert_eq!(format!("{event}"), "data: hello\ndata: world\n\n");
    }

    #[test]
    fn all() {
        let event = Event::builder()
            .comment("xx")
            .retry(Duration::from_secs(1))
            .id("1")
            .event("message")
            .data("hello\nworld\n")
            .build()
            .unwrap();
        assert_eq!(
            format!("{event}"),
            ": xx\nretry: 1000\nid: 1\nevent: message\ndata: hello\ndata: world\n\n"
        );
    }
}
