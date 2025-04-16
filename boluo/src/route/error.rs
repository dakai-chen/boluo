use boluo_core::http::StatusCode;
use boluo_core::request::Request;

/// 路由器路由错误的类别。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum RouteErrorKind {
    /// 请求路径不存在。
    NotFound,
    /// 请求方法不存在。
    MethodNotAllowed,
}

/// 路由器路由错误。
#[derive(Debug)]
pub struct RouteError {
    kind: RouteErrorKind,
    request: Request,
}

impl RouteError {
    /// 使用给定的请求和类别创建 [`RouteError`]。
    #[inline]
    pub fn new(request: Request, kind: RouteErrorKind) -> Self {
        Self { kind, request }
    }

    /// 使用给定的请求创建 [`RouteError`]，类别为 [`RouteErrorKind::NotFound`]。
    #[inline]
    pub fn not_found(request: Request) -> Self {
        Self::new(request, RouteErrorKind::NotFound)
    }

    /// 使用给定的请求创建 [`RouteError`]，类别为 [`RouteErrorKind::MethodNotAllowed`]。
    #[inline]
    pub fn method_not_allowed(request: Request) -> Self {
        Self::new(request, RouteErrorKind::MethodNotAllowed)
    }

    /// 返回此错误的类别。
    #[inline]
    pub fn kind(&self) -> RouteErrorKind {
        self.kind
    }

    /// 获取本次请求的引用。
    #[inline]
    pub fn request_ref(&self) -> &Request {
        &self.request
    }

    /// 获取本次请求的可变引用。
    #[inline]
    pub fn request_mut(&mut self) -> &mut Request {
        &mut self.request
    }

    /// 消耗错误，返回本次请求。
    #[inline]
    pub fn into_request(self) -> Request {
        self.request
    }
}

impl std::fmt::Display for RouteError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self.kind() {
            RouteErrorKind::NotFound => write!(f, "{}", StatusCode::NOT_FOUND),
            RouteErrorKind::MethodNotAllowed => {
                write!(f, "{}", StatusCode::METHOD_NOT_ALLOWED)
            }
        }
    }
}

impl std::error::Error for RouteError {}

/// 路由器构建错误。
#[derive(Debug, Clone)]
pub enum RouterError {
    /// 注册的路径冲突。
    PathConflict {
        /// 错误路径。
        path: String,
        /// 错误信息。
        message: String,
    },
    /// 注册的路径无效。
    InvalidPath {
        /// 错误路径。
        path: String,
        /// 错误信息。
        message: String,
    },
    /// 注册的路径超过最大上限。
    TooManyPath,
}

impl RouterError {
    pub(super) fn from_matchit_insert_error(path: String, error: matchit::InsertError) -> Self {
        match error {
            matchit::InsertError::Conflict { .. } => RouterError::PathConflict {
                path,
                message: "conflict with previously registered path".to_owned(),
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
