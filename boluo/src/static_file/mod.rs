//! 静态文件服务。

mod conditionals;

use std::io::SeekFrom;
use std::ops::Bound;
use std::path::{Path, PathBuf};

use boluo_core::body::Body;
use boluo_core::http::StatusCode;
use boluo_core::request::Request;
use boluo_core::response::{IntoResponse, Response};
use boluo_core::service::Service;
use headers::{
    AcceptRanges, ContentLength, ContentRange, ContentType, HeaderMapExt, LastModified, Range,
};
use tokio::fs::File;
use tokio::io::{AsyncReadExt, AsyncSeekExt};
use tokio_util::io::ReaderStream;

use self::conditionals::{Conditionals, ConditionalsResult};

/// 提供文件的服务。
///
/// # 例子
///
/// ```
/// use boluo::route::Router;
/// use boluo::static_file::ServeFile;
///
/// Router::new().route("/favicon.ico", ServeFile::new("favicon.ico"));
/// ```
#[derive(Debug, Clone)]
pub struct ServeFile {
    path: PathBuf,
}

impl ServeFile {
    /// 使用指定路径创建一个新的 [`ServeFile`]。
    pub fn new(path: impl Into<PathBuf>) -> Self {
        Self { path: path.into() }
    }
}

impl Service<Request> for ServeFile {
    type Response = Response;
    type Error = ServeFileError;

    async fn call(&self, request: Request) -> Result<Self::Response, Self::Error> {
        response_file(&request, &self.path)
            .await
            .map_err(ServeFileError::from_io)
    }
}

/// 提供目录的服务。
///
/// # 例子
///
/// ```
/// use boluo::route::Router;
/// use boluo::static_file::ServeDir;
///
/// Router::new().scope("/static/", ServeDir::new("static"));
/// ```
#[derive(Debug, Clone)]
pub struct ServeDir {
    root: PathBuf,
}

impl ServeDir {
    /// 使用指定路径创建一个新的 [`ServeDir`]。
    pub fn new(root: impl Into<PathBuf>) -> Self {
        Self { root: root.into() }
    }
}

impl Service<Request> for ServeDir {
    type Response = Response;
    type Error = ServeFileError;

    async fn call(&self, request: Request) -> Result<Self::Response, Self::Error> {
        if let Some(path) = sanitize_path(&self.root, request.uri().path()) {
            response_file(&request, &path)
                .await
                .map_err(ServeFileError::from_io)
        } else {
            Err(ServeFileError::NotFound)
        }
    }
}

async fn response_file(request: &Request, path: &Path) -> std::io::Result<Response> {
    let mut file = File::open(path).await?;
    let file_meta = file.metadata().await?;
    let file_size = file_meta.len();
    let file_modified = file_meta.modified().ok().map(LastModified::from);

    let conditionals = Conditionals::from(request.headers());

    let range = match conditionals.check(file_modified) {
        ConditionalsResult::ReturnEarly(response) => return Ok(response),
        ConditionalsResult::Continue(range) => range,
    };
    let Some((start, end)) = parse_range(range, file_size) else {
        let mut response = StatusCode::RANGE_NOT_SATISFIABLE.into_response_always();
        response
            .headers_mut()
            .typed_insert(ContentRange::unsatisfied_bytes(file_size));
        return Ok(response);
    };

    if start != 0 {
        file.seek(SeekFrom::Start(start)).await?;
    }

    let data_size = end - start;
    let data = ReaderStream::new(file.take(data_size));
    let mut response = Body::from_data_stream(data).into_response_always();

    if data_size != file_size {
        *response.status_mut() = StatusCode::PARTIAL_CONTENT;
        response
            .headers_mut()
            .typed_insert(ContentRange::bytes(start..end, file_size).unwrap());
    }

    let mime = mime_guess::from_path(path).first_or_octet_stream();

    response
        .headers_mut()
        .typed_insert(ContentLength(data_size));
    response.headers_mut().typed_insert(ContentType::from(mime));
    response.headers_mut().typed_insert(AcceptRanges::bytes());

    if let Some(last_modified) = file_modified {
        response.headers_mut().typed_insert(last_modified);
    }

    Ok(response)
}

fn parse_range(range: Option<Range>, max_len: u64) -> Option<(u64, u64)> {
    let Some(range) = range else {
        return Some((0, max_len));
    };

    let Some((start, end)) = range.satisfiable_ranges(max_len).next() else {
        return Some((0, max_len));
    };

    let start = match start {
        Bound::Unbounded => 0,
        Bound::Included(s) => s,
        Bound::Excluded(s) => s.saturating_add(1),
    };

    let end = match end {
        Bound::Unbounded => max_len,
        Bound::Included(s) => s.saturating_add(1),
        Bound::Excluded(s) => s,
    };

    if start < end && end <= max_len {
        Some((start, end))
    } else {
        None
    }
}

fn sanitize_path(base: impl AsRef<Path>, tail: &str) -> Option<PathBuf> {
    let mut buf = PathBuf::from(base.as_ref());
    let Ok(tail) = percent_encoding::percent_decode_str(tail).decode_utf8() else {
        return None;
    };
    for seg in tail.split('/') {
        if seg.starts_with("..") || seg.contains('\\') || (cfg!(windows) && seg.contains(':')) {
            return None;
        }
        buf.push(seg);
    }
    Some(buf)
}

/// 提供文件错误。
#[derive(Debug)]
pub enum ServeFileError {
    /// 文件不存在。
    NotFound,
    /// IO 错误。
    Io(std::io::Error),
}

impl std::fmt::Display for ServeFileError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ServeFileError::NotFound => f.write_str("file not found"),
            ServeFileError::Io(e) => write!(f, "IO error: {e}"),
        }
    }
}

impl std::error::Error for ServeFileError {}

impl ServeFileError {
    pub(crate) fn from_io(error: std::io::Error) -> Self {
        match error.kind() {
            std::io::ErrorKind::NotFound => ServeFileError::NotFound,
            _ => ServeFileError::Io(error),
        }
    }
}
