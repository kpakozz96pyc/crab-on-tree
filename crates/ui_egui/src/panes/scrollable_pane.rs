use crate::utils::scroll_config;
use eframe::egui;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ScrollMode {
    Vertical,
    Both,
}

pub struct ScrollablePaneConfig<'a> {
    pub scroll_id: &'a str,
    pub scroll_mode: ScrollMode,
}

impl<'a> ScrollablePaneConfig<'a> {
    pub fn new(scroll_id: &'a str) -> Self {
        Self {
            scroll_id,
            scroll_mode: ScrollMode::Vertical,
        }
    }

    pub fn new_both_scroll(scroll_id: &'a str) -> Self {
        Self {
            scroll_id,
            scroll_mode: ScrollMode::Both,
        }
    }
}

pub fn render<F>(ui: &mut egui::Ui, config: &ScrollablePaneConfig, content: F)
where
    F: FnOnce(&mut egui::Ui),
{
    let scroll_area = match config.scroll_mode {
        ScrollMode::Vertical => scroll_config::vertical_scroll(),
        ScrollMode::Both => scroll_config::both_scroll(),
    };

    scroll_area.id_source(config.scroll_id).show(ui, |ui| {
        scroll_config::set_full_width(ui);
        content(ui);
    });
}
