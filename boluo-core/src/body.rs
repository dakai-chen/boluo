//! HTTP 主体。

pub use bytes::Bytes;
pub use http_body::{Body as HttpBody, Frame, SizeHint};

use std::any::Any;
use std::borrow::Cow;
use std::pin::Pin;
use std::task::{Context, Poll};

use futures_core::{Stream, TryStream};
use http_body_util::{BodyExt, Empty, Full};

use crate::BoxError;

/// 请求和响应的主体类型。
pub struct Body {
    inner: Pin<Box<dyn AnyHttpBody + Send>>,
}

impl std::fmt::Debug for Body {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Body").finish()
    }
}

impl Body {
    /// 使用给定的 HttpBody 实现创建 Body 实例。
    pub fn new<B>(body: B) -> Self
    where
        B: HttpBody<Data = Bytes> + Send + 'static,
        B::Error: Into<BoxError>,
    {
        crate::util::__try_downcast(body).unwrap_or_else(|body| Self {
            inner: Box::pin(body),
        })
    }

    /// 创建一个空的 Body 实例。
    pub fn empty() -> Self {
        Self::new(Empty::new())
    }

    /// 从流中创建 Body 实例。
    pub fn from_data_stream<S>(stream: S) -> Self
    where
        S: TryStream + Send + 'static,
        S::Ok: Into<Bytes>,
        S::Error: Into<BoxError>,
    {
        Self::new(StreamBody { stream })
    }

    /// 将 Body 转换为仅包含数据帧的流，非数据帧的部分将被丢弃。
    pub fn into_data_stream(self) -> BodyDataStream {
        BodyDataStream { inner: self }
    }

    /// 收集 Body 中的所有数据并合并为单个 Bytes 缓冲区。
    pub async fn to_bytes(self) -> Result<Bytes, BoxError> {
        self.collect().await.map(|col| col.to_bytes())
    }

    /// 尝试将 Body 转换为 `T`，类型不匹配则返回自身。
    ///
    /// 要求 `T: Unpin + 'static`，适用于可安全解除 Pin 的类型。
    pub fn downcast<T>(self) -> Result<T, Self>
    where
        T: Unpin + 'static,
    {
        self.downcast_pin::<T>().map(|v| *Pin::into_inner(v))
    }

    /// 尝试将 Body 转换为对内部类型 `T` 的不可变引用，类型不匹配则返回 `None`。
    pub fn downcast_ref<T>(&self) -> Option<&T>
    where
        T: 'static,
    {
        self.inner.as_any_ref().downcast_ref()
    }

    /// 尝试将 Body 转换为对内部类型 `T` 的可变引用，类型不匹配则返回 `None`。
    ///
    /// 要求 `T: Unpin + 'static`，适用于可安全解除 Pin 的类型。
    pub fn downcast_mut<T>(&mut self) -> Option<&mut T>
    where
        T: Unpin + 'static,
    {
        self.downcast_pin_mut::<T>().map(Pin::get_mut)
    }

    /// 尝试将 Body 转换为 `Pin<Box<T>>`，类型不匹配则返回自身。
    ///
    /// 适用于非 `Unpin` 类型，保留 Pin 约束以保证内存安全。
    pub fn downcast_pin<T>(self) -> Result<Pin<Box<T>>, Self>
    where
        T: 'static,
    {
        if self.inner.as_any_ref().is::<T>() {
            let inner = self.inner.into_any_pin();
            // SAFETY:
            // 解包后仅用于调用 `downcast::<T>()` 做类型转换，无任何移动底层对象的操作。
            // 并且类型转换完成后会立即重新封装为 `Pin<Box<T>>`，恢复 Pin 约束，避免违反 Pin 契约。
            let inner = unsafe { Pin::into_inner_unchecked(inner) };
            let inner = inner.downcast::<T>().unwrap();
            Ok(Pin::from(inner))
        } else {
            Err(self)
        }
    }

    /// 尝试将 Body 转换为对内部类型 `T` 的固定可变引用，类型不匹配则返回 `None`。
    ///
    /// 适用于非 `Unpin` 类型，保留 Pin 约束以保证内存安全。
    pub fn downcast_pin_mut<T>(&mut self) -> Option<Pin<&mut T>>
    where
        T: 'static,
    {
        let inner = self.inner.as_mut();
        // SAFETY:
        // 解包后仅用于调用 `as_any_mut().downcast_mut::<T>()` 做类型转换，无任何移动
        // 底层对象的操作。并且类型转换完成后会立即重新封装为 `Pin<&mut T>`，恢复 Pin 约束，
        // 避免违反 Pin 契约。
        let inner = unsafe { Pin::get_unchecked_mut(inner) };
        inner.as_any_mut().downcast_mut().map(|v| {
            // SAFETY:
            // 重新封装为 `Pin<&mut T>`，恢复 Pin 约束，弥补了解包 Pin 带来的约束暂时缺失，
            // 无任何内存安全风险。
            unsafe { Pin::new_unchecked(v) }
        })
    }
}

impl Default for Body {
    fn default() -> Self {
        Self::empty()
    }
}

impl From<()> for Body {
    fn from(_: ()) -> Self {
        Self::empty()
    }
}

macro_rules! body_from_impl {
    ($ty:ty) => {
        impl From<$ty> for Body {
            fn from(buf: $ty) -> Self {
                Self::new(Full::from(buf))
            }
        }
    };
}

body_from_impl!(&'static [u8]);
body_from_impl!(Cow<'static, [u8]>);
body_from_impl!(Vec<u8>);

body_from_impl!(&'static str);
body_from_impl!(Cow<'static, str>);
body_from_impl!(String);

body_from_impl!(Bytes);

impl HttpBody for Body {
    type Data = Bytes;
    type Error = BoxError;

    #[inline]
    fn poll_frame(
        mut self: Pin<&mut Self>,
        cx: &mut Context<'_>,
    ) -> Poll<Option<Result<Frame<Self::Data>, Self::Error>>> {
        self.inner.as_mut().poll_frame(cx)
    }

    #[inline]
    fn size_hint(&self) -> SizeHint {
        self.inner.size_hint()
    }

    #[inline]
    fn is_end_stream(&self) -> bool {
        self.inner.is_end_stream()
    }
}

/// [`Body`] 的数据帧流。
#[derive(Debug)]
pub struct BodyDataStream {
    inner: Body,
}

impl Stream for BodyDataStream {
    type Item = Result<Bytes, BoxError>;

    #[inline]
    fn poll_next(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Option<Self::Item>> {
        loop {
            match futures_core::ready!(HttpBody::poll_frame(Pin::new(&mut self.inner), cx)?) {
                Some(frame) => match frame.into_data() {
                    Ok(data) => return Poll::Ready(Some(Ok(data))),
                    Err(_frame) => {}
                },
                None => return Poll::Ready(None),
            }
        }
    }
}

impl HttpBody for BodyDataStream {
    type Data = Bytes;
    type Error = BoxError;

    #[inline]
    fn poll_frame(
        mut self: Pin<&mut Self>,
        cx: &mut Context<'_>,
    ) -> Poll<Option<Result<Frame<Self::Data>, Self::Error>>> {
        HttpBody::poll_frame(Pin::new(&mut self.inner), cx)
    }

    #[inline]
    fn is_end_stream(&self) -> bool {
        HttpBody::is_end_stream(&self.inner)
    }

    #[inline]
    fn size_hint(&self) -> SizeHint {
        HttpBody::size_hint(&self.inner)
    }
}

pin_project_lite::pin_project! {
    struct StreamBody<S> {
        #[pin]
        stream: S,
    }
}

impl<S> HttpBody for StreamBody<S>
where
    S: TryStream,
    S::Ok: Into<Bytes>,
    S::Error: Into<BoxError>,
{
    type Data = Bytes;
    type Error = BoxError;

    fn poll_frame(
        self: Pin<&mut Self>,
        cx: &mut Context<'_>,
    ) -> Poll<Option<Result<Frame<Self::Data>, Self::Error>>> {
        let this = self.project();
        match futures_core::ready!(this.stream.try_poll_next(cx)) {
            Some(Ok(chunk)) => Poll::Ready(Some(Ok(Frame::data(chunk.into())))),
            Some(Err(err)) => Poll::Ready(Some(Err(err.into()))),
            None => Poll::Ready(None),
        }
    }
}

trait AnyHttpBody {
    fn poll_frame(
        self: Pin<&mut Self>,
        cx: &mut Context<'_>,
    ) -> Poll<Option<Result<Frame<Bytes>, BoxError>>>;

    fn is_end_stream(&self) -> bool;

    fn size_hint(&self) -> SizeHint;

    fn as_any_ref(&self) -> &dyn Any;

    fn as_any_mut(&mut self) -> &mut dyn Any;

    fn into_any_pin(self: Pin<Box<Self>>) -> Pin<Box<dyn Any>>;
}

impl<T> AnyHttpBody for T
where
    T: HttpBody<Data = Bytes> + Send + 'static,
    T::Error: Into<BoxError>,
{
    fn poll_frame(
        self: Pin<&mut Self>,
        cx: &mut Context<'_>,
    ) -> Poll<Option<Result<Frame<Bytes>, BoxError>>> {
        match self.poll_frame(cx) {
            Poll::Pending => Poll::Pending,
            Poll::Ready(None) => Poll::Ready(None),
            Poll::Ready(Some(Ok(frame))) => Poll::Ready(Some(Ok(frame))),
            Poll::Ready(Some(Err(err))) => Poll::Ready(Some(Err(err.into()))),
        }
    }

    fn is_end_stream(&self) -> bool {
        self.is_end_stream()
    }

    fn size_hint(&self) -> SizeHint {
        self.size_hint()
    }

    fn as_any_ref(&self) -> &dyn Any {
        self
    }

    fn as_any_mut(&mut self) -> &mut dyn Any {
        self
    }

    fn into_any_pin(self: Pin<Box<Self>>) -> Pin<Box<dyn Any>> {
        self
    }
}
