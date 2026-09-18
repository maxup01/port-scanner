use crate::{
    cli::Cli,
    port_scan::{PortScanResult, port_scan},
    primitives::ScanEvent,
    tui::render_tui,
};
use clap::Parser;
use core::net::SocketAddr;
use crossbeam_channel::unbounded;
use std::net::IpAddr;

pub struct App {
    target_host: String,
    port_range: (u16, u16),
    results: Vec<PortScanResult>,
}

impl App {
    pub async fn run() {
        let args = Cli::parse();

        let ip: IpAddr = args.address.parse().expect("invalid IP address");

        let (tx, rx) = unbounded::<ScanEvent>();

        let scan_handle = tokio::spawn(async move {
            for port in args.first_port..=args.last_port {
                let addr = SocketAddr::new(ip, port);

                let scan_results = port_scan(addr).await;

                let _ = tx.send(ScanEvent::ScanResult { port, scan_results });
            }

            let _ = tx.send(ScanEvent::ScanEnd);
        });

        let tui_handle = tokio::spawn(async move {
            render_tui(rx);
        });

        tokio::select! {
            _ = scan_handle => {},
            _ = tui_handle => {}
        }
    }
}
