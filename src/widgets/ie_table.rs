use kawaiifi::{Ie, InformationElement};
use std::{fmt::Display, ops::Deref};
use tui::{
    buffer::Buffer,
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    widgets::{
        Block, Borders, List, ListItem, ListState, Row, StatefulWidget, Table, TableState, Widget,
    },
};

pub struct IeTable {
    layout: Layout,
}

#[derive(Debug)]
pub struct IeTableState {
    ies: Vec<Ie>,
    rows: Vec<IeTableRow>,
    column_headers: Vec<IeTableColumnHeader>,
    table_state: TableState,
    list_state: ListState,
    is_focused: bool,
}

#[derive(Debug)]
struct IeTableRow {
    values: Vec<String>,
}

#[derive(Debug)]
pub enum IeTableColumnHeader {
    Id,
    Element,
    Length,
}

impl IeTable {
    pub fn new() -> IeTable {
        IeTable {
            layout: Layout::default()
                .direction(Direction::Horizontal)
                .margin(1)
                .constraints([Constraint::Percentage(33), Constraint::Percentage(67)]),
        }
    }
}

impl StatefulWidget for IeTable {
    type State = IeTableState;

    fn render(self, area: Rect, buf: &mut Buffer, state: &mut Self::State) {
        let areas = self.layout.split(area);

        let widths = &state
            .column_headers
            .iter()
            .map(|_| Constraint::Percentage((100f64 / state.column_headers.len() as f64) as u16))
            .collect::<Vec<Constraint>>();
        StatefulWidget::render(
            Table::new(
                state.column_headers.iter(),
                state.rows.iter().map(|row| Row::Data(row.iter())),
            )
            .block(
                Block::default()
                    .title("Information Elements")
                    .borders(Borders::ALL)
                    .border_style(if state.is_focused {
                        Style::default().fg(Color::Yellow)
                    } else {
                        Style::default()
                    }),
            )
            .widths(widths)
            .highlight_style(
                Style::default()
                    .fg(Color::Yellow)
                    .add_modifier(Modifier::BOLD),
            ),
            areas[0],
            buf,
            &mut state.table_state,
        );

        // Only render the list of IE fields if an IE is selected in the table
        if let Some(ie) = state.selected_ie() {
            let rows = ie
                .information_fields()
                .iter()
                .map(|field| field.to_string())
                .collect::<Vec<String>>();
            let items = rows
                .iter()
                .map(|row| ListItem::new(row.as_str()))
                .collect::<Vec<ListItem>>();

            Widget::render(
                List::new(items).block(Block::default().title(ie.name()).borders(Borders::ALL)),
                areas[1],
                buf,
            );
        }
    }
}

impl IeTableState {
    pub fn new() -> Self {
        IeTableState::default()
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

    pub fn set_ies(&mut self, ies: &[Ie]) {
        let previously_selected_ie = {
            if let Some(selected) = self.table_state.selected() {
                self.ies.get(selected)
            } else {
                None
            }
        };

        // Find the position of the previously selected IE in the new IEs
        // Save that position to the table state
        if let Some(previously_selected_ie) = previously_selected_ie {
            self.table_state.select(
                ies.iter()
                    .position(|ie| ie.id() == previously_selected_ie.id()),
            );
        }

        // Make sure that an IE is always selected
        if self.table_state.selected().is_none() && !ies.is_empty() {
            self.table_state.select(Some(0));
        }

        self.ies = ies.to_vec();
        self.rows = ies
            .iter()
            .map(|ie| IeTableRow::new(ie, &self.column_headers))
            .collect();
    }

    pub fn select_next(&mut self) {
        if let Some(selected) = self.table_state.selected() {
            if self.ies.len() > selected + 1 {
                self.table_state.select(Some(selected + 1));
            }
        } else if !self.ies.is_empty() {
            self.table_state.select(Some(0));
        }
    }

    pub fn select_previous(&mut self) {
        if let Some(selected) = self.table_state.selected() {
            if selected > 0 && !self.ies.is_empty() {
                self.table_state.select(Some(selected - 1));
            }
        }
    }

    pub fn selected_ie(&self) -> Option<&Ie> {
        if let Some(selected) = self.table_state.selected() {
            self.ies.get(selected)
        } else {
            None
        }
    }
}

impl Default for IeTableState {
    fn default() -> Self {
        IeTableState {
            column_headers: vec![
                IeTableColumnHeader::Id,
                IeTableColumnHeader::Element,
                IeTableColumnHeader::Length,
            ],
            ies: Vec::default(),
            rows: Vec::default(),
            table_state: TableState::default(),
            list_state: ListState::default(),
            is_focused: false,
        }
    }
}

impl IeTableRow {
    fn new(ie: &Ie, column_headers: &[IeTableColumnHeader]) -> Self {
        IeTableRow {
            values: column_headers
                .iter()
                .map(|column_header| match column_header {
                    IeTableColumnHeader::Id => ie.id().to_string(),
                    IeTableColumnHeader::Element => ie.name().to_string(),
                    IeTableColumnHeader::Length => format!("{} B", ie.bytes().len()),
                })
                .collect(),
        }
    }
}

impl Deref for IeTableRow {
    type Target = Vec<String>;

    fn deref(&self) -> &Self::Target {
        &self.values
    }
}

impl Display for IeTableColumnHeader {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            IeTableColumnHeader::Id => write!(f, "ID"),
            IeTableColumnHeader::Element => write!(f, "Element"),
            IeTableColumnHeader::Length => write!(f, "Length"),
        }
    }
}
