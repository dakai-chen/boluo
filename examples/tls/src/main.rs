mod tls_listener;
use tls_listener::{TlsListener, new_acceptor};

use std::path::PathBuf;

use boluo::response::IntoResponse;
use boluo::route::Router;
use boluo::server::Server;

#[tokio::main]
async fn main() {
    let acceptor = new_acceptor(
        PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("cert/cert.pem"),
        PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("cert/key.pem"),
    )
    .unwrap();
    let listener = TlsListener::bind("127.0.0.1:3000", acceptor).await.unwrap();

    let app = Router::new().mount(hello);

    Server::new(listener).run(app).await.unwrap();
}

#[boluo::route("/", method = "GET")]
async fn hello() -> impl IntoResponse {
    "Hello, World!"
}
