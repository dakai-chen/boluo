//! 服务的特征和相关类型的定义。

mod and_then;
mod boxed;
mod ext;
mod map_err;
mod map_request;
mod map_response;
mod map_result;
mod or_else;
mod service_fn;
mod then;

pub use and_then::AndThen;
pub use boxed::{ArcService, BoxCloneService, BoxService};
pub use ext::ServiceExt;
pub use map_err::MapErr;
pub use map_request::MapRequest;
pub use map_response::MapResponse;
pub use map_result::MapResult;
pub use or_else::OrElse;
pub use service_fn::{ServiceFn, service_fn};
pub use then::Then;

use std::sync::Arc;

/// 表示一个接收请求并返回响应的异步函数。
///
/// # 例子
///
/// ```
/// use std::convert::Infallible;
///
/// use boluo_core::request::Request;
/// use boluo_core::response::Response;
/// use boluo_core::service::Service;
///
/// // 回声服务，响应请求主体。
/// struct Echo;
///
/// impl Service<Request> for Echo {
///     type Response = Response;
///     type Error = Infallible;
///
///     async fn call(&self, req: Request) -> Result<Self::Response, Self::Error> {
///         Ok(Response::new(req.into_body()))
///     }
/// }
/// ```
pub trait Service<Req>: Send + Sync {
    /// 服务返回的响应。
    type Response;

    /// 服务产生的错误。
    type Error;

    /// 处理请求并异步返回响应。
    fn call(&self, req: Req) -> impl Future<Output = Result<Self::Response, Self::Error>> + Send;
}

impl<S, Req> Service<Req> for &mut S
where
    S: Service<Req> + ?Sized,
{
    type Response = S::Response;
    type Error = S::Error;

    fn call(&self, req: Req) -> impl Future<Output = Result<Self::Response, Self::Error>> + Send {
        S::call(self, req)
    }
}

impl<S, Req> Service<Req> for &S
where
    S: Service<Req> + ?Sized,
{
    type Response = S::Response;
    type Error = S::Error;

    fn call(&self, req: Req) -> impl Future<Output = Result<Self::Response, Self::Error>> + Send {
        S::call(self, req)
    }
}

impl<S, Req> Service<Req> for Box<S>
where
    S: Service<Req> + ?Sized,
{
    type Response = S::Response;
    type Error = S::Error;

    fn call(&self, req: Req) -> impl Future<Output = Result<Self::Response, Self::Error>> + Send {
        S::call(self, req)
    }
}

impl<S, Req> Service<Req> for Arc<S>
where
    S: Service<Req> + ?Sized,
{
    type Response = S::Response;
    type Error = S::Error;

    fn call(&self, req: Req) -> impl Future<Output = Result<Self::Response, Self::Error>> + Send {
        S::call(self, req)
    }
}
