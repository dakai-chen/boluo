//! 将请求转发到服务的类型和特征。

mod error;
pub use error::{RouteError, RouteErrorKind, RouterError};

mod method;
pub use method::{any, connect, delete, get, head, options, patch, post, put, trace};
pub use method::{IntoMethodRoute, MethodRoute};

mod router;
pub use router::{Route, Router};

mod params;
pub use params::PathParams;
