#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")] // hide console window on Windows in release
#![allow(rustdoc::missing_crate_level_docs)] // it's an example

use std::{cell::RefCell, path::PathBuf, rc::Rc};

use eframe::egui;
use views::{View, ViewNavigation};

mod services;
mod views;

fn main() -> eframe::Result {
    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_inner_size([640.0, 240.0])
            .with_drag_and_drop(true),
        ..Default::default()
    };

    // Our application state:
    eframe::run_native(
        "Native file dialogs and drag-and-drop files",
        options,
        Box::new(|_cc| Ok(Box::<MyApp>::default())),
    )
}

#[derive(Default)]
struct MyApp {
    current_view: Rc<RefCell<View>>,
    app_state: AppState,
}

#[derive(Default)]
struct AppState {
    picked_path: Option<PathBuf>,
}

impl eframe::App for MyApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        let nav = egui::CentralPanel::default().show(ctx, |ui| {
            let mut cur_view = self.current_view.borrow_mut();
            cur_view.item.show(&mut self.app_state, ctx, ui)
        });

        if let Some(nav) = nav.inner {
            match nav {
                ViewNavigation::Prev => {
                    let prev = self.current_view.borrow().prev.as_ref().unwrap().clone();
                    self.current_view = prev;
                }
                ViewNavigation::Next => {
                    let next = self.current_view.borrow().next.as_ref().unwrap().clone();
                    self.current_view = next;
                }
            };
        }
    }
}
