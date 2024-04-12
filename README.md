<h1 align="center">
    boluo
</h1>

<p align="center">
    简单易用的异步网络框架
</p>

## 可选功能

| 功能名    | 描述                                  | 默认启用 |
| --------- | ------------------------------------- | -------- |
| server    | 启用服务器和监听器                    | 是       |
| http1     | 添加服务器对HTTP1的支持               | 是       |
| http2     | 添加服务器对HTTP2的支持               |          |
| listener  | 启用监听器                            |          |
| multipart | 添加对`multipart/form-data`格式的支持 |          |
| sse       | 添加对服务器发送事件的支持            |          |
| ws        | 添加对网络套接字的支持                |          |
| fs        | 添加对静态文件的支持                  |          |

## 快速开始

新建项目：

```bash
cargo new demo && cd demo
```

添加依赖：

```toml
[dependencies]
boluo = "0.5"
tokio = { version = "1", features = ["full"] }
```

用以下内容覆盖`src/main.rs`：

```rust
use boluo::response::IntoResponse;
use boluo::route::Router;
use boluo::server::Server;
use tokio::net::TcpListener;

#[tokio::main]
async fn main() {
    let listener = TcpListener::bind("127.0.0.1:3000").await.unwrap();

    let app = Router::new().mount(hello);

    Server::new(listener).run(app).await.unwrap();
}

#[boluo::route("/", method = "GET")]
async fn hello() -> impl IntoResponse {
    "Hello, World!"
}
```

运行项目：

```bash
cargo run
```

访问服务：

```bash
curl http://127.0.0.1:3000/
```

## 更多示例

[在这里](./examples/)可以找到更多的示例代码。在示例目录中，你可以通过以下命令运行示例：

```bash
cargo run --bin hello
```

## 支持的最低Rust版本（MSRV）

支持的最低Rust版本为`1.75.0`。
