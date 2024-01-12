use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::mpsc::Receiver;
use crate::supervisor::event::SyscallEvent;

pub fn log_write_thread_main(rx: Receiver<SyscallEvent>, running_log_write: Arc<AtomicBool>) {
    while running_log_write.load(Ordering::SeqCst) {

    }
}