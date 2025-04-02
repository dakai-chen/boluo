//! `boluo` 的宏。

#![forbid(unsafe_code)]
#![warn(
    missing_debug_implementations,
    missing_docs,
    rust_2018_idioms,
    unreachable_pub
)]
#![cfg_attr(docsrs, feature(doc_auto_cfg, doc_cfg))]

mod route;

use proc_macro::TokenStream;

/// 为处理程序添加请求路径和方法。
///
/// # 例子
///
/// ```ignore
/// #[boluo::route("/", method = "GET")]
/// async fn hello() -> &'static str {
///     "Hello, World!"
/// }
/// ```
#[proc_macro_attribute]
pub fn route(attr: TokenStream, item: TokenStream) -> TokenStream {
    route::route(attr, item)
}
