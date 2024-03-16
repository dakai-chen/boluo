use boluo_core::http::StatusCode;
use boluo_core::request::Request;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum RouteErrorKind {
    NotFound,
    MethodNotAllowed,
}

#[derive(Debug)]
pub struct RouteError {
    kind: RouteErrorKind,
    request: Request,
}

impl RouteError {
    #[inline]
    pub fn new(request: Request, kind: RouteErrorKind) -> Self {
        Self { kind, request }
    }

    #[inline]
    pub fn not_found(request: Request) -> Self {
        Self::new(request, RouteErrorKind::NotFound)
    }

    #[inline]
    pub fn method_not_allowed(request: Request) -> Self {
        Self::new(request, RouteErrorKind::MethodNotAllowed)
    }

    #[inline]
    pub fn kind(&self) -> RouteErrorKind {
        self.kind
    }

    #[inline]
    pub fn request_ref(&self) -> &Request {
        &self.request
    }

    #[inline]
    pub fn request_mut(&mut self) -> &mut Request {
        &mut self.request
    }

    #[inline]
    pub fn into_request(self) -> Request {
        self.request
    }
}

impl std::fmt::Display for RouteError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self.kind() {
            RouteErrorKind::NotFound { .. } => write!(f, "{}", StatusCode::NOT_FOUND),
            RouteErrorKind::MethodNotAllowed { .. } => {
                write!(f, "{}", StatusCode::METHOD_NOT_ALLOWED)
            }
        }
    }
}

impl std::error::Error for RouteError {}

#[derive(Debug, Clone)]
pub enum RouterError {
    PathConflict { path: String, message: String },
    InvalidPath { path: String, message: String },
    TooManyPath,
}

impl RouterError {
    pub(super) fn from_matchit_insert_error(path: String, error: matchit::InsertError) -> Self {
        match error {
            matchit::InsertError::Conflict { .. } => RouterError::PathConflict {
                path,
                message: format!("conflict with previously registered path"),
            },
            _ => RouterError::InvalidPath {
                path,
                message: format!("{error}"),
            },
        }
    }
}

impl std::fmt::Display for RouterError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            RouterError::PathConflict { path, message } => {
                write!(f, "path conflict \"{path}\" ({message})")
            }
            RouterError::InvalidPath { path, message } => {
                write!(f, "invalid path \"{path}\" ({message})")
            }
            RouterError::TooManyPath => f.write_str("too many path"),
        }
    }
}

impl std::error::Error for RouterError {}
