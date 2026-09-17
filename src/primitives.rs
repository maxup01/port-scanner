#[derive(Clone)]
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
}

pub enum PortState {
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
