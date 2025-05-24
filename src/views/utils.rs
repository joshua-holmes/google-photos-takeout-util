use std::{sync::mpsc, thread};

#[derive(Debug)]
pub struct Receiver<T> {
    pub rx: mpsc::Receiver<T>,
    pub handle: thread::JoinHandle<()>,
}
