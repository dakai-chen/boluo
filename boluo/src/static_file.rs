//! 静态文件服务。

use std::fs::Metadata;
use std::io::{self, SeekFrom};
use std::ops::Bound;
use std::path::{Path, PathBuf};
use std::pin::Pin;
use std::task::Poll;

use boluo_core::body::Body;
use boluo_core::http::StatusCode;
use boluo_core::request::{Request, RequestParts};
use boluo_core::response::{IntoResponse, Response};
use boluo_core::service::Service;
use bytes::{Bytes, BytesMut};
use futures_util::future::{self, Either};
use futures_util::{FutureExt, Stream, StreamExt, ready, stream};
use headers::{
    AcceptRanges, ContentLength, ContentRange, ContentType, HeaderMap, HeaderMapExt,
    IfModifiedSince, IfRange, IfUnmodifiedSince, LastModified, Range,
};
use tokio::fs::File as TkFile;
use tokio::io::AsyncSeekExt;
use tokio_util::io::poll_read_buf;

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
        serve_file(request.into_parts(), &self.path).await
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
            serve_file(request.into_parts(), &path).await
        } else {
            Err(ServeFileError::NotFound)
        }
    }
}

async fn serve_file(parts: RequestParts, path: &Path) -> Result<Response, ServeFileError> {
    let conditionals = Conditionals::from(&parts.headers);
    let file = TkFile::open(path).await.map_err(ServeFileError::from_io)?;
    read_file(file, path, conditionals).await
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

#[derive(Debug)]
struct Conditionals {
    if_modified_since: Option<IfModifiedSince>,
    if_unmodified_since: Option<IfUnmodifiedSince>,
    if_range: Option<IfRange>,
    range: Option<Range>,
}

enum Cond {
    NoBody(Response),
    WithBody(Option<Range>),
}

impl Conditionals {
    fn check(self, last_modified: Option<LastModified>) -> Cond {
        if let Some(since) = self.if_unmodified_since {
            let precondition = last_modified
                .map(|time| since.precondition_passes(time.into()))
                .unwrap_or(false);

            if !precondition {
                return Cond::NoBody(StatusCode::PRECONDITION_FAILED.into_response().unwrap());
            }
        }
        if let Some(since) = self.if_modified_since {
            let unmodified = last_modified
                .map(|time| !since.is_modified(time.into()))
                // no last_modified means its always modified
                .unwrap_or(false);

            if unmodified {
                return Cond::NoBody(StatusCode::NOT_MODIFIED.into_response().unwrap());
            }
        }
        if let Some(if_range) = self.if_range {
            let can_range = !if_range.is_modified(None, last_modified.as_ref());
            if !can_range {
                return Cond::WithBody(None);
            }
        }
        Cond::WithBody(self.range)
    }
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

async fn metadata(file: TkFile) -> Result<(TkFile, Metadata), ServeFileError> {
    let meta = file.metadata().await.map_err(ServeFileError::from_io)?;
    Ok((file, meta))
}

async fn read_file(
    file: TkFile,
    path: &Path,
    conditionals: Conditionals,
) -> Result<Response, ServeFileError> {
    let (file, meta) = metadata(file).await?;

    let mut len = meta.len();
    let modified = meta.modified().ok().map(LastModified::from);

    let range = match conditionals.check(modified) {
        Cond::NoBody(resp) => return Ok(resp),
        Cond::WithBody(range) => range,
    };

    let (start, end) = match bytes_range(range, len) {
        Some(range) => range,
        None => {
            let mut resp = StatusCode::RANGE_NOT_SATISFIABLE.into_response().unwrap();
            resp.headers_mut()
                .typed_insert(ContentRange::unsatisfied_bytes(len));
            return Ok(resp);
        }
    };

    let sub_len = end - start;
    let buf_size = optimal_buf_size(&meta);
    let stream = file_to_stream(file, buf_size, (start, end));

    let mut resp = Body::from_data_stream(stream).into_response().unwrap();

    if sub_len != len {
        *resp.status_mut() = StatusCode::PARTIAL_CONTENT;
        resp.headers_mut()
            .typed_insert(ContentRange::bytes(start..end, len).expect("valid ContentRange"));

        len = sub_len;
    }

    let mime = mime_guess::from_path(path).first_or_octet_stream();

    resp.headers_mut().typed_insert(ContentLength(len));
    resp.headers_mut().typed_insert(ContentType::from(mime));
    resp.headers_mut().typed_insert(AcceptRanges::bytes());

    if let Some(last_modified) = modified {
        resp.headers_mut().typed_insert(last_modified);
    }

    Ok(resp)
}

fn bytes_range(range: Option<Range>, max_len: u64) -> Option<(u64, u64)> {
    let range = if let Some(range) = range {
        range
    } else {
        return Some((0, max_len));
    };

    range
        .satisfiable_ranges(max_len)
        .map(|(start, end)| {
            let start = match start {
                Bound::Unbounded => 0,
                Bound::Included(s) => s,
                Bound::Excluded(s) => s + 1,
            };

            let end = match end {
                Bound::Unbounded => max_len,
                Bound::Included(s) => {
                    // For the special case where s == the file size
                    if s == max_len { s } else { s + 1 }
                }
                Bound::Excluded(s) => s,
            };

            if start < end && end <= max_len {
                Some((start, end))
            } else {
                None
            }
        })
        .next()
        .unwrap_or(Some((0, max_len)))
}

fn file_to_stream(
    mut file: TkFile,
    buf_size: usize,
    (start, end): (u64, u64),
) -> impl Stream<Item = Result<Bytes, io::Error>> + Send {
    let seek = async move {
        if start != 0 {
            file.seek(SeekFrom::Start(start)).await?;
        }
        Ok(file)
    };

    seek.into_stream()
        .map(move |result| {
            let mut buf = BytesMut::new();
            let mut len = end - start;

            let mut f = match result {
                Ok(f) => f,
                Err(e) => return Either::Left(stream::once(future::err(e))),
            };

            Either::Right(stream::poll_fn(move |cx| {
                if len == 0 {
                    return Poll::Ready(None);
                }
                reserve_at_least(&mut buf, buf_size);

                let n = match ready!(poll_read_buf(Pin::new(&mut f), cx, &mut buf)) {
                    Ok(n) => n as u64,
                    Err(err) => {
                        return Poll::Ready(Some(Err(err)));
                    }
                };

                if n == 0 {
                    return Poll::Ready(None);
                }

                let mut chunk = buf.split().freeze();
                if n > len {
                    chunk = chunk.split_to(len as usize);
                    len = 0;
                } else {
                    len -= n;
                }

                Poll::Ready(Some(Ok(chunk)))
            }))
        })
        .flatten()
}

fn reserve_at_least(buf: &mut BytesMut, cap: usize) {
    if buf.capacity() - buf.len() < cap {
        buf.reserve(cap);
    }
}

fn optimal_buf_size(metadata: &Metadata) -> usize {
    const DEFAULT_READ_BUF_SIZE: u64 = 8_192;

    // If file length is smaller than block size, don't waste space
    // reserving a bigger-than-needed buffer.
    let size = std::cmp::min(DEFAULT_READ_BUF_SIZE, metadata.len());

    usize::try_from(size).unwrap_or(usize::MAX)
}

/// 提供文件错误。
#[derive(Debug)]
pub enum ServeFileError {
    /// 文件不存在。
    NotFound,
    /// IO 错误。
    IO(io::Error),
}

impl std::fmt::Display for ServeFileError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ServeFileError::NotFound => f.write_str("not found"),
            ServeFileError::IO(e) => write!(f, "io: {e}"),
        }
    }
}

impl std::error::Error for ServeFileError {}

impl ServeFileError {
    fn from_io(error: io::Error) -> Self {
        match error.kind() {
            io::ErrorKind::NotFound => ServeFileError::NotFound,
            _ => ServeFileError::IO(error),
        }
    }
}
