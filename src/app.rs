use crate::{cli::Cli, port_scan::port_scan, primitives::ScanEvent, tui::render_tui};
use clap::Parser;
use core::net::SocketAddr;
use crossbeam_channel::unbounded;
use std::net::IpAddr;

pub struct App {
    target_host: IpAddr,
    port_range: (u16, u16),
}

impl App {
    pub fn init() -> Self {
        let args = Cli::parse();

        let ip: IpAddr = args.address.parse().expect("invalid IP address");

        if args.first_port > args.last_port {
            panic!("invalid port range");
        }

        Self {
            target_host: ip,
            port_range: (args.first_port, args.last_port),
        }
    }

    pub async fn run(self) {
        let (tx, rx) = unbounded::<ScanEvent>();

        let (first_port, last_port) = self.port_range;
        let target_host = self.target_host;

        let scan_handle = tokio::spawn(async move {
            for port in first_port..=last_port {
                let addr = SocketAddr::new(target_host, port);

                let scan_results = port_scan(addr).await;

                if tx
                    .send(ScanEvent::ScanResult { port, scan_results })
                    .is_err()
                {
                    return;
                }
            }

            let _ = tx.send(ScanEvent::ScanEnd);
        });

        let tui_handle =
            tokio::task::spawn_blocking(move || render_tui(rx, target_host, self.port_range));

        let _ = tui_handle.await;
        scan_handle.abort();
    }
}
