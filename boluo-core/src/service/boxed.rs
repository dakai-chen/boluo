use std::sync::Arc;

use futures_core::future::BoxFuture;

use super::Service;

/// 装箱的[`Service`]特征对象。
///
/// [`BoxService`]将服务转换为特征对象并装箱，允许服务的[`Future`]是动态的。
///
/// 如果需要一个实现[`Clone`]的装箱服务，考虑使用[`BoxCloneService`]或[`ArcService`]。
pub struct BoxService<Req, Res, Err> {
    service: Box<dyn AnyService<Req, Response = Res, Error = Err>>,
}

impl<Req, Res, Err> BoxService<Req, Res, Err> {
    /// 将服务转换为[`Service`]特征对象并装箱。
    pub fn new<S>(service: S) -> Self
    where
        S: Service<Req, Response = Res, Error = Err> + 'static,
    {
        Self {
            service: Box::new(service),
        }
    }
}

impl<Req, Res, Err> Service<Req> for BoxService<Req, Res, Err> {
    type Response = Res;
    type Error = Err;

    fn call(&self, req: Req) -> impl Future<Output = Result<Self::Response, Self::Error>> + Send {
        self.service.call(req)
    }
}

impl<Req, Res, Err> std::fmt::Debug for BoxService<Req, Res, Err> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("BoxService").finish()
    }
}

/// 装箱的[`Service`]特征对象。
///
/// [`ArcService`]将服务转换为特征对象并装箱，允许服务的[`Future`]是动态的，
/// 并允许共享服务。
///
/// 这与[`BoxService`]类似，只是[`ArcService`]实现了[`Clone`]。
pub struct ArcService<Req, Res, Err> {
    service: Arc<dyn AnyService<Req, Response = Res, Error = Err>>,
}

impl<Req, Res, Err> ArcService<Req, Res, Err> {
    /// 将服务转换为[`Service`]特征对象并装箱。
    pub fn new<S>(service: S) -> Self
    where
        S: Service<Req, Response = Res, Error = Err> + 'static,
    {
        Self {
            service: Arc::new(service),
        }
    }
}

impl<Req, Res, Err> Service<Req> for ArcService<Req, Res, Err> {
    type Response = Res;
    type Error = Err;

    fn call(&self, req: Req) -> impl Future<Output = Result<Self::Response, Self::Error>> + Send {
        self.service.call(req)
    }
}

impl<Req, Res, Err> Clone for ArcService<Req, Res, Err> {
    fn clone(&self) -> Self {
        Self {
            service: Arc::clone(&self.service),
        }
    }
}

impl<Req, Res, Err> std::fmt::Debug for ArcService<Req, Res, Err> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("ArcService").finish()
    }
}

/// 装箱的[`Service`]特征对象。
///
/// [`BoxCloneService`]将服务转换为特征对象并装箱，允许服务的[`Future`]是动态的，
/// 并允许克隆服务。
///
/// 这与[`BoxService`]类似，只是[`BoxCloneService`]实现了[`Clone`]。
pub struct BoxCloneService<Req, Res, Err> {
    service: Box<dyn CloneService<Req, Response = Res, Error = Err>>,
}

impl<Req, Res, Err> BoxCloneService<Req, Res, Err> {
    /// 将服务转换为[`Service`]特征对象并装箱。
    pub fn new<S>(service: S) -> Self
    where
        S: Service<Req, Response = Res, Error = Err> + Clone + 'static,
    {
        Self {
            service: Box::new(service),
        }
    }
}

impl<Req, Res, Err> Service<Req> for BoxCloneService<Req, Res, Err> {
    type Response = Res;
    type Error = Err;

    fn call(&self, req: Req) -> impl Future<Output = Result<Self::Response, Self::Error>> + Send {
        self.service.call(req)
    }
}

impl<Req, Res, Err> Clone for BoxCloneService<Req, Res, Err> {
    fn clone(&self) -> Self {
        Self {
            service: self.service.clone_box(),
        }
    }
}

impl<Req, Res, Err> std::fmt::Debug for BoxCloneService<Req, Res, Err> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("BoxCloneService").finish()
    }
}

trait CloneService<Req>: AnyService<Req> {
    fn clone_box(
        &self,
    ) -> Box<dyn CloneService<Req, Response = Self::Response, Error = Self::Error>>;
}

impl<S, Req> CloneService<Req> for S
where
    S: Service<Req> + Clone + 'static,
{
    fn clone_box(
        &self,
    ) -> Box<dyn CloneService<Req, Response = Self::Response, Error = Self::Error>> {
        Box::new(self.clone())
    }
}

trait AnyService<Req>: Send + Sync {
    type Response;
    type Error;

    fn call<'a>(&'a self, req: Req) -> BoxFuture<'a, Result<Self::Response, Self::Error>>
    where
        Req: 'a;
}

impl<S, Req> AnyService<Req> for S
where
    S: Service<Req> + ?Sized,
{
    type Response = S::Response;
    type Error = S::Error;

    fn call<'a>(&'a self, req: Req) -> BoxFuture<'a, Result<Self::Response, Self::Error>>
    where
        Req: 'a,
    {
        Box::pin(Service::call(self, req))
    }
}
