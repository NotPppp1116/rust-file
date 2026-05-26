use tokio::io;
use tokio::net::UdpSocket;
use tokio::time::{Duration, timeout};

const DISCOVERY_ADDR: &str = "0.0.0.0:9001";
const DISCOVERY_BROADCAST: &str = "255.255.255.255:9001";
const DISCOVERY_TIMEOUT: Duration = Duration::from_secs(3);

pub async fn discovery_serve(file_name: &str, receiver_addr: &str) -> io::Result<()> {
    let socket = UdpSocket::bind(DISCOVERY_ADDR).await?;
    let mut buf = [0u8; 1024];
    let response = format!("{file_name}|{receiver_addr}");

    loop {
        let (n, peer) = socket.recv_from(&mut buf).await?;

        if &buf[..n] == format!("{}?", file_name).as_bytes() {
            socket.send_to(response.as_bytes(), peer).await?;
        }
    }
}

pub async fn find_receiver(file_name: &str) -> io::Result<String> {
    let socket = UdpSocket::bind("0.0.0.0:0").await?;
    socket.set_broadcast(true)?;

    let query = format!("{file_name}?");
    socket
        .send_to(query.as_bytes(), DISCOVERY_BROADCAST)
        .await?;

    let mut buf = [0u8; 1024];
    let (n, peer) = timeout(DISCOVERY_TIMEOUT, socket.recv_from(&mut buf))
        .await
        .map_err(|_| io::Error::new(io::ErrorKind::TimedOut, "receiver discovery timed out"))??;

    let response = String::from_utf8(buf[..n].to_vec()).map_err(|_| {
        io::Error::new(
            io::ErrorKind::InvalidData,
            format!("receiver {peer} sent an invalid discovery response"),
        )
    })?;

    let (response_file_name, receiver_addr) = response.split_once('|').ok_or_else(|| {
        io::Error::new(
            io::ErrorKind::InvalidData,
            format!("receiver {peer} sent an invalid discovery response"),
        )
    })?;

    if response_file_name != file_name {
        return Err(io::Error::new(
            io::ErrorKind::InvalidData,
            format!("receiver {peer} answered for a different file"),
        ));
    }

    Ok(receiver_addr.to_string())
}
