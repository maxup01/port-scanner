use crate::primitives::{NetworkTransmissionProtocol, PortState};
use core::{net::SocketAddr, time::Duration};
use std::{
    collections::HashMap,
    io::{self, ErrorKind},
};

pub(crate) async fn port_scan(addr: String, port: usize) -> PortScanResult {
    let scan_result: PortScanResult = HashMap::new();

    for protocol in NetworkTransmissionProtocol::protocols() {
        let result = open_connection(addr.as_str(), port, protocol.clone()).await;
    }

    scan_result
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

async fn check_tcp(addr: SocketAddr, wait: Duration) -> PortState {
    match tokio::time::timeout(wait, tokio::net::TcpStream::connect(addr)).await {
        Ok(Ok(_stream)) => PortState::Open,
        Ok(Err(e)) => match e.kind() {
            ErrorKind::ConnectionRefused => PortState::Closed,
            ErrorKind::HostUnreachable => PortState::Unreachable,
            _ => PortState::Filtered,
        },
        Err(_elapsed) => PortState::Filtered,
    }
}

async fn check_udp_state(addr: SocketAddr, wait: Duration) -> PortState {
    let bind_addr = if addr.is_ipv4() {
        "0.0.0.0:0"
    } else {
        "[::]:0"
    };

    let socket = match tokio::net::UdpSocket::bind(bind_addr).await {
        Ok(s) => s,
        Err(_) => return PortState::Filtered,
    };

    if socket.connect(addr).await.is_err() {
        return PortState::Unreachable;
    }

    if socket.send(&[]).await.is_err() {
        return PortState::Filtered;
    }

    let mut buf = [0u8; 512];

    match tokio::time::timeout(wait, socket.recv(&mut buf)).await {
        Ok(Ok(_n)) => PortState::Open,
        Ok(Err(e))
            if matches!(
                e.kind(),
                ErrorKind::ConnectionRefused | ErrorKind::ConnectionReset
            ) =>
        {
            PortState::Closed
        }
        Ok(Err(_)) => PortState::Filtered,
        Err(_elapsed) => PortState::OpenOrFiltered,
    }
}
