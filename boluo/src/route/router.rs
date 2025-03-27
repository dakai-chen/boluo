use std::collections::HashMap;
use std::sync::Arc;

use boluo_core::BoxError;
use boluo_core::http::Method;
use boluo_core::http::uri::Uri;
use boluo_core::middleware::{Middleware, middleware_fn};
use boluo_core::request::Request;
use boluo_core::response::{IntoResponse, Response};
use boluo_core::service::{ArcService, Service};
use matchit::{Match, MatchError};

use super::method::{MergeToMethodRouter, MethodRouter};
use super::{IntoMethodRoute, MethodRoute, RouteError, RouterError};

pub(super) const PRIVATE_TAIL_PARAM: &str = "__private__boluo_tail_param";
pub(super) const PRIVATE_TAIL_PARAM_CAPTURE: &str = "{*__private__boluo_tail_param}";

fn normalize_tail_param_capture(path: &str) -> String {
    if let Some(path) = path.strip_suffix("{*}") {
        format!("{path}{PRIVATE_TAIL_PARAM_CAPTURE}")
    } else {
        path.to_owned()
    }
}

type RouteEntry<'a> = (
    &'a str,
    Option<&'a Method>,
    Endpoint<&'a ArcService<Request, Response, BoxError>>,
);

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Default)]
struct RouteId(u32);

impl RouteId {
    #[inline]
    fn next(self) -> Option<Self> {
        self.0.checked_add(1).map(Self)
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
    fn match_route<'m, 'p>(
        &'m self,
        path: &'p str,
    ) -> Result<Match<'m, 'p, &'m RouteId>, MatchError> {
        self.inner.at(path)
    }

    fn get_path(&self, id: RouteId) -> Option<&str> {
        self.id_to_path.get(&id).map(Arc::as_ref)
    }

    fn get_id(&self, path: &str) -> Option<RouteId> {
        self.path_to_id.get(path).copied()
    }

    fn get_or_create_id(&mut self, path: &str) -> Result<RouteId, RouterError> {
        if let Some(id) = self.get_id(path) {
            Ok(id)
        } else {
            self.__insert(path)
        }
    }

    fn remove(&mut self, path: &str) -> Option<RouteId> {
        self.path_to_id.remove(path).inspect(|id| {
            self.id_to_path.remove(id);
            self.inner.remove(normalize_tail_param_capture(path));
        })
    }

    #[inline]
    fn generate_next_id(&mut self) -> Option<RouteId> {
        self.id.next().inspect(|&id| {
            self.id = id;
        })
    }

    /// 仅供 `RouterInner` 内部使用，不检查路径是否存在。
    fn __insert(&mut self, path: &str) -> Result<RouteId, RouterError> {
        let id = self.generate_next_id().ok_or(RouterError::TooManyPath)?;

        if let Err(e) = self.inner.insert(normalize_tail_param_capture(path), id) {
            return Err(RouterError::from_matchit_insert_error(path.to_owned(), e));
        }

        let shared_path = Arc::<str>::from(path);
        self.id_to_path.insert(id, shared_path.clone());
        self.path_to_id.insert(shared_path, id);

        Ok(id)
    }
}

/// 路由端点。
#[derive(Debug, Clone, Copy)]
pub enum Endpoint<T> {
    /// 普通路由。
    Route(T),
    /// 嵌套路由。
    Scope(T),
}

impl<T> AsRef<T> for Endpoint<T> {
    #[inline]
    fn as_ref(&self) -> &T {
        match self {
            Endpoint::Route(v) => v,
            Endpoint::Scope(v) => v,
        }
    }
}

impl<T> AsMut<T> for Endpoint<T> {
    #[inline]
    fn as_mut(&mut self) -> &mut T {
        match self {
            Endpoint::Route(v) => v,
            Endpoint::Scope(v) => v,
        }
    }
}

impl<T> Endpoint<T> {
    /// 得到端点内部的值。
    #[inline]
    pub fn into_inner(self) -> T {
        match self {
            Endpoint::Route(v) => v,
            Endpoint::Scope(v) => v,
        }
    }
}

/// 路由器。
///
/// # 例子
///
/// ```
/// use boluo::handler::handler_fn;
/// use boluo::route::Router;
///
/// #[boluo::route("/f", method = "GET")]
/// async fn f() -> &'static str {
///     "f"
/// }
///
/// let ab = Router::new()
///     .route("/a", handler_fn(|| async { "a" }))
///     .route("/b", handler_fn(|| async { "b" }));
///
/// let cd = Router::new()
///     .route("/c", handler_fn(|| async { "c" }))
///     .route("/d", handler_fn(|| async { "d" }));
///
/// Router::new()
///     // 路由。
///     .route("/a", handler_fn(|| async { "a" }))
///     .route("/b", handler_fn(|| async { "b" }))
///     // 嵌套路由。
///     .scope("/x", ab)
///     // 将其他路由器的路由合并到当前路由器。
///     .merge(cd)
///     // 挂载宏定义路由。
///     .mount(f);
/// ```
#[derive(Default, Clone)]
pub struct Router {
    inner: RouterInner,
    table: HashMap<RouteId, Endpoint<MethodRouter>>,
}

impl Router {
    /// 创建一个空的路由器。
    pub fn new() -> Self {
        Default::default()
    }

    /// 将服务添加到指定路径。
    ///
    #[doc = include_str!("../../doc/route/route.md")]
    ///
    /// # 恐慌
    ///
    /// 给定了无效路径或路由表发生冲突时会出现恐慌。
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

    /// 尝试将服务添加到指定路径。
    ///
    #[doc = include_str!("../../doc/route/route.md")]
    ///
    /// # 错误
    ///
    /// 给定了无效路径或路由表发生冲突时会返回错误。
    pub fn try_route<S>(self, path: &str, service: S) -> Result<Self, RouterError>
    where
        S: IntoMethodRoute,
        S::Service: Service<Request> + 'static,
        <S::Service as Service<Request>>::Response: IntoResponse,
        <S::Service as Service<Request>>::Error: Into<BoxError>,
    {
        Self::validate_path(path)?;

        let ep = Endpoint::Route(
            service
                .into_method_route()
                .with(middleware_fn(boluo_core::util::__into_arc_service)),
        );

        self.add_endpoint(path, ep)
    }

    /// 将服务嵌套到指定路径并去掉前缀，新路径总是以`/`开头。
    ///
    #[doc = include_str!("../../doc/route/scope.md")]
    ///
    /// # 恐慌
    ///
    /// 给定了无效路径或路由表发生冲突时会出现恐慌。
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

    /// 尝试将服务嵌套到指定路径并去掉前缀，新路径总是以`/`开头。
    ///
    #[doc = include_str!("../../doc/route/scope.md")]
    ///
    /// # 错误
    ///
    /// 给定了无效路径或路由表发生冲突时会返回错误。
    pub fn try_scope<S>(self, path: &str, service: S) -> Result<Self, RouterError>
    where
        S: IntoMethodRoute,
        S::Service: Service<Request> + 'static,
        <S::Service as Service<Request>>::Response: IntoResponse,
        <S::Service as Service<Request>>::Error: Into<BoxError>,
    {
        Self::validate_path(path)?;

        let ep = Endpoint::Scope(
            service
                .into_method_route()
                .with(middleware_fn(boluo_core::util::__into_arc_service)),
        );

        if path.ends_with("/{*}") {
            self.add_endpoint(path, ep)
        } else if path.ends_with('/') {
            self.add_endpoint(&format!("{path}{{*}}"), ep.clone())?
                .add_endpoint(path, ep)
        } else {
            self.add_endpoint(&format!("{path}/{{*}}"), ep.clone())?
                .add_endpoint(&format!("{path}/"), ep.clone())?
                .add_endpoint(path, ep)
        }
    }

    /// 将[`Route`](Route)对象注册到路由器，这通常和[`route`]宏配合使用。
    ///
    /// # 恐慌
    ///
    /// 给定了无效路径或路由表发生冲突时会出现恐慌。
    ///
    /// [`route`]: macro@boluo_macros::route
    pub fn mount<S>(self, route: impl Into<Route<S>>) -> Self
    where
        S: Service<Request> + 'static,
        S::Response: IntoResponse,
        S::Error: Into<BoxError>,
    {
        self.try_mount(route).unwrap_or_else(|e| panic!("{e}"))
    }

    /// 将[`Route`](Route)对象注册到路由器，并对服务应用中间件，这通常和[`route`]宏配合使用。
    ///
    /// # 恐慌
    ///
    /// 给定了无效路径或路由表发生冲突时会出现恐慌。
    ///
    /// [`route`]: macro@boluo_macros::route
    pub fn mount_with<S, M>(self, route: impl Into<Route<S>>, middleware: M) -> Self
    where
        M: Middleware<S>,
        M::Service: Service<Request> + 'static,
        <M::Service as Service<Request>>::Response: IntoResponse,
        <M::Service as Service<Request>>::Error: Into<BoxError>,
    {
        self.try_mount_with(route, middleware)
            .unwrap_or_else(|e| panic!("{e}"))
    }

    /// 尝试将[`Route`](Route)对象注册到路由器，这通常和[`route`]宏配合使用。
    ///
    /// # 错误
    ///
    /// 给定了无效路径或路由表发生冲突时会返回错误。
    ///
    /// [`route`]: macro@boluo_macros::route
    pub fn try_mount<S>(self, route: impl Into<Route<S>>) -> Result<Self, RouterError>
    where
        S: Service<Request> + 'static,
        S::Response: IntoResponse,
        S::Error: Into<BoxError>,
    {
        self.try_mount_with(route, middleware_fn(|s| s))
    }

    /// 尝试将[`Route`](Route)对象注册到路由器，并对服务应用中间件，这通常和[`route`]宏配合使用。
    ///
    /// # 错误
    ///
    /// 给定了无效路径或路由表发生冲突时会返回错误。
    ///
    /// [`route`]: macro@boluo_macros::route
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

    /// 将另一个路由器的所有路由合并到此路由器中。
    ///
    /// # 恐慌
    ///
    /// 当路由表发生冲突时会出现恐慌。
    pub fn merge(self, other: impl Into<Router>) -> Self {
        self.try_merge(other).unwrap_or_else(|e| panic!("{e}"))
    }

    /// 将另一个路由器的所有路由合并到此路由器中，并对合并的服务应用中间件。
    ///
    /// # 恐慌
    ///
    /// 当路由表发生冲突时会出现恐慌。
    pub fn merge_with<M>(self, other: impl Into<Router>, middleware: M) -> Self
    where
        M: Middleware<ArcService<Request, Response, BoxError>> + Clone,
        M::Service: Service<Request> + 'static,
        <M::Service as Service<Request>>::Response: IntoResponse,
        <M::Service as Service<Request>>::Error: Into<BoxError>,
    {
        self.try_merge_with(other, middleware)
            .unwrap_or_else(|e| panic!("{e}"))
    }

    /// 尝试将另一个路由器的所有路由合并到此路由器中。
    ///
    /// # 错误
    ///
    /// 当路由表发生冲突时会返回错误。
    pub fn try_merge(self, other: impl Into<Router>) -> Result<Self, RouterError> {
        self.try_merge_with(other, middleware_fn(|s| s))
    }

    /// 尝试将另一个路由器的所有路由合并到此路由器中，并对合并的服务应用中间件。
    ///
    /// # 错误
    ///
    /// 当路由表发生冲突时会返回错误。
    pub fn try_merge_with<M>(
        mut self,
        other: impl Into<Router>,
        middleware: M,
    ) -> Result<Self, RouterError>
    where
        M: Middleware<ArcService<Request, Response, BoxError>> + Clone,
        M::Service: Service<Request> + 'static,
        <M::Service as Service<Request>>::Response: IntoResponse,
        <M::Service as Service<Request>>::Error: Into<BoxError>,
    {
        let other = other.into();
        for (id, endpoint) in other.table {
            self = self.add_endpoint_with(
                other.inner.get_path(id).unwrap(),
                endpoint,
                middleware.clone(),
            )?;
        }
        Ok(self)
    }

    /// 将另一个路由器的所有路由添加前缀后合并到此路由器中。
    ///
    /// # 恐慌
    ///
    /// 给定了无效路径或路由表发生冲突时会出现恐慌。
    pub fn scope_merge(self, path: &str, other: impl Into<Router>) -> Self {
        self.try_scope_merge(path, other)
            .unwrap_or_else(|e| panic!("{e}"))
    }

    /// 将另一个路由器的所有路由添加前缀后合并到此路由器中，并对合并的服务应用中间件。
    ///
    /// # 恐慌
    ///
    /// 给定了无效路径或路由表发生冲突时会出现恐慌。
    pub fn scope_merge_with<M>(self, path: &str, other: impl Into<Router>, middleware: M) -> Self
    where
        M: Middleware<ArcService<Request, Response, BoxError>> + Clone,
        M::Service: Service<Request> + 'static,
        <M::Service as Service<Request>>::Response: IntoResponse,
        <M::Service as Service<Request>>::Error: Into<BoxError>,
    {
        self.try_scope_merge_with(path, other, middleware)
            .unwrap_or_else(|e| panic!("{e}"))
    }

    /// 尝试将另一个路由器的所有路由添加前缀后合并到此路由器中。
    ///
    /// # 错误
    ///
    /// 给定了无效路径或路由表发生冲突时会返回错误。
    pub fn try_scope_merge(
        self,
        path: &str,
        other: impl Into<Router>,
    ) -> Result<Self, RouterError> {
        self.try_scope_merge_with(path, other, middleware_fn(|s| s))
    }

    /// 尝试将另一个路由器的所有路由添加前缀后合并到此路由器中，并对合并的服务应用中间件。
    ///
    /// # 错误
    ///
    /// 给定了无效路径或路由表发生冲突时会返回错误。
    pub fn try_scope_merge_with<M>(
        mut self,
        path: &str,
        other: impl Into<Router>,
        middleware: M,
    ) -> Result<Self, RouterError>
    where
        M: Middleware<ArcService<Request, Response, BoxError>> + Clone,
        M::Service: Service<Request> + 'static,
        <M::Service as Service<Request>>::Response: IntoResponse,
        <M::Service as Service<Request>>::Error: Into<BoxError>,
    {
        Self::validate_path(path)?;

        let other = other.into();
        for (id, endpoint) in other.table {
            self = self.add_endpoint_with(
                &combine_path_segments(path, other.inner.get_path(id).unwrap()),
                endpoint,
                middleware.clone(),
            )?;
        }
        Ok(self)
    }

    /// 从路由器中移除指定的路由。
    ///
    /// # 例子
    ///
    /// ```
    /// use boluo::handler::handler_fn;
    /// use boluo::http::Method;
    /// use boluo::route::{Router, any, get};
    ///
    /// let router = Router::new()
    ///     .route("/a", any(handler_fn(|| async { "a" })))
    ///     .route("/b", get(handler_fn(|| async { "b" })))
    ///     // 移除路径 "/a" 下接收任意方法的路由
    ///     .remove("/a", None)
    ///     // 移除路径 "/b" 下接收 GET 方法的路由
    ///     .remove("/b", &Method::GET);
    /// ```
    pub fn remove<'a>(mut self, path: &str, method: impl Into<Option<&'a Method>>) -> Self {
        let Some(id) = self.inner.get_id(path) else {
            return self;
        };
        let Some(method_router) = self.table.get_mut(&id).map(Endpoint::as_mut) else {
            return self;
        };

        method_router.remove(method);
        if method_router.is_empty() {
            self.table.remove(&id);
            self.inner.remove(path);
        }

        self
    }

    /// 返回一个迭代器，遍历路由器中的所有路由。
    ///
    /// # 例子
    ///
    /// ```
    /// use boluo::handler::handler_fn;
    /// use boluo::route::{Endpoint, Router};
    ///
    /// let router = Router::new()
    ///     .route("/a", handler_fn(|| async { "a" }))
    ///     .route("/b", handler_fn(|| async { "b" }));
    ///
    /// for (path, method, endpoint) in router.iter() {
    ///     match endpoint {
    ///         Endpoint::Route(_) => {
    ///             println!("Route - Path: {}, Method: {:?}", path, method);
    ///         }
    ///         Endpoint::Scope(_) => {
    ///             println!("Scope - Path: {}, Method: {:?}", path, method);
    ///         }
    ///     }
    /// }
    /// ```
    pub fn iter(&self) -> impl Iterator<Item = RouteEntry<'_>> {
        self.table.iter().flat_map(|(&id, endpoint)| {
            let path = self.inner.get_path(id).unwrap();
            endpoint.as_ref().iter().map(move |(method, service)| {
                let endpoint = match endpoint {
                    Endpoint::Route(_) => Endpoint::Route(service),
                    Endpoint::Scope(_) => Endpoint::Scope(service),
                };
                (path, method, endpoint)
            })
        })
    }

    fn add_endpoint<T: MergeToMethodRouter>(
        self,
        path: &str,
        endpoint: Endpoint<T>,
    ) -> Result<Self, RouterError> {
        self.add_endpoint_with(path, endpoint, middleware_fn(|s| s))
    }

    fn add_endpoint_with<T: MergeToMethodRouter, M>(
        mut self,
        path: &str,
        endpoint: Endpoint<T>,
        middleware: M,
    ) -> Result<Self, RouterError>
    where
        M: Middleware<ArcService<Request, Response, BoxError>> + Clone,
        M::Service: Service<Request> + 'static,
        <M::Service as Service<Request>>::Response: IntoResponse,
        <M::Service as Service<Request>>::Error: Into<BoxError>,
    {
        let id = self.inner.get_or_create_id(path)?;

        let result = match endpoint {
            Endpoint::Route(service) => {
                let Some(method_router) = self.get_or_create_route_endpoint(id) else {
                    return Err(RouterError::PathConflict {
                        path: path.to_owned(),
                        message: "conflict with previously registered path".to_owned(),
                    });
                };
                service.merge_to_with(method_router, middleware)
            }
            Endpoint::Scope(service) => {
                let Some(method_router) = self.get_or_create_scope_endpoint(id) else {
                    return Err(RouterError::PathConflict {
                        path: path.to_owned(),
                        message: "conflict with previously registered path".to_owned(),
                    });
                };
                service.merge_to_with(method_router, middleware)
            }
        };

        if let Err(method) = result {
            let message = match method {
                Some(method) => {
                    format!("conflict with previously registered `{method}` HTTP method")
                }
                None => "conflict with previously registered any HTTP method".to_owned(),
            };
            return Err(RouterError::PathConflict {
                path: path.to_owned(),
                message,
            });
        }

        Ok(self)
    }

    fn get_or_create_route_endpoint(&mut self, id: RouteId) -> Option<&mut MethodRouter> {
        let Endpoint::Route(router) = self
            .table
            .entry(id)
            .or_insert_with(|| Endpoint::Route(Default::default()))
        else {
            return None;
        };
        Some(router)
    }

    fn get_or_create_scope_endpoint(&mut self, id: RouteId) -> Option<&mut MethodRouter> {
        let Endpoint::Scope(router) = self
            .table
            .entry(id)
            .or_insert_with(|| Endpoint::Scope(Default::default()))
        else {
            return None;
        };
        Some(router)
    }

    fn validate_path(path: &str) -> Result<(), RouterError> {
        if !path.starts_with('/') {
            return Err(RouterError::InvalidPath {
                path: path.to_owned(),
                message: "path must start with a `/`".to_owned(),
            });
        }
        Ok(())
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
        let Ok(Match { value: id, params }) = self.inner.match_route(req.uri().path()) else {
            return Err(RouteError::not_found(req).into());
        };
        let Some(endpoint) = self.table.get(id) else {
            return Err(RouteError::not_found(req).into());
        };

        let (params, tail) = super::params::parse_path_params(params);
        super::params::insert_path_params(req.extensions_mut(), params);

        match endpoint {
            Endpoint::Route(service) => service.call(req).await,
            Endpoint::Scope(service) => {
                req = replace_request_path(req, tail.as_deref().unwrap_or_default());
                service.call(req).await
            }
        }
    }
}

/// 路由。
///
/// 用于向路由器注册服务的类型，描述访问服务的请求路径和方法。
#[derive(Debug, Clone)]
pub struct Route<S> {
    path: String,
    service: MethodRoute<S>,
}

impl<S> Route<S> {
    /// 创建路由，服务使用给定路径进行访问。
    ///
    #[doc = include_str!("../../doc/route/route.md")]
    pub fn new<T>(path: impl Into<String>, service: T) -> Self
    where
        T: IntoMethodRoute<Service = S>,
    {
        Self {
            path: path.into(),
            service: service.into_method_route(),
        }
    }

    /// 获取服务的访问路径。
    pub fn path(&self) -> &str {
        &self.path
    }

    /// 消耗路由，得到内部方法路由。
    pub fn into_method_route(self) -> MethodRoute<S> {
        self.service
    }

    /// 消耗路由，得到内部服务。
    pub fn into_service(self) -> S {
        self.service.into_service()
    }

    /// 对路由内部的服务应用中间件。
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

fn replace_request_path(req: Request, path: &str) -> Request {
    let (mut parts, body) = req.into_inner();
    parts.uri = replace_uri_path(parts.uri, path);
    Request::from_parts(parts, body)
}

fn replace_uri_path(uri: Uri, path: &str) -> Uri {
    let path = if let Some(path) = path.strip_prefix('/') {
        path
    } else {
        path
    };
    let path_and_query = if let Some(query) = uri.query() {
        format!("/{path}?{query}")
    } else {
        format!("/{path}")
    };
    let mut parts = uri.into_parts();
    parts.path_and_query = Some(path_and_query.parse().unwrap());
    Uri::from_parts(parts).unwrap()
}

fn combine_path_segments(prefix: &str, path: &str) -> String {
    let prefix = prefix.strip_suffix('/').unwrap_or(prefix);
    let path = path.strip_prefix('/').unwrap_or(path);
    format!("{prefix}/{path}")
}
