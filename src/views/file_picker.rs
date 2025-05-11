use crate::AppState;
use eframe::egui;
use std::path::PathBuf;

use super::utils::Receiver;
use super::{ViewNavigation, Viewable};

#[derive(Default)]
pub struct FilePicker {
    dropped_files: Vec<egui::DroppedFile>,
    receiver: Option<Receiver<PathBuf>>,
}
impl Viewable for FilePicker {
    fn show(
        &mut self,
        app: &mut AppState,
        ctx: &egui::Context,
        ui: &mut egui::Ui,
    ) -> Option<ViewNavigation> {
        let nav = ui.vertical_centered_justified(|ui| {
            ui.label("Drag-and-drop files onto the window!");
            if ui.button("Open file…").clicked() {
                let (tx, rx) = std::sync::mpsc::channel();
                let handle = std::thread::spawn(move || {
                    if let Some(path) = rfd::FileDialog::new().pick_file() {
                        match tx.send(path) {
                            Ok(_) => {}
                            Err(err) => println!("Uh oh {:?}", err.to_string()),
                        }
                    }
                });
                self.receiver = Some(Receiver { rx, handle });
            }

            if let Some(receiver) = self.receiver.take() {
                if let Ok(picked_path) = receiver.rx.recv() {
                    app.picked_path = Some(picked_path);
                    receiver.handle.join().unwrap();
                    return Some(ViewNavigation::Next);
                } else {
                    // put receiver back if not used
                    self.receiver = Some(receiver);
                }
            }

            // Show dropped files (if any):
            if !self.dropped_files.is_empty() {
                ui.group(|ui| {
                    ui.label("Dropped files:");

                    for file in &self.dropped_files {
                        let mut info = if let Some(path) = &file.path {
                            path.display().to_string()
                        } else if !file.name.is_empty() {
                            file.name.clone()
                        } else {
                            "???".to_owned()
                        };

                        let mut additional_info = vec![];
                        if !file.mime.is_empty() {
                            additional_info.push(format!("type: {}", file.mime));
                        }
                        if let Some(bytes) = &file.bytes {
                            additional_info.push(format!("{} bytes", bytes.len()));
                        }
                        if !additional_info.is_empty() {
                            info += &format!(" ({})", additional_info.join(", "));
                        }

                        ui.label(info);
                    }
                });
            }

            None
        });

        preview_files_being_dropped(ctx);

        // Collect dropped files:
        ctx.input(|i| {
            if !i.raw.dropped_files.is_empty() {
                self.dropped_files.clone_from(&i.raw.dropped_files);
            }
        });

        nav.inner
    }
}

fn preview_files_being_dropped(ctx: &egui::Context) {
    use egui::{Align2, Color32, Id, LayerId, Order, TextStyle};
    use std::fmt::Write as _;

    if !ctx.input(|i| i.raw.hovered_files.is_empty()) {
        let text = ctx.input(|i| {
            let mut text = "Dropping files:\n".to_owned();
            for file in &i.raw.hovered_files {
                if let Some(path) = &file.path {
                    write!(text, "\n{}", path.display()).ok();
                } else if !file.mime.is_empty() {
                    write!(text, "\n{}", file.mime).ok();
                } else {
                    text += "\n???";
                }
            }
            text
        });

        let painter =
            ctx.layer_painter(LayerId::new(Order::Foreground, Id::new("file_drop_target")));

        let screen_rect = ctx.screen_rect();
        painter.rect_filled(screen_rect, 0.0, Color32::from_black_alpha(192));
        painter.text(
            screen_rect.center(),
            Align2::CENTER_CENTER,
            text,
            TextStyle::Heading.resolve(&ctx.style()),
            Color32::WHITE,
        );
    }
}
