//! Framework-agnostic UI types for CrabOnTree.
//!
//! This crate provides UI primitives like colors, themes, and keyboard shortcuts
//! that can be used across different UI frameworks.

pub mod color;
pub mod date_format;
pub mod shortcuts;
pub mod theme;

pub use color::Color;
pub use date_format::{format_absolute_time, format_relative_time};
pub use shortcuts::{Action, Key, Modifiers, Shortcut};
pub use theme::Theme;
