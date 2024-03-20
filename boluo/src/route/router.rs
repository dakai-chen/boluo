use std::collections::HashMap;
use std::sync::Arc;

use boluo_core::http::uri::{Parts, Uri};
use boluo_core::middleware::{middleware_fn, Middleware};
use boluo_core::request::Request;
use boluo_core::response::{IntoResponse, Response};
use boluo_core::service::{ArcService, Service};
use boluo_core::BoxError;
use matchit::{Match, MatchError};

use super::method::{MergeToMethodRouter, MethodRouter};
use super::{IntoMethodRoute, MethodRoute, RouteError, RouterError};

pub(super) const PRIVATE_TAIL_PARAM: &'static str = "__private__tail_param";

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Default)]
struct RouteId(u32);

impl RouteId {
    fn next(mut self) -> Option<Self> {
        self.0.checked_add(1).map(|id| {
            self.0 = id;
            self
        })
    }
}

#[derive(Default, Clone)]
struct RouterInner {
    id: RouteId,
    inner: matchit::Router<RouteId>,
    id_to_path: HashMap<RouteId, Arc<str>>,
    path_to_id: HashMap<Arc<str>, RouteId>,
}

impl RouterInner {
    fn at<'m, 'p>(&'m self, path: &'p str) -> Result<Match<'m, 'p, &'m RouteId>, MatchError> {
        self.inner.at(path)
    }

    fn find(&self, path: &str) -> Option<RouteId> {
        self.path_to_id.get(path).copied()
    }

    fn next(&mut self) -> Option<RouteId> {
        self.id.next().map(|id| {
            self.id = id;
            id
        })
    }

    fn add(&mut self, path: &str) -> Result<RouteId, RouterError> {
        let id = self.next().ok_or_else(|| RouterError::TooManyPath)?;

        if let Err(e) = self.inner.insert(path, id) {
            return Err(RouterError::from_matchit_insert_error(path.to_owned(), e));
        }

        let path = Arc::<str>::from(path);
        self.id_to_path.insert(id, path.clone());
        self.path_to_id.insert(path, id);

        Ok(id)
    }
}

#[derive(Debug, Clone)]
enum Endpoint<T> {
    Route(T),
    Scope(T),
}

#[derive(Default, Clone)]
pub struct Router {
    inner: RouterInner,
    table: HashMap<RouteId, Endpoint<MethodRouter>>,
}

impl Router {
    pub fn new() -> Self {
        Default::default()
    }

    pub fn route<S>(self, path: &str, service: S) -> Self
    where
        S: IntoMethodRoute,
        S::Service: Service<Request> + 'static,
        <S::Service as Service<Request>>::Response: IntoResponse,
        <S::Service as Service<Request>>::Error: Into<BoxError>,
    {
        self.try_route(path, service)
            .unwrap_or_else(|e| panic!("{e}"))
    }

    pub fn try_route<S>(self, path: &str, service: S) -> Result<Self, RouterError>
    where
        S: IntoMethodRoute,
        S::Service: Service<Request> + 'static,
        <S::Service as Service<Request>>::Response: IntoResponse,
        <S::Service as Service<Request>>::Error: Into<BoxError>,
    {
        if !path.starts_with('/') {
            return Err(RouterError::InvalidPath {
                path: path.to_owned(),
                message: format!("path must start with a `/`"),
            });
        }
        let path = if let Some((path, "")) = path.rsplit_once("{*}") {
            format!("{path}{{*{PRIVATE_TAIL_PARAM}}}")
        } else {
            path.into()
        };
        self.add_route(
            path,
            Endpoint::Route(
                service
                    .into_method_route()
                    .with(middleware_fn(boluo_core::util::__into_arc_service)),
            ),
        )
    }

    pub fn scope<S>(self, path: &str, service: S) -> Self
    where
        S: IntoMethodRoute,
        S::Service: Service<Request> + 'static,
        <S::Service as Service<Request>>::Response: IntoResponse,
        <S::Service as Service<Request>>::Error: Into<BoxError>,
    {
        self.try_scope(path, service)
            .unwrap_or_else(|e| panic!("{e}"))
    }

    pub fn try_scope<S>(self, path: &str, service: S) -> Result<Self, RouterError>
    where
        S: IntoMethodRoute,
        S::Service: Service<Request> + 'static,
        <S::Service as Service<Request>>::Response: IntoResponse,
        <S::Service as Service<Request>>::Error: Into<BoxError>,
    {
        if !path.starts_with('/') {
            return Err(RouterError::InvalidPath {
                path: path.to_owned(),
                message: format!("path must start with a `/`"),
            });
        }
        let ep = Endpoint::Scope(
            service
                .into_method_route()
                .with(middleware_fn(boluo_core::util::__into_arc_service)),
        );
        if let Some((path, "")) = path.rsplit_once("/{*}") {
            self.add_route(format!("{path}{{*{PRIVATE_TAIL_PARAM}}}"), ep)
        } else if path.ends_with('/') {
            self.add_route(format!("{path}{{*{PRIVATE_TAIL_PARAM}}}"), ep.clone())?
                .add_route(format!("{path}"), ep)
        } else {
            self.add_route(format!("{path}/{{*{PRIVATE_TAIL_PARAM}}}"), ep.clone())?
                .add_route(format!("{path}/"), ep.clone())?
                .add_route(format!("{path}"), ep)
        }
    }

    pub fn mount<S>(self, route: impl Into<Route<S>>) -> Self
    where
        S: Service<Request> + 'static,
        S::Response: IntoResponse,
        S::Error: Into<BoxError>,
    {
        self.try_mount(route).unwrap_or_else(|e| panic!("{e}"))
    }

    pub fn mount_with<S, M>(self, route: impl Into<Route<S>>, middleware: M) -> Self
    where
        M: Middleware<S>,
        M::Service: Service<Request> + 'static,
        <M::Service as Service<Request>>::Response: IntoResponse,
        <M::Service as Service<Request>>::Error: Into<BoxError>,
    {
        route
            .into()
            .with(middleware)
            .try_mount_to(self)
            .unwrap_or_else(|e| panic!("{e}"))
    }

    pub fn try_mount<S>(self, route: impl Into<Route<S>>) -> Result<Self, RouterError>
    where
        S: Service<Request> + 'static,
        S::Response: IntoResponse,
        S::Error: Into<BoxError>,
    {
        route.into().try_mount_to(self)
    }

    pub fn try_mount_with<S, M>(
        self,
        route: impl Into<Route<S>>,
        middleware: M,
    ) -> Result<Self, RouterError>
    where
        M: Middleware<S>,
        M::Service: Service<Request> + 'static,
        <M::Service as Service<Request>>::Response: IntoResponse,
        <M::Service as Service<Request>>::Error: Into<BoxError>,
    {
        route.into().with(middleware).try_mount_to(self)
    }

    pub fn merge(self, other: Router) -> Self {
        self.try_merge(other).unwrap_or_else(|e| panic!("{e}"))
    }

    pub fn merge_with<M>(self, other: Router, middleware: M) -> Self
    where
        M: Middleware<ArcService<Request, Response, BoxError>> + Clone,
        M::Service: Service<Request> + 'static,
        <M::Service as Service<Request>>::Response: IntoResponse,
        <M::Service as Service<Request>>::Error: Into<BoxError>,
    {
        self.try_merge_with(other, middleware)
            .unwrap_or_else(|e| panic!("{e}"))
    }

    pub fn try_merge(self, other: Router) -> Result<Self, RouterError> {
        self.try_merge_with(other, middleware_fn(|s| s))
    }

    pub fn try_merge_with<M>(mut self, other: Router, middleware: M) -> Result<Self, RouterError>
    where
        M: Middleware<ArcService<Request, Response, BoxError>> + Clone,
        M::Service: Service<Request> + 'static,
        <M::Service as Service<Request>>::Response: IntoResponse,
        <M::Service as Service<Request>>::Error: Into<BoxError>,
    {
        for (id, endpoint) in other.table {
            self = self.add_route_with(
                other.inner.id_to_path[&id].as_ref().to_owned(),
                endpoint,
                middleware.clone(),
            )?;
        }
        Ok(self)
    }

    fn add_route<T: MergeToMethodRouter>(
        self,
        path: String,
        endpoint: Endpoint<T>,
    ) -> Result<Self, RouterError> {
        self.add_route_with(path, endpoint, middleware_fn(|s| s))
    }

    fn add_route_with<T: MergeToMethodRouter, M>(
        mut self,
        path: String,
        endpoint: Endpoint<T>,
        middleware: M,
    ) -> Result<Self, RouterError>
    where
        M: Middleware<ArcService<Request, Response, BoxError>> + Clone,
        M::Service: Service<Request> + 'static,
        <M::Service as Service<Request>>::Response: IntoResponse,
        <M::Service as Service<Request>>::Error: Into<BoxError>,
    {
        let id = self.add_path(&path)?;

        let result = match endpoint {
            Endpoint::Route(service) => {
                let Endpoint::Route(router) = self
                    .table
                    .entry(id)
                    .or_insert_with(|| Endpoint::Route(Default::default()))
                else {
                    return Err(RouterError::PathConflict {
                        path,
                        message: format!("conflict with previously registered path"),
                    });
                };
                service.merge_to_with(router, middleware)
            }
            Endpoint::Scope(service) => {
                let Endpoint::Scope(router) = self
                    .table
                    .entry(id)
                    .or_insert_with(|| Endpoint::Scope(Default::default()))
                else {
                    return Err(RouterError::PathConflict {
                        path,
                        message: format!("conflict with previously registered path"),
                    });
                };
                service.merge_to_with(router, middleware)
            }
        };

        if let Err(method) = result {
            let message = match method {
                Some(method) => {
                    format!("conflict with previously registered `{method}` HTTP method")
                }
                None => format!("conflict with previously registered any HTTP method"),
            };
            return Err(RouterError::PathConflict { path, message });
        }

        Ok(self)
    }

    fn add_path(&mut self, path: &str) -> Result<RouteId, RouterError> {
        let id = if let Some(id) = self.inner.find(path) {
            id
        } else {
            self.inner.add(path)?
        };
        Ok(id)
    }
}

impl std::fmt::Debug for Router {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Router").finish()
    }
}

impl Service<Request> for Router {
    type Response = Response;
    type Error = BoxError;

    async fn call(&self, mut req: Request) -> Result<Self::Response, Self::Error> {
        let path = req.uri().path();

        match self.inner.at(path) {
            Ok(Match { value, params }) => {
                let (params, tail) = super::params::prase_path_params(params);
                super::params::insert_path_params(req.extensions_mut(), params);
                match self.table.get(value) {
                    Some(Endpoint::Route(service)) => service.call(req).await,
                    Some(Endpoint::Scope(service)) => {
                        replace_request_path(&mut req, tail.as_deref().unwrap_or_default());
                        service.call(req).await
                    }
                    None => Err(RouteError::not_found(req).into()),
                }
            }
            Err(_) => Err(RouteError::not_found(req).into()),
        }
    }
}

#[derive(Debug, Clone)]
pub struct Route<S> {
    path: String,
    service: MethodRoute<S>,
}

impl<S> Route<S> {
    pub fn new<T>(path: impl Into<String>, service: T) -> Self
    where
        T: IntoMethodRoute<Service = S>,
    {
        Self {
            path: path.into(),
            service: service.into_method_route(),
        }
    }

    pub fn path(&self) -> &str {
        &self.path
    }

    pub fn into_method_route(self) -> MethodRoute<S> {
        self.service
    }

    pub fn into_service(self) -> S {
        self.service.into_service()
    }

    pub fn with<T>(self, middleware: T) -> Route<T::Service>
    where
        T: Middleware<S>,
    {
        Route {
            path: self.path,
            service: self.service.with(middleware),
        }
    }

    fn try_mount_to(self, router: Router) -> Result<Router, RouterError>
    where
        S: Service<Request> + 'static,
        S::Response: IntoResponse,
        S::Error: Into<BoxError>,
    {
        router.try_route(&self.path, self.service)
    }
}

fn replace_request_path(req: &mut Request, path: &str) {
    let uri = req.uri_mut();

    let path = if path.starts_with('/') {
        path[1..].as_ref()
    } else {
        path
    };

    let path_and_query = if let Some(query) = uri.query() {
        format!("/{path}?{query}")
    } else {
        format!("/{path}")
    };

    let mut parts = Parts::default();

    parts.scheme = uri.scheme().cloned();
    parts.authority = uri.authority().cloned();
    parts.path_and_query = Some(path_and_query.parse().unwrap());

    *uri = Uri::from_parts(parts).unwrap();
}
