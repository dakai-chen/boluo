//! `boluo` 是一个简单易用的异步网络框架。
//!
//! # 目录
//!
//! - [快速开始](#快速开始)
//! - [服务](#服务)
//! - [处理程序](#处理程序)
//! - [提取器](#提取器)
//! - [响应](#响应)
//! - [路由](#路由)
//! - [错误处理](#错误处理)
//! - [中间件](#中间件)
//!
//! # 快速开始
//!
//! 新建项目：
//!
//! ```bash
//! cargo new demo && cd demo
//! ```
//!
//! 添加依赖：
//!
//! ```toml
//! [dependencies]
//! boluo = "0.7"
//! tokio = { version = "1", features = ["full"] }
//! ```
//!
//! 用以下内容覆盖 `src/main.rs`：
//!
//! ```no_run
//! use boluo::response::IntoResponse;
//! use boluo::route::Router;
//! use boluo::server::Server;
//! use tokio::net::TcpListener;
//!
//! #[tokio::main]
//! async fn main() {
//!     let listener = TcpListener::bind("127.0.0.1:3000").await.unwrap();
//!
//!     let app = Router::new().mount(hello);
//!
//!     Server::new(listener).run(app).await.unwrap();
//! }
//!
//! #[boluo::route("/", method = "GET")]
//! async fn hello() -> impl IntoResponse {
//!     "Hello, World!"
//! }
//! ```
//!
//! 运行项目：
//!
//! ```bash
//! cargo run
//! ```
//!
//! 访问服务：
//!
//! ```bash
//! curl http://127.0.0.1:3000/
//! ```
//!
//! # 服务
//!
//! [`Service`] 特征表示一个接收请求并返回响应的异步函数。
//!
//! # 处理程序
//!
//! 处理程序是一个异步函数，它接受零个或多个提取器作为参数，并返回可以转换为响应的内容。
//!
//! 处理程序如下所示：
//!
//! ```
//! use boluo::body::Body;
//! use boluo::handler::handler_fn;
//! use boluo::response::IntoResponse;
//!
//! // 返回空的 `200 OK` 响应的处理程序。
//! async fn empty() {}
//!
//! // 返回带有纯文本主体的 `200 OK` 响应的处理程序。
//! async fn hello() -> &'static str {
//!     "Hello, World!"
//! }
//!
//! // 返回带有请求主体的 `200 OK` 响应的处理程序。
//! //
//! // `Body` 实现了 `FromRequest` 特征，可以作为提取器解析请求。并且也实现了
//! // `IntoResponse` 特征，可以作为响应类型。
//! async fn echo(body: Body) -> impl IntoResponse {
//!     body
//! }
//!
//! // 使用 `handler_fn` 函数将处理程序转换为 `Service`。
//! let service = handler_fn(echo);
//! ```
//!
//! # 提取器
//!
//! 提取器是实现了 [`FromRequest`] 特征的类型，可以根据 [`Request`] 创建实例。
//!
//! ```
//! use std::convert::Infallible;
//!
//! use boluo::extract::FromRequest;
//! use boluo::http::{header, HeaderValue};
//! use boluo::request::Request;
//!
//! // 从请求头中提取 HOST 的提取器。
//! struct Host(Option<HeaderValue>);
//!
//! // 为提取器实现 `FromRequest` 特征。
//! impl FromRequest for Host {
//!     type Error = Infallible;
//!
//!     async fn from_request(request: &mut Request) -> Result<Self, Self::Error> {
//!         let value = request.headers().get(header::HOST).map(|v| v.to_owned());
//!         Ok(Host(value))
//!     }
//! }
//!
//! // 在处理程序中使用提取器从请求中提取数据。
//! async fn using_extractor(Host(host): Host) {
//!     println!("{host:?}")
//! }
//! ```
//!
//! # 响应
//!
//! 任何实现 [`IntoResponse`] 特征的类型都可以作为响应。
//!
//! ```
//! use boluo::response::Html;
//! use boluo::response::IntoResponse;
//!
//! // 返回带有纯文本主体的 `200 OK` 响应的处理程序。
//! async fn hello() -> &'static str {
//!     "Hello, World!"
//! }
//!
//! // 返回显示 `Hello, World!` 的 HTML 页面的处理程序。
//! async fn html() -> impl IntoResponse {
//!     Html("<html><body>Hello, World!</body></html>")
//! }
//! ```
//!
//! # 路由
//!
//! [`Router`] 用于设置哪些路径通向哪些服务。
//!
//! ```
//! use boluo::handler::handler_fn;
//! use boluo::route::Router;
//!
//! #[boluo::route("/f", method = "GET")]
//! async fn f() -> &'static str {
//!     "f"
//! }
//!
//! let ab = Router::new()
//!     .route("/a", handler_fn(|| async { "a" }))
//!     .route("/b", handler_fn(|| async { "b" }));
//!
//! let cd = Router::new()
//!     .route("/c", handler_fn(|| async { "c" }))
//!     .route("/d", handler_fn(|| async { "d" }));
//!
//! Router::new()
//!     // 路由。
//!     .route("/a", handler_fn(|| async { "a" }))
//!     .route("/b", handler_fn(|| async { "b" }))
//!     // 嵌套路由。
//!     .scope("/x", ab)
//!     // 将其他路由器的路由合并到当前路由器。
//!     .merge(cd)
//!     // 挂载宏定义路由。
//!     .mount(f);
//! ```
//!
//! # 错误处理
//!
//! 错误和响应是分离的，可以在中间件对特定错误进行捕获，并将错误转换为自己想要的响应格式。
//!
//! 未经处理的错误到达服务器时，服务器将返回带有错误信息的 `500 INTERNAL_SERVER_ERROR` 响应。
//!
//! ```
//! use boluo::http::StatusCode;
//! use boluo::response::{IntoResponse, Response};
//! use boluo::route::{RouteError, RouteErrorKind, Router};
//! use boluo::service::ServiceExt;
//! use boluo::BoxError;
//!
//! #[derive(Debug)]
//! struct MyError;
//!
//! impl std::fmt::Display for MyError {
//!     fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
//!         write!(f, "some error message")
//!     }
//! }
//!
//! impl std::error::Error for MyError {}
//!
//! // 处理程序。
//! #[boluo::route("/", method = "GET")]
//! async fn throw_error() -> Result<(), MyError> {
//!     Err(MyError)
//! }
//!
//! // 错误处理。
//! async fn handle_error(err: BoxError) -> Result<Response, BoxError> {
//!     // 处理框架抛出的路由错误，并自定义响应方式。
//!     if let Some(e) = err.downcast_ref::<RouteError>() {
//!         let status_code = match e.kind() {
//!             RouteErrorKind::NotFound => StatusCode::NOT_FOUND,
//!             RouteErrorKind::MethodNotAllowed => StatusCode::METHOD_NOT_ALLOWED,
//!         };
//!         return Ok((status_code, format!("{status_code}")).into_response()?);
//!     }
//!     if let Some(_) = err.downcast_ref::<MyError>() {
//!         // 记录错误、转为响应等等。
//!     }
//!     Err(err)
//! }
//!
//! Router::new().mount(throw_error).or_else(handle_error);
//! ```
//!
//! # 中间件
//!
//! 中间件是实现了 [`Middleware`] 特征的类型，可以调用 [`ServiceExt::with`] 函数将中间件
//! 应用于 [`Service`]。
//!
//! 中间件实际上是将原始服务转换为新的服务。
//!
//! ```
//! use boluo::data::Extension;
//! use boluo::handler::handler_fn;
//! use boluo::service::ServiceExt;
//!
//! async fn extension(Extension(text): Extension<&'static str>) -> &'static str {
//!     text
//! }
//!
//! let service = handler_fn(extension);
//! let service = service.with(Extension("Hello, World!"));
//! ```
//!
//! [`Service`]: crate::service::Service
//! [`FromRequest`]: crate::extract::FromRequest
//! [`Request`]: crate::request::Request
//! [`IntoResponse`]: crate::response::IntoResponse
//! [`Router`]: crate::route::Router
//! [`Middleware`]: crate::middleware::Middleware
//! [`ServiceExt::with`]: crate::service::ServiceExt::with

#![forbid(unsafe_code)]
#![warn(
    missing_debug_implementations,
    missing_docs,
    rust_2018_idioms,
    unreachable_pub
)]
#![cfg_attr(docsrs, feature(doc_auto_cfg, doc_cfg))]

pub use boluo_core::BoxError;
pub use boluo_core::{body, handler, http, request, service, upgrade};

pub use boluo_macros::route;

pub mod data;
pub mod extract;
pub mod listener;
pub mod middleware;
pub mod response;
pub mod route;

pub use headers;

#[cfg(any(feature = "http1", feature = "http2"))]
pub mod server;

#[cfg(feature = "multipart")]
pub mod multipart;

#[cfg(feature = "ws")]
pub mod ws;

#[cfg(feature = "static-file")]
pub mod static_file;
