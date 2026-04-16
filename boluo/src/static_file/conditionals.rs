use boluo_core::http::{HeaderMap, StatusCode};
use boluo_core::response::{IntoResponse, Response};
use headers::{HeaderMapExt, IfModifiedSince, IfRange, IfUnmodifiedSince, LastModified, Range};

#[derive(Debug)]
pub(crate) struct Conditionals {
    if_modified_since: Option<IfModifiedSince>,
    if_unmodified_since: Option<IfUnmodifiedSince>,
    if_range: Option<IfRange>,
    range: Option<Range>,
}

pub(crate) enum ConditionalsResult {
    ReturnEarly(Response),
    Continue(Option<Range>),
}

impl From<&HeaderMap> for Conditionals {
    fn from(headers: &HeaderMap) -> Self {
        Self {
            if_modified_since: headers.typed_get(),
            if_unmodified_since: headers.typed_get(),
            if_range: headers.typed_get(),
            range: headers.typed_get(),
        }
    }
}

impl Conditionals {
    pub(crate) fn check(self, last_modified: Option<LastModified>) -> ConditionalsResult {
        if let Some(since) = self.if_unmodified_since {
            let precondition = last_modified
                .map(|time| since.precondition_passes(time.into()))
                .unwrap_or(false);

            if !precondition {
                return ConditionalsResult::ReturnEarly(
                    StatusCode::PRECONDITION_FAILED.into_response_always(),
                );
            }
        }
        if let Some(since) = self.if_modified_since {
            let unmodified = last_modified
                .map(|time| !since.is_modified(time.into()))
                // no last_modified means its always modified
                .unwrap_or(false);

            if unmodified {
                return ConditionalsResult::ReturnEarly(
                    StatusCode::NOT_MODIFIED.into_response_always(),
                );
            }
        }
        if let Some(if_range) = self.if_range {
            let can_range = !if_range.is_modified(None, last_modified.as_ref());
            if !can_range {
                return ConditionalsResult::Continue(None);
            }
        }
        ConditionalsResult::Continue(self.range)
    }
}
