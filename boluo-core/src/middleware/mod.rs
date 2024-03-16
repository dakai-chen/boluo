mod middleware_fn;

pub use middleware_fn::{middleware_fn, MiddlewareFn};

pub trait Middleware<S> {
    type Service;

    fn transform(self, service: S) -> Self::Service;
}
