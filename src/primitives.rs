use crate::port_scan::PortScanResult;

#[derive(Eq, Hash, PartialEq, Clone)]
pub(crate) enum NetworkTransmissionProtocol {
    Tcp,
    Udp,
}

impl NetworkTransmissionProtocol {
    pub fn protocols() -> &'static [NetworkTransmissionProtocol] {
        const PROTOCOLS: [NetworkTransmissionProtocol; 2] = [
            NetworkTransmissionProtocol::Udp,
            NetworkTransmissionProtocol::Tcp,
        ];

        &PROTOCOLS
    }

    pub fn label(&self) -> &'static str {
        match self {
            NetworkTransmissionProtocol::Tcp => "TCP",
            NetworkTransmissionProtocol::Udp => "UDP",
        }
    }
}

pub(crate) enum PortState {
    /// A service accepted the connection (TCP) or replied to a probe (UDP).
    Open,
    /// The host answered but nothing is listening: TCP RST, or ICMP port
    /// unreachable for UDP.
    Closed,
    /// No answer at all, or an active rejection. Almost always a firewall.
    Filtered,
    /// UDP only: silence, which looks identical for an open port that ignores
    /// our probe and a filtered one.
    OpenOrFiltered,
    /// The host itself is unreachable, so nothing can be said about the port.
    Unreachable,
}

impl PortState {
    /// Short label for the UI.
    pub fn label(&self) -> &'static str {
        match self {
            PortState::Open => "open",
            PortState::Closed => "closed",
            PortState::Filtered => "filtered",
            PortState::OpenOrFiltered => "open|filtered",
            PortState::Unreachable => "unreachable",
        }
    }
}

pub(crate) enum ScanEvent {
    ScanResult {
        port: u16,
        scan_results: PortScanResult,
    },
    ScanEnd,
}
