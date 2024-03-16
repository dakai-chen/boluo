use std::future::Future;
use std::io;
use std::net::SocketAddr;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct ConnectionInfo {
    pub local: SocketAddr,
    pub remote: SocketAddr,
}

pub trait Listener {
    type IO;
    type Error;

    fn accept(
        &mut self,
    ) -> impl Future<Output = Result<(Self::IO, Option<ConnectionInfo>), Self::Error>> + Send;
}

impl Listener for tokio::net::TcpListener {
    type IO = tokio::net::TcpStream;
    type Error = io::Error;

    async fn accept(&mut self) -> io::Result<(Self::IO, Option<ConnectionInfo>)> {
        tokio::net::TcpListener::accept(self)
            .await
            .and_then(|(conn, remote)| {
                self.local_addr()
                    .map(|local| (conn, Some(ConnectionInfo { local, remote })))
            })
    }
}
