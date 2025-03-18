//! 将请求转发到服务的类型和特征。

mod error;
mod method;
mod params;
mod router;

pub use error::{RouteError, RouteErrorKind, RouterError};
pub use method::{IntoMethodRoute, MethodRoute};
pub use method::{any, connect, delete, get, head, options, patch, post, put, trace};
pub use params::PathParams;
pub use router::{Route, Router};
