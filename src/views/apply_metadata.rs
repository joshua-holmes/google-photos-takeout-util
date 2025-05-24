use std::{sync::mpsc, thread, time::Duration};

use super::utils::Receiver;
use crate::{AppState, services};
use eframe::egui;

use super::{ViewNavigation, Viewable};

#[derive(Default)]
pub struct ApplyMetadata {
    receiver: Option<Receiver<()>>,
}
impl Viewable for ApplyMetadata {
    fn show(
        &mut self,
        app: &mut AppState,
        ctx: &egui::Context,
        ui: &mut egui::Ui,
    ) -> Option<ViewNavigation> {
        if let Some(receiver) = self.receiver.take() {
            if receiver.rx.recv_timeout(Duration::from_millis(1)).is_ok() {
                receiver.handle.join().unwrap();
                return Some(ViewNavigation::Next);
            } else {
                self.receiver = Some(receiver);
            }
        } else {
            // spawn thread and execute metadata application
            let (tx, rx) = mpsc::channel();
            let path = app
                .picked_path
                .clone()
                .expect("Did not save file path correctly. Please report this unexpected bug.");
            let handle = thread::spawn(move || {
                services::extract_and_apply_metadata(&path);
                tx.send(()).unwrap();
            });
            self.receiver = Some(Receiver { handle, rx });
        }

        ui.vertical_centered(|ui| {
            ui.label("Applying metadata...");
            ui.spinner();
        });
        None
    }
}
