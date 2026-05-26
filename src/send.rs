use tokio::net::TcpListener;

use tokio::io::{self, AsyncReadExt, AsyncWriteExt};
use tokio::net::TcpStream;

pub async fn send_single(dst: &str, data: &[u8]) -> io::Result<()> {
    let mut stream = TcpStream::connect(&dst).await?;

    stream.write_all(data).await?;

    Ok(())
}

pub async fn recieve_single(port: &str) -> io::Result<Vec<u8>> {
    let listener = TcpListener::bind(&port).await?;

    let (mut stream, _peer_adrr) = listener.accept().await?;

    let mut data = Vec::new();
    stream.read_to_end(&mut data).await?;

    Ok(data)
}


