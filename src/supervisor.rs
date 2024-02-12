mod log_writer;
mod listener;
mod event;

use std::os::fd::{RawFd};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, mpsc};
use std::thread;
use nix::libc;
use nix::libc::{c_int, SECCOMP_GET_NOTIF_SIZES, seccomp_notif_sizes, SYS_seccomp};
use crate::Cli;
use crate::supervisor::listener::listener_thread_main;
use crate::supervisor::log_writer::log_write_thread_main;
use tokio::signal::unix::{signal, SignalKind};

#[repr(C)]
pub struct seccomp_data {
    pub nr: c_int,
    pub arch: u32,
    pub instruction_pointer: u64,
    pub args: [u64; 6],
}

#[repr(C)]
pub struct seccomp_notif {
    pub id: u64,
    pub pid: u32,
    pub flags: u32,
    pub data: seccomp_data,
}

pub fn supervisor_main(cmd: Cli) {
    println!("Running in supervisor mode!");

    if cmd.fd == 0 {
        panic!("No file descriptor provided!");
    }

    let raw_fd: RawFd = cmd.fd as RawFd;
    let mut notif_sizes: seccomp_notif_sizes = seccomp_notif_sizes {
        seccomp_notif: 0,
        seccomp_notif_resp: 0,
        seccomp_data: 0,
    };

    let mut stream = signal(SignalKind::terminate()).unwrap();

    unsafe {
        let raw_notif_sizes = &mut notif_sizes as *mut seccomp_notif_sizes;

        let error = libc::syscall(SYS_seccomp, SECCOMP_GET_NOTIF_SIZES, 0, raw_notif_sizes);

        if error != 0 {
            panic!("Error {} at SECCOMP_GET_NOTIF_SIZES syscall", error);
        }
    }

    println!("SECCOMP NOTIF SIZE: {}", notif_sizes.seccomp_notif);
    println!("SECCOMP NOTIF RESPONSE SIZE: {}", notif_sizes.seccomp_notif_resp);
    println!("SECCOMP NOTIF DATA SIZE: {}", notif_sizes.seccomp_data);
    println!("RUST NOTIF SIZE: {}", std::mem::size_of::<seccomp_notif>());

    //This thread stays on the listener thread and waits for seccomp messages. The other thread will run the log and file writer thread

    std::fs::create_dir_all("/var/log/charge_scmp/process/").expect("Error while creating directory");

    let (tx, rx) = mpsc::channel();
    let running = Arc::new(AtomicBool::new(true));
    let running_log_write = Arc::clone(&running);
    let running_signal = Arc::clone(&running);

    //Spawn log writer thread
    thread::spawn(move || {
        log_write_thread_main(rx, running_log_write);
    });

    //Spawn signal thread for systemd
    thread::spawn(move || async move {
        loop {
            stream.recv().await;

            running_signal.store(false, Ordering::SeqCst);
        }
    });

    listener_thread_main(tx, running, raw_fd);

    println!("Shutting down supervisor...");
}