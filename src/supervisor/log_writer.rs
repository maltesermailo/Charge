use std::fmt::format;
use std::fs::File;
use std::io::Read;
use std::ptr::null;
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::mpsc::Receiver;
use crate::supervisor::event::SyscallEvent;

pub async fn log_write_thread_main(rx: Receiver<SyscallEvent>, running_log_write: Arc<AtomicBool>) {
    let mut file: Option<File> = None;

    while running_log_write.load(Ordering::SeqCst) {
        let event: SyscallEvent = rx.recv().await;

        match file {
            None => {
                //Init
                let mut cmdline = std::fs::read_to_string(format!("/proc/{}/cmdline", event.pid)).unwrap_or_else(|error| {
                    println!("Couldn't find cmdline for process {}", event.pid);

                    return String::new();
                });
                cmdline = cmdline.split(' ')[0];

                file = Some(File::open(format!("/var/log/charge_scmp/process/{}-{}.json", cmdline, event.pid)).unwrap());
            }
            _ => {}
        }


    }
}