use std::fmt::format;
use std::fs::File;
use std::io::Read;
use std::ptr::null;
use std::str::FromStr;
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::mpsc::Receiver;
use crate::supervisor::event::SyscallEvent;

pub fn log_write_thread_main(rx: Receiver<SyscallEvent>, running_log_write: Arc<AtomicBool>) {
    let mut file: Option<File> = None;

    while running_log_write.load(Ordering::SeqCst) {
        let event: SyscallEvent = rx.recv().unwrap();

        match file {
            None => {
                //Init
                let cmdline = std::fs::read_to_string(format!("/proc/{}/cmdline", event.pid)).unwrap_or_else(|error| {
                    println!("Couldn't find cmdline for process {}", event.pid);

                    return String::new();
                });
                let mut split_cmdline = cmdline.split(" ");
                let executable = split_cmdline.next().unwrap_or_else(|| {
                    return "nocmdline";
                });

                file = Some(File::open(format!("/var/log/charge_scmp/process/{}-{}.json", executable, event.pid)).unwrap());
            }
            _ => {}
        }


    }
}