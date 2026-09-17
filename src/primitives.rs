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
