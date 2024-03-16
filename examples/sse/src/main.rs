use std::time::Duration;

use boluo::response::sse::{Event, KeepAlive, Sse};
use boluo::response::IntoResponse;
use boluo::route::Router;
use boluo::server::Server;
use futures_util::stream::repeat_with;
use tokio::net::TcpListener;
use tokio_stream::StreamExt;

#[tokio::main]
async fn main() {
    let listener = TcpListener::bind("127.0.0.1:3000").await.unwrap();

    let app = Router::new().mount(sse);

    Server::new(listener).run(app).await.unwrap();
}

#[boluo::route("/", method = "GET")]
async fn sse() -> impl IntoResponse {
    let stream = repeat_with(|| Event::builder().data("hi!").build())
        // 在事件之间添加3秒的延迟
        .throttle(Duration::from_secs(3));

    Sse::new(stream).keep_alive(KeepAlive::default())
}
