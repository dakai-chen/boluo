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
    fn any(&mut self, service: ArcService<Request, Response, BoxError>) {
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

#[derive(Debug, Clone)]
pub struct MethodRoute<S> {
    methods: Methods,
    service: S,
}

impl<S> MethodRoute<S> {
    #[inline]
    pub fn any(service: S) -> Self {
        Self {
            methods: Methods::Any,
            service,
        }
    }

    #[inline]
    pub fn one(service: S, method: Method) -> Self {
        Self {
            methods: Methods::One(method),
            service,
        }
    }

    pub fn add(mut self, method: Method) -> Self {
        self.methods = self.methods.add(method);
        self
    }

    pub fn into_service(self) -> S {
        self.service
    }

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

#[inline]
pub fn any<S>(service: S) -> MethodRoute<S> {
    MethodRoute::any(service)
}

macro_rules! impl_method_fn {
    ($name:ident, $method:expr) => {
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

pub trait IntoMethodRoute: private::Sealed {
    type Service;

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
            router.any(boluo_core::util::__into_arc_service(
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
                router.any(boluo_core::util::__into_arc_service(
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
                    if router.contains(&method) {
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
