use crate::widgets::{
    BssTable, BssTableColumnHeader, BssTableState, IeTable, IeTableState, TableSortOrder,
};
use kawaiifi::Bss;
use std::io::{self, Stdout};
use tui::{
    backend::CrosstermBackend,
    layout::{Constraint, Direction, Layout},
    Terminal,
};

#[derive(Debug)]
pub struct App {
    bss_table: BssTableState,
    ie_table: IeTableState,
    layout: Layout,
}

impl App {
    pub fn new() -> Self {
        App::default()
    }

    pub fn update_scan_results(&mut self, scan_results: Vec<Bss>) {
        self.bss_table.set_scan_results(scan_results);
        self.reset_ie_table();
    }

    pub fn sort_bss_table(
        &mut self,
        column_header: BssTableColumnHeader,
        sort_order: TableSortOrder,
    ) {
        self.bss_table.sort(column_header, sort_order);
        self.reset_ie_table();
    }

    pub fn reset_ie_table(&mut self) {
        if let Some(selected_bss) = self.bss_table.selected_bss() {
            self.ie_table.set_ies(selected_bss.ies());
        }
    }

    pub fn select_next(&mut self) {
        if self.bss_table.is_focused() {
            self.bss_table.select_next();
        } else {
            self.ie_table.select_next();
        }
        self.reset_ie_table();
    }

    pub fn select_previous(&mut self) {
        if self.bss_table.is_focused() {
            self.bss_table.select_previous();
        } else {
            self.ie_table.select_previous();
        }
        self.reset_ie_table();
    }

    pub fn focus_next(&mut self) {
        if self.bss_table.is_focused() {
            self.ie_table.focus();
            self.bss_table.unfocus();
        } else if self.ie_table.is_focused() {
            self.bss_table.focus();
            self.ie_table.unfocus();
        }
    }

    pub fn render(&mut self, terminal: &mut Terminal<CrosstermBackend<Stdout>>) -> io::Result<()> {
        terminal.draw(|frame| {
            let chunks = self.layout.split(frame.size());
            frame.render_stateful_widget(BssTable::new(), chunks[0], &mut self.bss_table);
            frame.render_stateful_widget(IeTable::new(), chunks[1], &mut self.ie_table);
        })
    }
}

impl Default for App {
    fn default() -> Self {
        let mut bss_table_state = BssTableState::new();
        bss_table_state.focus();

        App {
            bss_table: bss_table_state,
            ie_table: IeTableState::new(),
            layout: Layout::default()
                .direction(Direction::Vertical)
                .margin(1)
                .constraints([Constraint::Percentage(50), Constraint::Percentage(50)].as_ref()),
        }
    }
}
