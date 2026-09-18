use crate::primitives::PortState;
use ratatui::style::Color;

pub fn port_state_color(state: PortState) -> Color {
    match state {
        PortState::Open => Color::Green,
        PortState::Closed => Color::DarkGray,
        PortState::Filtered => Color::Yellow,
        PortState::OpenOrFiltered => Color::LightYellow,
        PortState::Unreachable => Color::Red,
    }
}
