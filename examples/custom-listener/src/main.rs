mod listener;
use listener::FileListener;

use boluo::extract::Path;
use boluo::response::IntoResponse;
use boluo::route::Router;
use boluo::server::Server;

#[tokio::main]
async fn main() {
    let listener = FileListener::new("custom-listener/http", "custom-listener/http_out")
        .await
        .unwrap();

    let app = Router::new().mount(basic);

    Server::new(listener).run(app).await.unwrap();
}

#[boluo::route("/{op}/{a}/{b}", method = "POST")]
async fn basic(Path((op, a, b)): Path<(String, i64, i64)>) -> impl IntoResponse {
    match op.as_str() {
        "add" => format!("{a} + {b} = {}", a + b),
        "sub" => format!("{a} - {b} = {}", a - b),
        "mul" => format!("{a} * {b} = {}", a * b),
        "div" => format!("{a} / {b} = {}", a / b),
        _ => format!("Does not support `{op}`"),
    }
}
