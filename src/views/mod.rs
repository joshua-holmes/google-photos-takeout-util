use eframe::egui;
use file_picker::FilePicker;

use crate::{AppState, MyApp};

pub mod file_picker;

/// A double linked list to allow traversing to prev and next views easily. Also allows inserting new ones with O(1)
/// runtime.
pub struct View {
    pub prev: Option<Box<View>>,
    pub next: Option<Box<View>>,
    item: Box<dyn Viewable>,
}
impl View {
    /// Insert view into linked list of views
    pub fn insert(&mut self, mut view: View) {
        let next = self.next.take();
        view.next = next;
        self.next = Some(Box::new(view));
    }

    pub fn show(&mut self, app: &mut AppState, ctx: &egui::Context, ui: &mut egui::Ui) {
        self.item.show(app, ctx, ui);
    }
}
impl Default for View {
    fn default() -> Self {
        Self {
            prev: None,
            next: None,
            item: Box::new(FilePicker),
        }
    }
}

pub trait Viewable {
    fn show(&self, app: &mut AppState, ctx: &egui::Context, ui: &mut egui::Ui);
}
