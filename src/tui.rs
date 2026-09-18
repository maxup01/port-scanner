use crate::primitives::{PortState, ScanEvent};
use crossbeam_channel::Receiver;
use ratatui::style::{Color, Modifier, Style};

pub fn port_state_color(state: &PortState) -> Color {
    match state {
        PortState::Open => Color::Green,
        PortState::Closed => Color::DarkGray,
        PortState::Filtered => Color::Yellow,
        PortState::OpenOrFiltered => Color::LightYellow,
        PortState::Unreachable => Color::Red,
    }
}

pub fn port_state_style(state: &PortState) -> Style {
    let style = Style::default().fg(port_state_color(state));

    match state {
        PortState::Open => style.add_modifier(Modifier::BOLD),
        PortState::Closed => style.add_modifier(Modifier::DIM),
        _ => style,
    }
}

pub fn render_tui(rx: Receiver<ScanEvent>) {}
