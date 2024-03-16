use std::sync::{Arc, Mutex};

use boluo::data::Extension;
use boluo::response::IntoResponse;
use boluo::route::Router;
use boluo::server::Server;
use boluo::service::ServiceExt;
use tokio::net::TcpListener;

#[tokio::main]
async fn main() {
    let state = Count::new(); // 统计访问次数

    let listener = TcpListener::bind("127.0.0.1:3000").await.unwrap();

    let app = Router::new()
        .mount(hello)
        .mount(count)
        .with(Extension(state)); // 添加扩展

    Server::new(listener).run(app).await.unwrap();
}

#[boluo::route("/", method = "GET")]
async fn hello(Extension(state): Extension<Count>) -> impl IntoResponse {
    state.inc(); // 每次访问将计数器加一
}

#[boluo::route("/count", method = "GET")]
async fn count(Extension(state): Extension<Count>) -> impl IntoResponse {
    format!("{}", state.as_u64()) // 返回访问次数
}

#[derive(Debug, Clone)]
struct Count(Arc<Mutex<u64>>);

impl Count {
    fn new() -> Self {
        Self(Arc::new(Mutex::new(0)))
    }

    fn inc(&self) {
        *self.0.lock().unwrap() += 1;
    }

    fn as_u64(&self) -> u64 {
        *self.0.lock().unwrap()
    }
}
