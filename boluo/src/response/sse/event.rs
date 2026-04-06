use std::borrow::Cow;
use std::time::Duration;

/// 服务器发送的事件。
#[derive(Debug, Clone, PartialEq, Eq, Hash, Default)]
pub struct Event {
    comment: Option<Cow<'static, str>>,
    retry: Option<Duration>,
    id: Option<Cow<'static, str>>,
    event: Option<Cow<'static, str>>,
    data: Option<Cow<'static, str>>,
}

impl Event {
    /// 创建新的构建器以构建事件。
    pub fn builder() -> EventBuilder {
        EventBuilder::new()
    }

    /// 创建一个空事件。
    pub fn new() -> Event {
        Event::default()
    }

    /// 设置事件的注释字段。
    ///
    /// # 恐慌
    ///
    /// 如果设置的值包含换行符或回车符。
    pub fn comment(self, value: impl Into<Cow<'static, str>>) -> Self {
        EventBuilder::from(self).comment(value).build().unwrap()
    }

    /// 设置事件的重试超时字段。
    pub fn retry(self, value: Duration) -> Self {
        EventBuilder::from(self).retry(value).build().unwrap()
    }

    /// 设置事件的标识符字段。
    ///
    /// # 恐慌
    ///
    /// 如果设置的值包含换行符或回车符。
    pub fn id(self, value: impl Into<Cow<'static, str>>) -> Self {
        EventBuilder::from(self).id(value).build().unwrap()
    }

    /// 设置事件的名称字段。
    ///
    /// # 恐慌
    ///
    /// 如果设置的值包含换行符或回车符。
    pub fn event(self, value: impl Into<Cow<'static, str>>) -> Self {
        EventBuilder::from(self).event(value).build().unwrap()
    }

    /// 设置事件的数据字段。
    ///
    /// # 恐慌
    ///
    /// 如果设置的值包含回车符。
    pub fn data(self, value: impl Into<Cow<'static, str>>) -> Self {
        EventBuilder::from(self).data(value).build().unwrap()
    }
}

impl std::fmt::Display for Event {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
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
            if data.is_empty() {
                writeln!(f, "data: ")?;
            } else {
                for line in data.lines() {
                    writeln!(f, "data: {line}")?;
                }
            }
        }
        writeln!(f)
    }
}

/// 事件构建器。
#[derive(Debug)]
pub struct EventBuilder {
    inner: Result<Event, EventValueError>,
}

impl From<Event> for EventBuilder {
    fn from(event: Event) -> Self {
        Self { inner: Ok(event) }
    }
}

impl Default for EventBuilder {
    fn default() -> Self {
        Self::from(Event::default())
    }
}

impl EventBuilder {
    /// 创建构建器实例。
    pub fn new() -> Self {
        Self::default()
    }

    /// 设置事件的注释字段。
    ///
    /// # 错误
    ///
    /// 如果设置的值包含换行符或回车符，则调用 [`EventBuilder::build`] 将返回错误。
    pub fn comment(self, value: impl Into<Cow<'static, str>>) -> Self {
        self.and_then(|mut event| {
            Self::validate_field(value).map(|value| {
                event.comment = Some(value);
                event
            })
        })
    }

    /// 设置事件的重试超时字段。
    pub fn retry(self, value: Duration) -> Self {
        Self {
            inner: self.inner.map(|mut event| {
                event.retry = Some(value);
                event
            }),
        }
    }

    /// 设置事件的标识符字段。
    ///
    /// # 错误
    ///
    /// 如果设置的值包含换行符或回车符，则调用 [`EventBuilder::build`] 将返回错误。
    pub fn id(self, value: impl Into<Cow<'static, str>>) -> Self {
        self.and_then(|mut event| {
            Self::validate_field(value).map(|value| {
                event.id = Some(value);
                event
            })
        })
    }

    /// 设置事件的名称字段。
    ///
    /// # 错误
    ///
    /// 如果设置的值包含换行符或回车符，则调用 [`EventBuilder::build`] 将返回错误。
    pub fn event(self, value: impl Into<Cow<'static, str>>) -> Self {
        self.and_then(|mut event| {
            Self::validate_field(value).map(|value| {
                event.event = Some(value);
                event
            })
        })
    }

    /// 设置事件的数据字段。
    ///
    /// # 错误
    ///
    /// 如果设置的值包含回车符，则调用 [`EventBuilder::build`] 将返回错误。
    pub fn data(self, value: impl Into<Cow<'static, str>>) -> Self {
        self.and_then(|mut event| {
            let value: Cow<'static, str> = value.into();
            Self::validate_data(value).map(|value| {
                event.data = Some(value);
                event
            })
        })
    }

    /// 消耗构建器，构建事件。
    ///
    /// # 错误
    ///
    /// 如果之前配置的任意一个参数发生错误，则在调用此函数时将返回错误。
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

    fn validate_field(
        value: impl Into<Cow<'static, str>>,
    ) -> Result<Cow<'static, str>, EventValueError> {
        let value = value.into();
        if memchr::memchr2(b'\r', b'\n', value.as_bytes()).is_none() {
            Ok(value)
        } else {
            Err(EventValueError { _priv: () })
        }
    }

    fn validate_data(
        value: impl Into<Cow<'static, str>>,
    ) -> Result<Cow<'static, str>, EventValueError> {
        let value = value.into();
        if memchr::memchr(b'\r', value.as_bytes()).is_none() {
            Ok(value)
        } else {
            Err(EventValueError { _priv: () })
        }
    }
}

/// SSE 字段值非法（包含换行符或回车符）。
pub struct EventValueError {
    _priv: (),
}

impl std::fmt::Debug for EventValueError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("EventValueError").finish()
    }
}

impl std::fmt::Display for EventValueError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "SSE field value cannot contain newlines or carriage returns"
        )
    }
}

impl std::error::Error for EventValueError {}

#[cfg(test)]
mod tests {
    use std::time::Duration;

    use super::Event;

    #[test]
    fn empty() {
        let event = Event::new();
        assert_eq!(format!("{event}"), "\n");
    }

    #[test]
    fn comment() {
        let event = Event::new().comment("");
        assert_eq!(format!("{event}"), ": \n\n");

        let event = Event::new().comment("xx");
        assert_eq!(format!("{event}"), ": xx\n\n");
    }

    #[test]
    fn retry() {
        let event = Event::new().retry(Duration::from_secs(1));
        assert_eq!(format!("{event}"), "retry: 1000\n\n");
    }

    #[test]
    fn id() {
        let event = Event::new().id("");
        assert_eq!(format!("{event}"), "id: \n\n");

        let event = Event::new().id("1");
        assert_eq!(format!("{event}"), "id: 1\n\n");
    }

    #[test]
    fn event() {
        let event = Event::new().event("");
        assert_eq!(format!("{event}"), "event: \n\n");

        let event = Event::new().event("message");
        assert_eq!(format!("{event}"), "event: message\n\n");
    }

    #[test]
    fn data() {
        let event = Event::new().data("");
        assert_eq!(format!("{event}"), "data: \n\n");

        let event = Event::new().data("hello\nworld\n");
        assert_eq!(format!("{event}"), "data: hello\ndata: world\n\n");
    }

    #[test]
    fn all() {
        let event = Event::new()
            .comment("")
            .retry(Duration::from_secs(1))
            .id("")
            .event("")
            .data("");
        assert_eq!(
            format!("{event}"),
            ": \nretry: 1000\nid: \nevent: \ndata: \n\n"
        );

        let event = Event::new()
            .comment("xx")
            .retry(Duration::from_secs(1))
            .id("1")
            .event("message")
            .data("hello\nworld\n");
        assert_eq!(
            format!("{event}"),
            ": xx\nretry: 1000\nid: 1\nevent: message\ndata: hello\ndata: world\n\n"
        );
    }
}
