use crate::utils::scroll_config;
use eframe::egui;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ScrollMode {
    Vertical,
    Both,
}

pub struct ScrollablePaneConfig<'a> {
    pub title: &'a str,
    pub scroll_id: &'a str,
    pub scroll_mode: ScrollMode,
}

impl<'a> ScrollablePaneConfig<'a> {
    pub fn new(title: &'a str, scroll_id: &'a str) -> Self {
        Self {
            title,
            scroll_id,
            scroll_mode: ScrollMode::Vertical,
        }
    }

    pub fn new_both_scroll(title: &'a str, scroll_id: &'a str) -> Self {
        Self {
            title,
            scroll_id,
            scroll_mode: ScrollMode::Both,
        }
    }
}

pub fn render<F>(ui: &mut egui::Ui, config: &ScrollablePaneConfig, content: F)
where
    F: FnOnce(&mut egui::Ui),
{
    const HEADER_HEIGHT: f32 = 32.0;

    ui.allocate_ui_with_layout(
        egui::vec2(ui.available_width(), HEADER_HEIGHT),
        egui::Layout::top_down(egui::Align::Center),
        |ui| {
            ui.add_space(6.0);
            ui.heading(config.title);
            ui.add_space(6.0);
        },
    );

    ui.separator();

    let scroll_area = match config.scroll_mode {
        ScrollMode::Vertical => scroll_config::vertical_scroll(),
        ScrollMode::Both => scroll_config::both_scroll(),
    };

    scroll_area.id_source(config.scroll_id).show(ui, |ui| {
        scroll_config::set_full_width(ui);
        content(ui);
    });
}
