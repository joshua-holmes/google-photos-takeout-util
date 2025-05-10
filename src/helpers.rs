use std::{sync::mpsc, thread};

pub struct Receiver<T> {
    pub rx: mpsc::Receiver<T>,
    pub handle: thread::JoinHandle<()>,
}
