use std::path::Path;
use std::sync::Arc;

use boluo::BoxError;
use boluo::listener::{ConnectInfo, Listener};
use tokio::net::{TcpListener, TcpStream, ToSocketAddrs};
use tokio_rustls::TlsAcceptor;
use tokio_rustls::rustls::ServerConfig;
use tokio_rustls::rustls::pki_types::pem::PemObject;
use tokio_rustls::rustls::pki_types::{CertificateDer, PrivateKeyDer};
use tokio_rustls::server::TlsStream;

pub struct TlsListener {
    listener: TcpListener,
    acceptor: TlsAcceptor,
}

impl TlsListener {
    pub async fn bind<A>(addr: A, acceptor: TlsAcceptor) -> Result<Self, BoxError>
    where
        A: ToSocketAddrs,
    {
        Ok(Self {
            listener: TcpListener::bind(addr).await?,
            acceptor,
        })
    }
}

impl Listener for TlsListener {
    type IO = TlsStream<TcpStream>;
    type Addr = ConnectInfo;
    type Error = BoxError;

    async fn accept(&mut self) -> Result<(Self::IO, Self::Addr), Self::Error> {
        loop {
            let (stream, remote) = self.listener.accept().await?;

            let stream = match self.acceptor.accept(stream).await {
                Ok(stream) => stream,
                Err(error) => {
                    println!("error: {error}");
                    continue;
                }
            };

            let connect_info = ConnectInfo {
                local: self.listener.local_addr()?,
                remote,
            };

            return Ok((stream, connect_info));
        }
    }
}

pub fn new_acceptor<T, K>(cert: T, key: K) -> Result<TlsAcceptor, BoxError>
where
    T: AsRef<Path>,
    K: AsRef<Path>,
{
    let certs = CertificateDer::pem_file_iter(cert)?.collect::<Result<Vec<_>, _>>()?;
    let key = PrivateKeyDer::from_pem_file(key)?;
    Ok(TlsAcceptor::from(Arc::new(
        ServerConfig::builder()
            .with_no_client_auth()
            .with_single_cert(certs, key)?,
    )))
}
