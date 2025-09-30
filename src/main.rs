use std::{
    fs::File,
    io::BufReader,
    net::SocketAddr,
    sync::Arc,
};

use anyhow::Result;
use futures_util::{SinkExt, StreamExt};
use tokio::{
    net::TcpListener,
    sync::broadcast,
};
use tokio::io::{AsyncBufReadExt, AsyncWriteExt}; // <-- needed for lines() and write_all()
use tokio_rustls::{
    rustls::{
        self,
        pki_types::{CertificateDer, PrivateKeyDer, PrivatePkcs8KeyDer},
        ServerConfig,
    },
    TlsAcceptor,
};
use tokio_stream::wrappers::BroadcastStream;

/// Load certificates from a PEM file
fn load_certs(path: &str) -> Result<Vec<CertificateDer<'static>>> {
    let mut reader = BufReader::new(File::open(path)?);
    let certs = rustls_pemfile::certs(&mut reader)
        .collect::<Result<Vec<_>, _>>()?;
    Ok(certs)
}

/// Load a private key from a PEM file
fn load_key(path: &str) -> Result<PrivateKeyDer<'static>> {
    let mut reader = BufReader::new(File::open(path)?);
    let key = rustls_pemfile::pkcs8_private_keys(&mut reader)
        .filter_map(|r| r.ok())
        .next()
        .ok_or_else(|| anyhow::anyhow!("no private keys found"))?;
    Ok(PrivateKeyDer::Pkcs8(key))
}

#[tokio::main]
async fn main() -> Result<()> {
    // === TLS SETUP ===
    let certs = load_certs("cert.pem")?;
    let key = load_key("key.pem")?;

    let config = ServerConfig::builder()
        .with_no_client_auth()
        .with_single_cert(certs, key)?;

    let acceptor = TlsAcceptor::from(Arc::new(config));

    // === BROADCAST CHANNEL ===
    let (tx, _rx) = broadcast::channel::<String>(100);

    // === TCP LISTENER ===
    let addr: SocketAddr = "127.0.0.1:8080".parse()?;
    let listener = TcpListener::bind(addr).await?;
    println!("TLS Chat Server running on {}", addr);

    loop {
        let (stream, peer) = listener.accept().await?;
        let acceptor = acceptor.clone();
        let tx = tx.clone();
        let mut rx = tx.subscribe();

        tokio::spawn(async move {
            let tls_stream = match acceptor.accept(stream).await {
                Ok(s) => s,
                Err(e) => {
                    eprintln!("TLS handshake failed with {}: {:?}", peer, e);
                    return;
                }
            };

            println!("Client connected: {}", peer);

            let (reader, mut writer) = tokio::io::split(tls_stream);
            let mut lines = tokio::io::BufReader::new(reader).lines();

            // forward incoming broadcast messages to client
            let mut rx_stream = BroadcastStream::new(rx);

            loop {
                tokio::select! {
                    // receive from this client
                    maybe_line = lines.next_line() => {
                        match maybe_line {
                            Ok(Some(line)) => {
                                println!("{}: {}", peer, line);
                                let _ = tx.send(format!("{}: {}", peer, line));
                            }
                            Ok(None) => {
                                println!("{} disconnected", peer);
                                break;
                            }
                            Err(e) => {
                                eprintln!("error reading from {}: {:?}", peer, e);
                                break;
                            }
                        }
                    }

                    // receive from broadcast
                    Some(Ok(msg)) = rx_stream.next() => {
                        if let Err(e) = writer.write_all(msg.as_bytes()).await {
                            eprintln!("error writing to {}: {:?}", peer, e);
                            break;
                        }
                        let _ = writer.write_all(b"\n").await;
                    }
                }
            }
        });
    }
}