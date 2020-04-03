use std::sync::mpsc;
use std::thread;
use std::thread::JoinHandle;

pub struct Progress<T> {
    pub handle: JoinHandle<T>,
    pub recver: mpsc::Receiver<String>,
}

impl<T: Send + 'static> Progress<T> {
    pub fn from_fn<F: (FnOnce(mpsc::Sender<String>) -> T) + Send + 'static>(f: F) -> Progress<T> {
        let (sender, recver) = mpsc::channel();
        let handle = thread::spawn(move || f(sender));
        Progress { handle, recver }
    }
}
