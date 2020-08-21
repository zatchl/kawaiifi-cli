use crate::widgets::TableSortOrder;
use kawaiifi::Bss;
use std::{fmt::Display, iter::Iterator, ops::Deref};
use tui::{
    buffer::Buffer,
    layout::{Constraint, Rect},
    style::{Color, Modifier, Style},
    widgets::{Block, Borders, Row, StatefulWidget, Table, TableState},
};

pub struct BssTable;

#[derive(Debug)]
pub struct BssTableState {
    scan_results: Vec<Bss>,
    rows: Vec<BssTableRow>,
    column_headers: Vec<BssTableColumnHeader>,
    state: TableState,
    is_focused: bool,
    sorting: (BssTableColumnHeader, TableSortOrder),
}

#[derive(Debug)]
struct BssTableRow {
    values: Vec<String>,
}

#[derive(Debug, Copy, Clone, Eq, PartialEq)]
pub enum BssTableColumnHeader {
    Bssid,
    Ssid,
    Channel,
    ChannelWidth,
    Band,
    Frequency,
    Signal,
    WiFiProtocols,
    Security,
    MaxRate,
}

impl BssTable {
    pub fn new() -> BssTable {
        BssTable {}
    }
}

impl StatefulWidget for BssTable {
    type State = BssTableState;

    fn render(self, area: Rect, buf: &mut Buffer, state: &mut Self::State) {
        Table::new(
            state.header().iter().map(|column| {
                if *column == state.sorting.0 {
                    format!("{}*", column.to_string())
                } else {
                    format!("{}", column.to_string())
                }
            }),
            state.rows.iter().map(|row| Row::Data(row.iter())),
        )
        .block(
            Block::default()
                .title("Basic Service Sets")
                .borders(Borders::ALL)
                .border_style(if state.is_focused {
                    Style::default().fg(Color::Yellow)
                } else {
                    Style::default()
                }),
        )
        .widths(
            &state
                .column_headers
                .iter()
                .map(|_| {
                    Constraint::Percentage((100f64 / state.column_headers.len() as f64) as u16)
                })
                .collect::<Vec<Constraint>>(),
        )
        .style(Style::default())
        .column_spacing(1)
        .highlight_style(
            Style::default()
                .fg(Color::Yellow)
                .add_modifier(Modifier::BOLD),
        )
        .render(area, buf, &mut state.state.clone());
    }
}

impl BssTableState {
    pub fn new() -> Self {
        BssTableState::default()
    }

    pub fn is_focused(&self) -> bool {
        self.is_focused
    }

    pub fn focus(&mut self) {
        self.is_focused = true;
    }

    pub fn unfocus(&mut self) {
        self.is_focused = false;
    }

    pub fn set_scan_results(&mut self, mut scan_results: Vec<Bss>) {
        scan_results.sort_by_column(self.sorting.0, self.sorting.1);
        let previously_selected_bss = {
            if let Some(selected) = self.state.selected() {
                self.scan_results.get(selected)
            } else {
                None
            }
        };

        // Find the position of the previously selected BSS in the new scan results
        // Save that position to the table state
        if let Some(previously_selected_bss) = previously_selected_bss {
            self.state.select(
                scan_results
                    .iter()
                    .position(|bss| bss == previously_selected_bss),
            );
        }

        // Make sure that a BSS is always selected
        if self.state.selected().is_none() && !scan_results.is_empty() {
            self.state.select(Some(0));
        }

        self.scan_results = scan_results;
        self.rows = self
            .scan_results
            .iter()
            .map(|bss| BssTableRow::new(bss, &self.column_headers))
            .collect();
    }

    pub fn select_next(&mut self) {
        if let Some(selected) = self.state.selected() {
            if self.scan_results.len() > selected + 1 {
                self.state.select(Some(selected + 1));
            }
        } else if !self.scan_results.is_empty() {
            self.state.select(Some(0));
        }
    }

    pub fn select_previous(&mut self) {
        if let Some(selected) = self.state.selected() {
            if selected > 0 && !self.scan_results.is_empty() {
                self.state.select(Some(selected - 1));
            }
        }
    }

    pub fn selected_bss(&self) -> Option<&Bss> {
        if let Some(selected) = self.state.selected() {
            self.scan_results.get(selected)
        } else {
            None
        }
    }

    pub fn header(&self) -> &Vec<BssTableColumnHeader> {
        &self.column_headers
    }

    pub fn sort(&mut self, column: BssTableColumnHeader, sort_order: TableSortOrder) {
        self.scan_results.sort_by_column(column, sort_order);
        self.rows = self
            .scan_results
            .iter()
            .map(|bss| BssTableRow::new(bss, &self.column_headers))
            .collect();
        self.sorting = (column, sort_order);
        if !self.scan_results.is_empty() {
            self.state.select(Some(0));
        }
    }
}

impl Default for BssTableState {
    fn default() -> Self {
        BssTableState {
            column_headers: vec![
                BssTableColumnHeader::Bssid,
                BssTableColumnHeader::Ssid,
                BssTableColumnHeader::Channel,
                BssTableColumnHeader::ChannelWidth,
                BssTableColumnHeader::Band,
                BssTableColumnHeader::Frequency,
                BssTableColumnHeader::Signal,
                BssTableColumnHeader::WiFiProtocols,
                BssTableColumnHeader::Security,
                BssTableColumnHeader::MaxRate,
            ],
            scan_results: Vec::default(),
            rows: Vec::default(),
            state: TableState::default(),
            is_focused: false,
            sorting: (BssTableColumnHeader::Bssid, TableSortOrder::Descending),
        }
    }
}

impl BssTableRow {
    fn new(bss: &Bss, column_headers: &[BssTableColumnHeader]) -> Self {
        BssTableRow {
            values: column_headers
                .iter()
                .map(|column_header| match column_header {
                    BssTableColumnHeader::Bssid => bss.bssid().to_string(),
                    BssTableColumnHeader::Ssid => bss.ssid().unwrap_or_default().to_string(),
                    BssTableColumnHeader::Channel => bss.channel().number().to_string(),
                    BssTableColumnHeader::ChannelWidth => bss.channel().width().to_string(),
                    BssTableColumnHeader::Band => bss.channel().band().to_string(),
                    BssTableColumnHeader::Frequency => {
                        format!("{} MHz", bss.channel().center_freq_mhz())
                    }
                    BssTableColumnHeader::Signal => format!("{} dBm", bss.signal_dbm()),
                    BssTableColumnHeader::WiFiProtocols => bss.wifi_protocols().to_string(),
                    BssTableColumnHeader::Security => "None".to_string(),
                    BssTableColumnHeader::MaxRate => "0.0 Mbps".to_string(),
                })
                .collect(),
        }
    }
}

impl Deref for BssTableRow {
    type Target = Vec<String>;

    fn deref(&self) -> &Self::Target {
        &self.values
    }
}

trait Sortable {
    fn sort_by_column(&mut self, column: BssTableColumnHeader, sort_order: TableSortOrder);
}

impl Sortable for Vec<Bss> {
    fn sort_by_column(&mut self, column: BssTableColumnHeader, sort_order: TableSortOrder) {
        let compare_bss = |a: &Bss, b: &Bss| match column {
            BssTableColumnHeader::Bssid => a.bssid().cmp(b.bssid()),
            BssTableColumnHeader::Ssid => a.ssid().cmp(&b.ssid()),
            BssTableColumnHeader::Channel => a.channel().number().cmp(&b.channel().number()),
            BssTableColumnHeader::ChannelWidth => a.channel().width().cmp(&b.channel().width()),
            BssTableColumnHeader::Band => a.channel().band().cmp(&b.channel().band()),
            BssTableColumnHeader::Frequency => a
                .channel()
                .center_freq_mhz()
                .cmp(&b.channel().center_freq_mhz()),
            BssTableColumnHeader::Signal => a.signal_dbm().cmp(&b.signal_dbm()),
            BssTableColumnHeader::WiFiProtocols => a.wifi_protocols().cmp(&b.wifi_protocols()),
            _ => a.bssid().cmp(b.bssid()),
        };

        self.sort_by(|a, b| match sort_order {
            TableSortOrder::Ascending => compare_bss(a, b).reverse(),
            TableSortOrder::Descending => compare_bss(a, b),
        });
    }
}

impl Display for BssTableColumnHeader {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            BssTableColumnHeader::Bssid => write!(f, "BSSID"),
            BssTableColumnHeader::Ssid => write!(f, "SSID"),
            BssTableColumnHeader::Channel => write!(f, "Channel"),
            BssTableColumnHeader::ChannelWidth => write!(f, "Channel Width"),
            BssTableColumnHeader::Band => write!(f, "Band"),
            BssTableColumnHeader::Frequency => write!(f, "Frequency"),
            BssTableColumnHeader::Signal => write!(f, "Signal"),
            BssTableColumnHeader::WiFiProtocols => write!(f, "Wi-Fi Protocols"),
            BssTableColumnHeader::Security => write!(f, "Security"),
            BssTableColumnHeader::MaxRate => write!(f, "Max Rate"),
        }
    }
}
