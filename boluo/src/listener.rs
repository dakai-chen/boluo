//! 监听器的特征和相关类型的定义。

use std::net::SocketAddr;

/// 连接信息。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct ConnectInfo {
    /// 连接的本地地址。
    pub local: SocketAddr,
    /// 连接的远程地址。
    pub remote: SocketAddr,
}

/// 表示可以提供连接的类型，用于实现监听器。
pub trait Listener {
    /// 监听器返回的连接。
    type IO;

    /// 监听器返回的连接地址。
    type Addr;

    /// 监听器产生的错误。
    type Error;

    /// 接收此监听器新传入的连接。
    fn accept(
        &mut self,
    ) -> impl Future<Output = Result<(Self::IO, Self::Addr), Self::Error>> + Send;
}

#[cfg(feature = "tokio")]
impl Listener for tokio::net::TcpListener {
    type IO = tokio::net::TcpStream;
    type Addr = ConnectInfo;
    type Error = std::io::Error;

    async fn accept(&mut self) -> std::io::Result<(Self::IO, Self::Addr)> {
        tokio::net::TcpListener::accept(self)
            .await
            .and_then(|(conn, remote)| {
                self.local_addr()
                    .map(|local| (conn, ConnectInfo { local, remote }))
            })
    }
}
