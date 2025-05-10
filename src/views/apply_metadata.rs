use crate::AppState;
use eframe::egui;

use super::{ViewNavigation, Viewable};

pub struct ApplyMetadata;
impl Viewable for ApplyMetadata {
    fn show(&mut self, app: &mut AppState, ctx: &egui::Context, ui: &mut egui::Ui) -> Option<ViewNavigation> {
        ui.centered_and_justified(|ui| {
            ui.spinner();
        });
        None
    }
}
