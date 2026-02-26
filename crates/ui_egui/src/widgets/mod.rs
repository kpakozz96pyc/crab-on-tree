//! Reusable UI widgets for the CrabOnTree application.

pub mod bottom_panel;
pub mod diff_view;
pub mod file_content;
pub mod file_row;
pub mod selectable_row;

pub use bottom_panel::render_with_bottom_panel;
pub use diff_view::DiffView;
pub use file_content::FileContentView;
pub use file_row::{FileRow, FileRowInteraction};
pub use selectable_row::selectable_row;
