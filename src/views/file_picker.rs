use crate::AppState;
use eframe::egui;
use std::time::Duration;

use super::Viewable;

pub struct FilePicker;
impl Viewable for FilePicker {
    fn show(&self, app: &mut AppState, ctx: &egui::Context, ui: &mut egui::Ui) {
        ui.vertical_centered_justified(|ui| {
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
                app.rx = Some(rx);
                app.handle = Some(handle);
            }

            if let Some(picked_path) = app
                .rx
                .as_ref()
                .and_then(|rx| rx.recv_timeout(Duration::new(0, 1_000_000)).ok())
            {
                app.picked_path = Some(picked_path);
                app.rx = None;
                app.handle.take().unwrap().join().unwrap();
            }

            if let Some(picked_path) = app.picked_path.as_ref() {
                ui.horizontal(|ui| {
                    ui.label("Picked file:");
                    ui.monospace(picked_path.display().to_string());
                });
            }

            // Show dropped files (if any):
            if !app.dropped_files.is_empty() {
                ui.group(|ui| {
                    ui.label("Dropped files:");

                    for file in &app.dropped_files {
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
        });

        preview_files_being_dropped(ctx);

        // Collect dropped files:
        ctx.input(|i| {
            if !i.raw.dropped_files.is_empty() {
                app.dropped_files.clone_from(&i.raw.dropped_files);
            }
        });
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
