mod bss_table;
mod ie_table;

pub use bss_table::BssTable;
pub use bss_table::BssTableColumnHeader;
pub use bss_table::BssTableState;

pub use ie_table::IeTable;
pub use ie_table::IeTableState;

#[derive(Debug, Copy, Clone)]
pub enum TableSortOrder {
    Ascending,
    Descending,
}
