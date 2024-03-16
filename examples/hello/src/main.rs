use boluo::handler::handler_fn;
use boluo::server::Server;
use tokio::net::TcpListener;

#[tokio::main]
async fn main() {
    let listener = TcpListener::bind("127.0.0.1:3000").await.unwrap();

    let f = handler_fn(|| async { "Hello, World!" });

    Server::new(listener).run(f).await.unwrap();
}
