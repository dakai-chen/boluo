use std::fmt::Write;
use std::path::PathBuf;

use boluo::BoxError;
use boluo::handler::handler_fn;
use boluo::response::{Html, IntoResponse};
use boluo::route::Router;
use boluo::server::Server;
use boluo::static_file::{ServeDir, ServeFile};
use tokio::net::TcpListener;

#[tokio::main]
async fn main() {
    let listener = TcpListener::bind("127.0.0.1:3000").await.unwrap();

    let app = Router::new()
        // 显示首页。
        .route("/", ServeFile::new(assets_dir().join("index.html")))
        // 列出`assets`目录中的文件。
        .route("/assets/", handler_fn(list))
        // 提供`assets`目录中的文件。
        .scope("/assets/{*}", ServeDir::new(assets_dir()));

    Server::new(listener).run(app).await.unwrap();
}

async fn list() -> Result<impl IntoResponse, BoxError> {
    let mut list = String::new();

    for path in std::fs::read_dir(assets_dir())?
        .filter_map(|v| v.ok())
        .filter(|v| v.path().is_file())
        .map(|v| v.file_name())
    {
        if let Some(path) = path.to_str() {
            write!(&mut list, "<li><a href='{path}'>{path}</a></li>")?;
        }
    }

    Ok(Html(format!("<html><body><ul>{list}</ul></body></html>")))
}

fn assets_dir() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("assets")
}
