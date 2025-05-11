use std::{cell::RefCell, rc::Rc};

use eframe::egui;

mod apply_metadata;
mod file_picker;
mod success;
pub mod utils;

use apply_metadata::ApplyMetadata;
use file_picker::FilePicker;
use success::Success;

use crate::AppState;

/// A double linked list to allow traversing to prev and next views easily. Also allows inserting new ones with O(1)
/// runtime.
pub struct View {
    pub prev: Option<Rc<RefCell<View>>>,
    pub next: Option<Rc<RefCell<View>>>,
    pub item: Box<dyn Viewable>,
}
impl Default for View {
    fn default() -> Self {
        // list of view in order from first to last
        let views: [Box<dyn Viewable>; 3] = [
            Box::new(FilePicker::default()),
            Box::new(ApplyMetadata::default()),
            Box::new(Success),
        ];

        // build views as type `View`
        let mut root = None;
        for view in views.into_iter().rev() {
            let mut v = Self {
                prev: None,
                next: None,
                item: view,
            };

            if let Some(root) = root {
                v.next = Some(Rc::new(RefCell::new(root)));
            }

            root = Some(v);
        }

        root.expect("No views were created! Please report unexpected this bug.")
    }
}

pub trait Viewable {
    fn show(
        &mut self,
        app: &mut AppState,
        ctx: &egui::Context,
        ui: &mut egui::Ui,
    ) -> Option<ViewNavigation>;
}

#[derive(Clone)]
pub enum ViewNavigation {
    Prev,
    Next,
}
