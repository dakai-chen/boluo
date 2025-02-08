use std::fmt::Write;

use boluo::handler::handler_fn;
use boluo::response::{Html, IntoResponse};
use boluo::route::Router;
use boluo::server::Server;
use boluo::static_file::{ServeDir, ServeFile};
use boluo::BoxError;
use tokio::net::TcpListener;

const ASSETS_DIR: &str = "./static_file/assets";

#[tokio::main]
async fn main() {
    let listener = TcpListener::bind("127.0.0.1:3000").await.unwrap();

    let app = Router::new()
        // 显示首页。
        .route("/", ServeFile::new(format!("{ASSETS_DIR}/index.html")))
        // 列出`assets`目录中的文件。
        .route("/assets/", handler_fn(list))
        // 提供`assets`目录中的文件。
        .scope("/assets/{*}", ServeDir::new(ASSETS_DIR));

    Server::new(listener).run(app).await.unwrap();
}

async fn list() -> Result<impl IntoResponse, BoxError> {
    let mut list = String::new();

    for path in std::fs::read_dir(ASSETS_DIR)?
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
