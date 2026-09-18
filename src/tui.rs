use crate::port_scan::PortScanResult;
use crate::primitives::{NetworkTransmissionProtocol, PortState, ScanEvent};
use core::time::Duration;
use crossbeam_channel::{Receiver, TryRecvError};
use crossterm::event::{self, Event, KeyCode, KeyEventKind, KeyModifiers};
use ratatui::{
    Frame,
    layout::{Constraint, Layout, Margin, Rect},
    style::{Color, Modifier, Style, Stylize},
    text::{Line, Span},
    widgets::{
        Block, Cell, Gauge, Paragraph, Row, Scrollbar, ScrollbarOrientation, ScrollbarState, Table,
        TableState,
    },
};
use std::{net::IpAddr, ops::Range};

const TICK: Duration = Duration::from_millis(100);
const MAX_DRAIN_PER_FRAME: usize = 4096;

pub(crate) fn port_state_color(state: &PortState) -> Color {
    match state {
        PortState::Open => Color::Green,
        PortState::Closed => Color::DarkGray,
        PortState::Filtered => Color::Yellow,
        PortState::OpenOrFiltered => Color::LightYellow,
        PortState::Unreachable => Color::Red,
    }
}

pub(crate) fn port_state_style(state: &PortState) -> Style {
    let style = Style::default().fg(port_state_color(state));

    match state {
        PortState::Open => style.add_modifier(Modifier::BOLD),
        PortState::Closed => style.add_modifier(Modifier::DIM),
        _ => style,
    }
}

pub(crate) fn render_tui(rx: Receiver<ScanEvent>, target: IpAddr, port_range: (u16, u16)) {
    let mut terminal = ratatui::init();
    let mut tui = Tui::new(target, port_range);

    loop {
        tui.drain(&rx);

        if terminal.draw(|frame| tui.draw(frame)).is_err() {
            break;
        }

        if tui.handle_input() {
            break;
        }
    }

    ratatui::restore();
}

#[derive(Default)]
struct Summary {
    open: usize,
    open_or_filtered: usize,
    filtered: usize,
    closed: usize,
    unreachable: usize,
}

impl Summary {
    fn record(&mut self, state: &PortState) {
        let counter = match state {
            PortState::Open => &mut self.open,
            PortState::OpenOrFiltered => &mut self.open_or_filtered,
            PortState::Filtered => &mut self.filtered,
            PortState::Closed => &mut self.closed,
            PortState::Unreachable => &mut self.unreachable,
        };

        *counter += 1;
    }

    fn breakdown(&self) -> [(PortState, usize); 5] {
        [
            (PortState::Open, self.open),
            (PortState::OpenOrFiltered, self.open_or_filtered),
            (PortState::Filtered, self.filtered),
            (PortState::Closed, self.closed),
            (PortState::Unreachable, self.unreachable),
        ]
    }
}

struct Tui {
    target: IpAddr,
    port_range: (u16, u16),
    rows: Vec<(u16, PortScanResult)>,
    visible: Vec<usize>,
    summary: Summary,
    selected: usize,
    offset: usize,
    follow: bool,
    hide_closed: bool,
    done: bool,
}

impl Tui {
    fn new(target: IpAddr, port_range: (u16, u16)) -> Self {
        Self {
            target,
            port_range,
            rows: Vec::new(),
            visible: Vec::new(),
            summary: Summary::default(),
            selected: 0,
            offset: 0,
            follow: true,
            hide_closed: false,
            done: false,
        }
    }

    fn total_ports(&self) -> usize {
        let (first, last) = self.port_range;

        usize::from(last) - usize::from(first) + 1
    }

    /// Pulls whatever the scanner has produced without blocking.
    fn drain(&mut self, rx: &Receiver<ScanEvent>) {
        for _ in 0..MAX_DRAIN_PER_FRAME {
            match rx.try_recv() {
                Ok(ScanEvent::ScanResult { port, scan_results }) => self.push(port, scan_results),
                Ok(ScanEvent::ScanEnd) => self.done = true,
                Err(TryRecvError::Empty) => break,
                // The scanner hung up without a ScanEnd; nothing more is coming.
                Err(TryRecvError::Disconnected) => {
                    self.done = true;
                    break;
                }
            }
        }
    }

    fn push(&mut self, port: u16, result: PortScanResult) {
        for state in result.values() {
            self.summary.record(state);
        }

        let shown = !(self.hide_closed && is_all_closed(&result));
        let index = self.rows.len();

        self.rows.push((port, result));

        if shown {
            self.visible.push(index);
        }
    }

    fn rebuild_visible(&mut self) {
        let hide_closed = self.hide_closed;

        self.visible = self
            .rows
            .iter()
            .enumerate()
            .filter(|(_, (_, result))| !(hide_closed && is_all_closed(result)))
            .map(|(index, _)| index)
            .collect();
        self.selected = self.selected.min(self.visible.len().saturating_sub(1));
        self.offset = 0;
    }

    fn handle_input(&mut self) -> bool {
        if !event::poll(TICK).unwrap_or(false) {
            return false;
        }

        let Ok(Event::Key(key)) = event::read() else {
            return false;
        };

        if key.kind != KeyEventKind::Press {
            return false;
        }

        match key.code {
            KeyCode::Char('q') | KeyCode::Esc => return true,
            KeyCode::Char('c') if key.modifiers.contains(KeyModifiers::CONTROL) => return true,
            KeyCode::Up | KeyCode::Char('k') => self.scroll_by(-1),
            KeyCode::Down | KeyCode::Char('j') => self.scroll_by(1),
            KeyCode::PageUp => self.scroll_by(-20),
            KeyCode::PageDown => self.scroll_by(20),
            KeyCode::Home | KeyCode::Char('g') => {
                self.follow = false;
                self.selected = 0;
            }
            KeyCode::End | KeyCode::Char('G') => self.follow = true,
            KeyCode::Char('f') => {
                self.hide_closed = !self.hide_closed;
                self.rebuild_visible();
            }
            _ => {}
        }

        false
    }

    fn scroll_by(&mut self, delta: isize) {
        if self.visible.is_empty() {
            return;
        }

        self.follow = false;

        let last = self.visible.len() as isize - 1;

        self.selected = (self.selected as isize + delta).clamp(0, last) as usize;
    }

    fn draw(&mut self, frame: &mut Frame) {
        let [header, body, footer] = Layout::vertical([
            Constraint::Length(4),
            Constraint::Min(3),
            Constraint::Length(1),
        ])
        .areas(frame.area());

        self.draw_header(frame, header);
        self.draw_results(frame, body);
        self.draw_footer(frame, footer);
    }

    fn draw_header(&self, frame: &mut Frame, area: Rect) {
        let block = Block::bordered().title(" port-scanner ");
        let inner = block.inner(area);

        frame.render_widget(block, area);

        let [info, progress] =
            Layout::vertical([Constraint::Length(1), Constraint::Length(1)]).areas(inner);

        let (first, last) = self.port_range;

        let status = if self.done {
            "done".green().bold()
        } else {
            "scanning".cyan().bold()
        };

        let line = Line::from(vec![
            "target ".dim(),
            self.target.to_string().bold(),
            "   ports ".dim(),
            format!("{first}-{last}").bold(),
            "   ".into(),
            status,
        ]);

        frame.render_widget(Paragraph::new(line), info);

        let scanned = self.rows.len();

        let total = self.total_ports();

        let gauge = Gauge::default()
            .gauge_style(Style::default().fg(Color::Cyan).bg(Color::DarkGray))
            .ratio((scanned as f64 / total as f64).clamp(0.0, 1.0))
            .label(format!("{scanned}/{total}"));

        frame.render_widget(gauge, progress);
    }

    fn draw_results(&mut self, frame: &mut Frame, area: Rect) {
        let hidden = self.rows.len() - self.visible.len();

        let title = if hidden > 0 {
            format!(" results ({hidden} closed hidden) ")
        } else {
            " results ".to_string()
        };

        let block = Block::bordered().title(title);

        let inner = block.inner(area);

        frame.render_widget(block, area);

        if self.visible.is_empty() {
            let message = if self.done {
                "nothing to show"
            } else {
                "scanning…"
            };

            frame.render_widget(Paragraph::new(message.dim()), inner);

            return;
        }

        let height = usize::from(inner.height.saturating_sub(1));

        let window = self.window(height);

        let rows = self.visible[window].iter().map(|&index| {
            let (port, result) = &self.rows[index];

            Row::new(vec![
                Cell::from(Line::from(port.to_string()).right_aligned()),
                protocol_cell(result, &NetworkTransmissionProtocol::Tcp),
                protocol_cell(result, &NetworkTransmissionProtocol::Udp),
            ])
        });

        let header = Row::new(vec![
            Cell::from(Line::from("PORT").right_aligned()),
            Cell::from(NetworkTransmissionProtocol::Tcp.label()),
            Cell::from(NetworkTransmissionProtocol::Udp.label()),
        ])
        .style(Style::default().add_modifier(Modifier::BOLD));

        let table = Table::new(
            rows,
            [
                Constraint::Length(7),
                Constraint::Length(14),
                Constraint::Length(14),
            ],
        )
        .header(header)
        .column_spacing(2)
        .row_highlight_style(Style::default().add_modifier(Modifier::REVERSED));

        let mut state = TableState::default().with_selected(Some(self.selected - self.offset));

        frame.render_stateful_widget(table, inner, &mut state);

        if self.visible.len() > height {
            let mut scrollbar_state =
                ScrollbarState::new(self.visible.len() - height).position(self.offset);

            frame.render_stateful_widget(
                Scrollbar::new(ScrollbarOrientation::VerticalRight)
                    .begin_symbol(None)
                    .end_symbol(None),
                area.inner(Margin {
                    vertical: 1,
                    horizontal: 0,
                }),
                &mut scrollbar_state,
            );
        }
    }

    fn window(&mut self, height: usize) -> Range<usize> {
        if self.follow {
            self.selected = self.visible.len() - 1;
        }

        self.selected = self.selected.min(self.visible.len() - 1);

        if self.selected < self.offset {
            self.offset = self.selected;
        }

        if height > 0 && self.selected >= self.offset + height {
            self.offset = self.selected + 1 - height;
        }

        self.offset = self.offset.min(self.visible.len().saturating_sub(height));

        let end = (self.offset + height).min(self.visible.len());

        self.offset..end
    }

    fn draw_footer(&self, frame: &mut Frame, area: Rect) {
        const KEYS: &str = "↑↓ scroll · f hide closed · g/G top/bottom · q quit ";

        let [counts, keys] = Layout::horizontal([
            Constraint::Min(0),
            Constraint::Length(KEYS.chars().count() as u16),
        ])
        .areas(area);

        let mut spans = Vec::new();
        for (state, count) in self.summary.breakdown() {
            if !spans.is_empty() {
                spans.push(Span::raw("  "));
            }

            spans.push(Span::styled(
                format!("{} {}", state.label(), count),
                port_state_style(&state),
            ));
        }

        frame.render_widget(Paragraph::new(Line::from(spans)), counts);
        frame.render_widget(Paragraph::new(KEYS.dim()), keys);
    }
}

fn protocol_cell(result: &PortScanResult, protocol: &NetworkTransmissionProtocol) -> Cell<'static> {
    match result.get(protocol) {
        Some(state) => Cell::from(state.label()).style(port_state_style(state)),
        _ => Cell::from("-").style(Style::default().add_modifier(Modifier::DIM)),
    }
}

fn is_all_closed(result: &PortScanResult) -> bool {
    !result.is_empty() && result.values().all(|s| matches!(s, PortState::Closed))
}
