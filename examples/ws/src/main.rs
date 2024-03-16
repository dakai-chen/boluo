use boluo::response::IntoResponse;
use boluo::route::Router;
use boluo::server::Server;
use boluo::ws::WebSocketUpgrade;
use tokio::net::TcpListener;

#[tokio::main]
async fn main() {
    let listener = TcpListener::bind("127.0.0.1:3000").await.unwrap();

    let app = Router::new().mount(echo);

    Server::new(listener).run(app).await.unwrap();
}

#[boluo::route("/", method = "GET")]
async fn echo(upgrade: WebSocketUpgrade) -> impl IntoResponse {
    upgrade.on_upgrade(|mut socket| async move {
        while let Some(Ok(message)) = socket.recv().await {
            if let Err(e) = socket.send(message).await {
                eprintln!("{e}");
            }
        }
    })
}
