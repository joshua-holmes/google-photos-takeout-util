use crate::AppState;
use eframe::egui;

use super::{ViewNavigation, Viewable};

pub struct Success;
impl Viewable for Success {
    fn show(
        &mut self,
        app: &mut AppState,
        ctx: &egui::Context,
        ui: &mut egui::Ui,
    ) -> Option<ViewNavigation> {
        ui.centered_and_justified(|ui| {
            ui.heading("Success!");
            ui.label("You can close the application now.");
        });
        None
    }
}
