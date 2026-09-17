use crate::primitives::{NetworkTransmissionProtocol, PortState};
use std::{collections::HashMap, io};

pub(crate) async fn port_scan(addr: String, port: usize) -> io::Result<()> {
    for protocol in NetworkTransmissionProtocol::protocols() {
        let result = open_connection(addr.as_str(), port, protocol.clone()).await;
    }

    Ok(())
}

pub(crate) type PortScanResult = HashMap<NetworkTransmissionProtocol, PortState>;

enum Connection {
    Udp(tokio::net::UdpSocket),
    Tcp(tokio::net::TcpStream),
}

async fn open_connection(
    addr: &str,
    port: usize,
    protocol: NetworkTransmissionProtocol,
) -> io::Result<Connection> {
    let addr = format!("{addr}:{port}");

    let connection = match protocol {
        NetworkTransmissionProtocol::Tcp => {
            let socket = tokio::net::TcpStream::connect(addr).await?;

            Connection::Tcp(socket)
        }
        NetworkTransmissionProtocol::Udp => {
            let socket = tokio::net::UdpSocket::bind("0.0.0.0:0").await?;
            socket.connect(addr).await?;

            Connection::Udp(socket)
        }
    };

    Ok(connection)
}
