use crate::primitives::{NetworkTransmissionProtocol, PortState};
use core::{net::SocketAddr, time::Duration};
use std::{collections::HashMap, io::ErrorKind};

pub(crate) async fn port_scan(addr: SocketAddr) -> PortScanResult {
    let mut scan_result: PortScanResult = HashMap::new();

    for protocol in NetworkTransmissionProtocol::protocols() {
        let result = match protocol {
            NetworkTransmissionProtocol::Tcp => {
                check_tcp_state(addr, Duration::from_millis(2500)).await
            }
            NetworkTransmissionProtocol::Udp => {
                check_udp_state(addr, Duration::from_millis(2500)).await
            }
        };

        scan_result.insert(protocol.clone(), result);
    }

    scan_result
}

pub(crate) type PortScanResult = HashMap<NetworkTransmissionProtocol, PortState>;

async fn check_tcp_state(addr: SocketAddr, wait: Duration) -> PortState {
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
