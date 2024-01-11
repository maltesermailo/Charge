use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::mpsc::Receiver;

pub fn log_write_thread_main(rx: Receiver<&str>, runningLogWrite: Arc<AtomicBool>) {
    while runningLogWrite.load(Ordering::SeqCst) {

    }
}