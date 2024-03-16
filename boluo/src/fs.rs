use std::fs::Metadata;
use std::io::{self, SeekFrom};
use std::ops::Bound;
use std::path::{Path, PathBuf};
use std::pin::Pin;
use std::task::Poll;

use boluo_core::body::Body;
use boluo_core::http::StatusCode;
use boluo_core::request::Request;
use boluo_core::response::{IntoResponse, Response};
use boluo_core::service::Service;
use bytes::{Bytes, BytesMut};
use futures_util::future::{self, Either};
use futures_util::{ready, stream, FutureExt, Stream, StreamExt};
use headers::{
    AcceptRanges, ContentLength, ContentRange, ContentType, HeaderMap, HeaderMapExt,
    IfModifiedSince, IfRange, IfUnmodifiedSince, LastModified, Range,
};
use tokio::fs::File as TkFile;
use tokio::io::AsyncSeekExt;
use tokio_util::io::poll_read_buf;

#[derive(Debug, Clone)]
pub struct ServeFile {
    path: PathBuf,
}

impl ServeFile {
    pub fn new(path: impl Into<PathBuf>) -> Self {
        Self { path: path.into() }
    }
}

impl<B> Service<Request<B>> for ServeFile
where
    B: Send,
{
    type Response = Response;
    type Error = FileError;

    async fn call(&self, req: Request<B>) -> Result<Self::Response, Self::Error> {
        response_file(req, &self.path).await
    }
}

#[derive(Debug, Clone)]
pub struct ServeDir {
    root: PathBuf,
}

impl ServeDir {
    pub fn new(root: impl Into<PathBuf>) -> Self {
        Self { root: root.into() }
    }
}

impl<B> Service<Request<B>> for ServeDir
where
    B: Send,
{
    type Response = Response;
    type Error = FileError;

    async fn call(&self, req: Request<B>) -> Result<Self::Response, Self::Error> {
        let Some(path) = sanitize_path(&self.root, req.uri().path()) else {
            return Err(FileError::NotFound);
        };
        response_file(req, &path).await
    }
}

async fn response_file<B: Send>(req: Request<B>, path: &Path) -> Result<Response, FileError> {
    let conditionals = Conditionals::from(req.headers());
    match TkFile::open(path).await {
        Ok(f) => read_file(f, path, conditionals).await,
        Err(e) => {
            let err = match e.kind() {
                io::ErrorKind::NotFound => FileError::NotFound,
                io::ErrorKind::PermissionDenied => FileError::PermissionDenied(e),
                _ => FileError::OpenFailed(e),
            };
            Err(err)
        }
    }
}

fn sanitize_path(base: impl AsRef<Path>, tail: &str) -> Option<PathBuf> {
    let mut buf = PathBuf::from(base.as_ref());
    let Ok(tail) = percent_encoding::percent_decode_str(tail).decode_utf8() else {
        return None;
    };
    for seg in tail.split('/') {
        if seg.starts_with("..") {
            return None;
        } else if seg.contains('\\') {
            return None;
        } else if cfg!(windows) && seg.contains(':') {
            return None;
        } else {
            buf.push(seg);
        }
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

async fn metadata(f: TkFile) -> Result<(TkFile, Metadata), FileError> {
    match f.metadata().await {
        Ok(meta) => Ok((f, meta)),
        Err(err) => Err(match err.kind() {
            io::ErrorKind::NotFound => FileError::NotFound,
            io::ErrorKind::PermissionDenied => FileError::PermissionDenied(err),
            _ => FileError::OpenFailed(err),
        }),
    }
}

async fn read_file(
    f: TkFile,
    path: &Path,
    conditionals: Conditionals,
) -> Result<Response, FileError> {
    let (file, meta) = metadata(f).await?;

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

    let mut resp = Body::from_stream(stream).into_response().unwrap();

    if sub_len != len {
        *resp.status_mut() = StatusCode::PARTIAL_CONTENT;
        resp.headers_mut()
            .typed_insert(ContentRange::bytes(start..end, len).expect("valid ContentRange"));

        len = sub_len;
    }

    let mime = mime_guess::from_path(&path).first_or_octet_stream();

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

    let ret = range
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
                    if s == max_len {
                        s
                    } else {
                        s + 1
                    }
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
        .unwrap_or(Some((0, max_len)));

    ret
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

const DEFAULT_READ_BUF_SIZE: usize = 8_192;

fn optimal_buf_size(metadata: &Metadata) -> usize {
    let block_size = get_block_size(metadata);

    // If file length is smaller than block size, don't waste space
    // reserving a bigger-than-needed buffer.
    std::cmp::min(block_size as u64, metadata.len()) as usize
}

#[cfg(unix)]
fn get_block_size(metadata: &Metadata) -> usize {
    use std::os::unix::fs::MetadataExt;
    //TODO: blksize() returns u64, should handle bad cast...
    //(really, a block size bigger than 4gb?)

    // Use device blocksize unless it's really small.
    std::cmp::max(metadata.blksize() as usize, DEFAULT_READ_BUF_SIZE)
}

#[cfg(not(unix))]
fn get_block_size(_metadata: &Metadata) -> usize {
    DEFAULT_READ_BUF_SIZE
}

#[derive(Debug)]
pub enum FileError {
    NotFound,
    OpenFailed(io::Error),
    PermissionDenied(io::Error),
}

impl std::fmt::Display for FileError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            FileError::NotFound => f.write_str("file not found"),
            FileError::OpenFailed(e) => write!(f, "file open failed: ({})", e),
            FileError::PermissionDenied(e) => {
                write!(f, "file permission denied: ({})", e)
            }
        }
    }
}

impl std::error::Error for FileError {}
