use crate::{cli::Cli, port_scan::port_scan};
use clap::Parser;
use core::net::SocketAddr;
use std::net::IpAddr;

pub async fn run() {
    let args = Cli::parse();

    let ip: IpAddr = args.address.parse().expect("invalid IP address");

    for port in args.first_port..=args.last_port {
        let addr = SocketAddr::new(ip, port);

        let states = port_scan(addr);
    }
}
