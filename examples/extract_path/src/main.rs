use boluo::extract::Path;
use boluo::response::IntoResponse;
use boluo::route::Router;
use boluo::server::Server;
use tokio::net::TcpListener;

#[tokio::main]
async fn main() {
    let listener = TcpListener::bind("127.0.0.1:3000").await.unwrap();

    let app = Router::new().mount(basic_arithmetic);

    Server::new(listener).run(app).await.unwrap();
}

#[boluo::route("/{f}/{a}/{b}", method = "GET")]
async fn basic_arithmetic(Path((f, a, b)): Path<(BasicArithmetic, i32, i32)>) -> impl IntoResponse {
    match f {
        BasicArithmetic::Add => format!("{a} + {b} = {}", a + b),
        BasicArithmetic::Sub => format!("{a} - {b} = {}", a - b),
        BasicArithmetic::Mul => format!("{a} * {b} = {}", a * b),
        BasicArithmetic::Div => format!("{a} / {b} = {}", a / b),
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
