use std::{io, mem::MaybeUninit, path::PathBuf, sync::mpsc, thread, time::Duration};

use crate::{AppState, services};
use eframe::egui;

use super::{ViewNavigation, Viewable};

#[derive(Debug)]
pub struct ThreadManager<R, T> {
    pub rx: mpsc::Receiver<R>,
    pub tx: mpsc::Sender<T>,
    pub handle: thread::JoinHandle<()>,
}

#[derive(Default)]
pub struct ApplyMetadata {
    thread_manager: Option<ThreadManager<Option<(PathBuf, io::Error)>, ()>>,
    error: Option<(PathBuf, io::Error)>,
}
impl Viewable for ApplyMetadata {
    fn show(
        &mut self,
        app: &mut AppState,
        ctx: &egui::Context,
        ui: &mut egui::Ui,
    ) -> Option<ViewNavigation> {
        if let Some(receiver) = self.thread_manager.take() {
            if let Ok(maybe_err) = receiver.rx.recv_timeout(Duration::from_millis(1)) {
                self.error = maybe_err;
                if self.error.is_none() {
                    receiver.handle.join().unwrap();
                    return Some(ViewNavigation::Next);
                }
                self.thread_manager = Some(receiver);
            } else {
                self.thread_manager = Some(receiver);
            }
        } else {
            // spawn thread and execute metadata application
            let (tx_confirm, rx_confirm) = mpsc::channel();
            let (tx_err, rx_err) = mpsc::channel();
            let path = app
                .picked_path
                .clone()
                .expect("Did not save file path correctly. Please report this unexpected bug.");
            let handle = thread::spawn(move || {
                services::extract_and_apply_metadata(&path, &rx_confirm, &tx_err);
                if let Err(err) = tx_err.send(None) {
                    panic!("Failed to signal end of metadata application: {}", err);
                }
            });
            self.thread_manager = Some(ThreadManager {
                handle,
                rx: rx_err,
                tx: tx_confirm,
            });
        }

        ui.vertical_centered(|ui| {
            if let Some((img_path, err)) = self.error.as_ref() {
                ui.label("An error has occurred for file:");
                ui.label(img_path.to_str().unwrap());
                ui.label(err.to_string());
                if ui.button("Ok").clicked() {
                    self.thread_manager
                        .as_ref()
                        .unwrap()
                        .tx
                        .send(())
                        .expect("Could not confirm error");
                    self.error = None;
                }
            } else {
                ui.label("Applying metadata...");
                ui.spinner();
            }
        });
        None
    }
}
