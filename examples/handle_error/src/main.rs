use boluo::extract::Path;
use boluo::http::StatusCode;
use boluo::response::{IntoResponse, Response};
use boluo::route::Router;
use boluo::server::Server;
use boluo::service::ServiceExt;
use boluo::BoxError;
use tokio::net::TcpListener;

#[tokio::main]
async fn main() {
    let listener = TcpListener::bind("127.0.0.1:3000").await.unwrap();

    let app = Router::new()
        .mount(basic_arithmetic)
        // 添加错误处理程序
        .or_else(handle_error);

    Server::new(listener).run(app).await.unwrap();
}

#[boluo::route("/{f}/{a}/{b}", method = "GET")]
async fn basic_arithmetic(
    Path((f, a, b)): Path<(BasicArithmetic, i32, i32)>,
) -> Result<impl IntoResponse, DivisionByZeroError> {
    if let BasicArithmetic::Div = f {
        if b == 0 {
            // 返回除零错误
            return Err(DivisionByZeroError);
        }
    }

    match f {
        BasicArithmetic::Add => Ok(format!("{a} + {b} = {}", a + b)),
        BasicArithmetic::Sub => Ok(format!("{a} - {b} = {}", a - b)),
        BasicArithmetic::Mul => Ok(format!("{a} * {b} = {}", a * b)),
        BasicArithmetic::Div => Ok(format!("{a} / {b} = {}", a / b)),
    }
}

#[derive(serde::Deserialize)]
#[serde(rename_all = "snake_case")]
enum BasicArithmetic {
    Add,
    Sub,
    Mul,
    Div,
}

async fn handle_error(error: BoxError) -> Result<Response, BoxError> {
    // 捕获除零错误，将错误转换为用户友好的响应消息
    if let Some(e) = error.downcast_ref::<DivisionByZeroError>() {
        return (StatusCode::BAD_REQUEST, e.to_string()).into_response();
    }
    Err(error)
}

/// 自定义除零错误
#[derive(Debug, Clone, Copy)]
struct DivisionByZeroError;

impl std::fmt::Display for DivisionByZeroError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "division by zero")
    }
}

impl std::error::Error for DivisionByZeroError {}
