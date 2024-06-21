use std::collections::{HashMap, HashSet};

use boluo_core::http::Method;
use boluo_core::middleware::{middleware_fn, Middleware};
use boluo_core::request::Request;
use boluo_core::response::{IntoResponse, Response};
use boluo_core::service::{ArcService, Service};
use boluo_core::BoxError;

use super::RouteError;

#[derive(Debug, Default, Clone)]
pub(super) struct MethodRouter {
    map: HashMap<Method, ArcService<Request, Response, BoxError>>,
    any: Option<ArcService<Request, Response, BoxError>>,
}

impl MethodRouter {
    #[inline]
    fn add(&mut self, service: ArcService<Request, Response, BoxError>, method: Method) {
        self.map.insert(method, service);
    }

    #[inline]
    fn add_any(&mut self, service: ArcService<Request, Response, BoxError>) {
        self.any = Some(service);
    }

    #[inline]
    fn contains(&self, method: &Method) -> bool {
        self.map.contains_key(method)
    }

    #[inline]
    fn contains_any(&self) -> bool {
        self.any.is_some()
    }
}

impl Service<Request> for MethodRouter {
    type Response = Response;
    type Error = BoxError;

    async fn call(&self, req: Request) -> Result<Self::Response, Self::Error> {
        fn match_method<'a>(
            router: &'a MethodRouter,
            method: &Method,
        ) -> Option<&'a ArcService<Request, Response, BoxError>> {
            if let Some(service) = router.map.get(method) {
                return Some(service);
            }
            if method == Method::HEAD {
                if let Some(service) = router.map.get(&Method::GET) {
                    return Some(service);
                }
            }
            router.any.as_ref()
        }

        let Some(service) = match_method(self, req.method()) else {
            return Err(RouteError::method_not_allowed(req).into());
        };
        service.call(req).await
    }
}

#[derive(Debug, Clone)]
enum Methods {
    Any,
    One(Method),
    Set(HashSet<Method>),
}

impl Methods {
    fn add(mut self, method: Method) -> Self {
        match self {
            Methods::Any => Methods::One(method),
            Methods::One(m) => Self::Set(HashSet::from([m, method])),
            Methods::Set(ref mut s) => {
                s.insert(method);
                self
            }
        }
    }
}

/// 方法路由。
///
/// 用于向路由器注册服务的类型，描述访问服务的请求方法。
#[derive(Debug, Clone)]
pub struct MethodRoute<S> {
    methods: Methods,
    service: S,
}

impl<S> MethodRoute<S> {
    /// 创建方法路由，服务接收任意方法的请求。
    #[inline]
    fn any(service: S) -> Self {
        Self {
            methods: Methods::Any,
            service,
        }
    }

    /// 创建方法路由，服务接收给定方法的请求。
    #[inline]
    fn one(service: S, method: Method) -> Self {
        Self {
            methods: Methods::One(method),
            service,
        }
    }

    /// 增加访问服务的请求方法。
    ///
    /// 使用[`any`]创建的方法路由调用此函数后，服务不再接收任意方法的请求。
    #[allow(clippy::should_implement_trait)]
    pub fn add(mut self, method: Method) -> Self {
        self.methods = self.methods.add(method);
        self
    }

    /// 消耗方法路由，得到内部服务。
    pub fn into_service(self) -> S {
        self.service
    }

    /// 对方法路由内部的服务应用中间件。
    pub fn with<T>(self, middleware: T) -> MethodRoute<T::Service>
    where
        T: Middleware<S>,
    {
        MethodRoute {
            methods: self.methods,
            service: middleware.transform(self.service),
        }
    }
}

#[inline]
fn method<S>(service: S, method: Method) -> MethodRoute<S> {
    MethodRoute::one(service, method)
}

/// 创建[`MethodRoute`]，服务接收任意方法的请求。
#[inline]
pub fn any<S>(service: S) -> MethodRoute<S> {
    MethodRoute::any(service)
}

macro_rules! impl_method_fn {
    ($name:ident, $method:expr) => {
        #[doc = concat!("创建[`MethodRoute`]，服务接收[`", stringify!($method), "`]请求。")]
        #[inline]
        pub fn $name<S>(service: S) -> MethodRoute<S> {
            method(service, $method)
        }
    };
}

impl_method_fn!(connect, Method::CONNECT);
impl_method_fn!(delete, Method::DELETE);
impl_method_fn!(get, Method::GET);
impl_method_fn!(head, Method::HEAD);
impl_method_fn!(options, Method::OPTIONS);
impl_method_fn!(patch, Method::PATCH);
impl_method_fn!(post, Method::POST);
impl_method_fn!(put, Method::PUT);
impl_method_fn!(trace, Method::TRACE);

mod private {
    use super::{MethodRoute, Request, Service};

    pub trait Sealed {}

    impl<S> Sealed for S where S: Service<Request> {}
    impl<S> Sealed for MethodRoute<S> {}
}

/// 用于生成[`MethodRoute`]的特征。
pub trait IntoMethodRoute: private::Sealed {
    /// 返回的[`MethodRoute`]内部的服务类型。
    type Service;

    /// 得到一个[`MethodRoute`]实例。
    fn into_method_route(self) -> MethodRoute<Self::Service>;
}

impl<S> IntoMethodRoute for S
where
    S: Service<Request>,
{
    type Service = S;

    #[inline]
    fn into_method_route(self) -> MethodRoute<Self::Service> {
        MethodRoute::any(self)
    }
}

impl<S> IntoMethodRoute for MethodRoute<S> {
    type Service = S;

    #[inline]
    fn into_method_route(self) -> MethodRoute<S> {
        self
    }
}

pub(super) trait MergeToMethodRouter: Sized {
    #[allow(dead_code)]
    fn merge_to(self, router: &mut MethodRouter) -> Result<(), Option<Method>> {
        self.merge_to_with(router, middleware_fn(|s| s))
    }

    fn merge_to_with<M>(
        self,
        router: &mut MethodRouter,
        middleware: M,
    ) -> Result<(), Option<Method>>
    where
        M: Middleware<ArcService<Request, Response, BoxError>> + Clone,
        M::Service: Service<Request> + 'static,
        <M::Service as Service<Request>>::Response: IntoResponse,
        <M::Service as Service<Request>>::Error: Into<BoxError>;
}

impl MergeToMethodRouter for MethodRouter {
    fn merge_to_with<M>(
        self,
        router: &mut MethodRouter,
        middleware: M,
    ) -> Result<(), Option<Method>>
    where
        M: Middleware<ArcService<Request, Response, BoxError>> + Clone,
        M::Service: Service<Request> + 'static,
        <M::Service as Service<Request>>::Response: IntoResponse,
        <M::Service as Service<Request>>::Error: Into<BoxError>,
    {
        for method in self.map.keys() {
            if router.contains(method) {
                return Err(Some(method.clone()));
            }
        }
        if let Some(service) = self.any {
            if router.contains_any() {
                return Err(None);
            }
            router.add_any(boluo_core::util::__into_arc_service(
                middleware.clone().transform(service),
            ));
        }
        for (method, service) in self.map {
            router.add(
                boluo_core::util::__into_arc_service(middleware.clone().transform(service)),
                method,
            );
        }
        Ok(())
    }
}

impl MergeToMethodRouter for MethodRoute<ArcService<Request, Response, BoxError>> {
    fn merge_to_with<M>(
        self,
        router: &mut MethodRouter,
        middleware: M,
    ) -> Result<(), Option<Method>>
    where
        M: Middleware<ArcService<Request, Response, BoxError>> + Clone,
        M::Service: Service<Request> + 'static,
        <M::Service as Service<Request>>::Response: IntoResponse,
        <M::Service as Service<Request>>::Error: Into<BoxError>,
    {
        match self.methods {
            Methods::Any => {
                if router.contains_any() {
                    return Err(None);
                }
                router.add_any(boluo_core::util::__into_arc_service(
                    middleware.transform(self.service),
                ));
            }
            Methods::One(method) => {
                if router.contains(&method) {
                    return Err(Some(method));
                }
                router.add(
                    boluo_core::util::__into_arc_service(middleware.transform(self.service)),
                    method,
                );
            }
            Methods::Set(methods) => {
                for method in methods.iter() {
                    if router.contains(method) {
                        return Err(Some(method.clone()));
                    }
                }
                for method in methods {
                    router.add(
                        boluo_core::util::__into_arc_service(
                            middleware.clone().transform(self.service.clone()),
                        ),
                        method,
                    );
                }
            }
        }
        Ok(())
    }
}
