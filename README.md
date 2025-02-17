<h1 align="center">
    boluo
</h1>

<p align="center">
    简单易用的高性能异步网络框架
</p>

## 介绍

`boluo` 是一个基于 `tokio` 和 `hyper` 开发的轻量级路由层，几乎没有额外的性能开销，拥有极快的运行速度。

## 特点

- 简单清晰的路由定义，支持嵌套路由、宏定义路由和路由合并。
- 提供核心 Trait `Service` 和 `Middleware`，灵活且易于扩展。
- 提供统一的错误处理机制，简化了错误处理逻辑。

## 可选功能

| 功能名      | 描述                                  | 默认启用 |
| ----------- | ------------------------------------- | -------- |
| http1       | 启用HTTP1服务器                       | 是       |
| http2       | 启用HTTP2服务器                       |          |
| multipart   | 添加对`multipart/form-data`格式的支持 |          |
| sse         | 添加对服务器发送事件的支持            |          |
| ws          | 添加对网络套接字的支持                |          |
| static-file | 添加对静态文件的支持                  |          |

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

支持的最低Rust版本为`1.85.0`。
