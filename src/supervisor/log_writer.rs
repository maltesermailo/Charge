use std::fmt::format;
use std::fs;
use std::fs::{File, OpenOptions};
use std::io::Write;
use std::ptr::null;
use std::str::FromStr;
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::mpsc::Receiver;
use serde_json::to_string;
use crate::supervisor::event::SyscallEvent;
use crate::utils::remove_first_char;

pub fn log_write_thread_main(rx: Receiver<SyscallEvent>, pid: u32, id: String, running_log_write: Arc<AtomicBool>) {
    let mut file: Option<File> = None;

    while running_log_write.load(Ordering::SeqCst) {
        let event: SyscallEvent = rx.recv().unwrap();

        if(event.pid == 0) {
            println!("Received empty pid, heartbeat");
            continue;
        }

        match file {
            None => {
                //Init
                let cmdline = std::fs::read_to_string(format!("/proc/{}/cmdline", event.pid)).unwrap_or_else(|error| {
                    println!("Couldn't find cmdline for process {}", event.pid);

                    return String::new();
                });
                let mut split_cmdline = cmdline.split('\0');

                println!("{}", cmdline);

                let mut executable = split_cmdline.next().unwrap_or_else(|| {
                    return "nocmdline";
                });

                if(executable.contains("charge_wrapper")) {
                    executable = split_cmdline.next().unwrap_or_else(|| {
                        return "nocmdline";
                    });
                }

                if(executable.starts_with("/")) {
                    executable = remove_first_char(executable);
                }

                let tmp = executable.replace("/", "-");
                executable = tmp.as_str();

                let mut path = format!("/var/log/charge_scmp/process/{}", executable);

                if(!id.is_empty() && id != "0") {
                    path.push_str(format!("-{}", id.as_str()).as_str());
                }
                path.push_str(format!("-{}.json", event.pid).as_str());

                println!("{}", path);

                file = Some(OpenOptions::new().create(true).truncate(true).read(true).write(true).open(path).unwrap());
            }
            _ => {}
        }

        let json = serde_json::to_string(&event).unwrap();

        writeln!(file.as_ref().unwrap(), "{}", json).expect("'Error while writing to log file");
    }
}