// TODO 对该框架的基准测试还没有完成，以下是一个简单的例子。

use boluo::BoxError;
use boluo::body::Body;
use boluo::handler::handler_fn;
use boluo::http::Method;
use boluo::request::Request;
use boluo::response::IntoResponse;
use boluo::route::{Router, get, patch, post, put};
use boluo::service::Service;
use criterion::{Criterion, black_box, criterion_group, criterion_main};
use tokio::runtime::Builder;

fn router() -> impl Service<Request, Response = impl IntoResponse, Error = impl Into<BoxError>> {
    Router::new()
        .route("/users", post(handler_fn(|| async {})))
        .route("/posts", post(handler_fn(|| async {})))
        .route("/users/{id}", patch(handler_fn(|| async {})))
        .route("/posts/{id}", patch(handler_fn(|| async {})))
        .route("/users/{id}/nickname", put(handler_fn(|| async {})))
        .route("/posts/{id}/state", put(handler_fn(|| async {})))
        .route("/users/{id}/nickname", get(handler_fn(|| async {})))
        .route("/posts/{id}/state", get(handler_fn(|| async {})))
        .route("/users/{id}", get(handler_fn(|| async {})))
        .route("/posts/{id}", get(handler_fn(|| async {})))
}

fn router_benchmark(c: &mut Criterion) {
    let runtime = Builder::new_multi_thread()
        .worker_threads(4)
        .build()
        .unwrap();

    let router_a = black_box(router());

    c.bench_function("router", |b| {
        b.to_async(&runtime).iter(|| async {
            for request in black_box(requests()) {
                let _ = router_a.call(request).await;
            }
        })
    });
}

criterion_group!(benches, router_benchmark);
criterion_main!(benches);

fn requests() -> impl IntoIterator<Item = Request> {
    [
        request!(Method::POST, "/users"),
        request!(Method::POST, "/posts"),
        request!(Method::PATCH, "/users/643734"),
        request!(Method::PATCH, "/posts/124133"),
        request!(Method::PUT, "/users/643734/nickname"),
        request!(Method::PUT, "/posts/124133/state"),
        request!(Method::GET, "/users/210875/username"),
        request!(Method::GET, "/users/490584/nickname"),
        request!(Method::GET, "/posts/124543"),
        request!(Method::GET, "/posts/637577"),
        request!(Method::DELETE, "/posts/124812"),
        request!(Method::GET, "/posts/450824/content"),
    ]
}

macro_rules! request {
    ($method:expr, $uri:expr) => {
        Request::builder()
            .method($method)
            .uri($uri)
            .body(Body::empty())
            .unwrap()
    };
}

use request;
