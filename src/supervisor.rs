mod log_writer;
mod listener;
mod event;

use std::os::fd::{RawFd};
use std::process::exit;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, mpsc};
use std::thread;
use std::thread::sleep;
use std::time::Duration;
use nix::libc;
use nix::libc::{c_int, pid_t, SECCOMP_GET_NOTIF_SIZES, seccomp_notif_sizes, SYS_seccomp};
use nix::sys::signal::kill;
use nix::unistd::Pid;
use crate::Cli;
use crate::supervisor::listener::listener_thread_main;
use crate::supervisor::log_writer::log_write_thread_main;
use tokio::signal::unix::{signal, SignalKind};
use k8s_openapi::api::core::v1::Pod;
use kube::{Api, Client};

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
    let running_container = Arc::clone(&running);

    let pid = cmd.pid;
    let id = cmd.id;
    let containerName = cmd.container_name;
    let containerNameClone = containerName.clone();

    //Spawn log writer thread
    thread::spawn(move || {
        log_write_thread_main(rx, pid, id, containerName, running_log_write);
    });

    //Spawn signal thread for systemd
    thread::spawn(move || async move {
        loop {
            stream.recv().await;

            running_signal.store(false, Ordering::SeqCst);
        }
    });

    //Spawn container check thread
    thread::spawn(move || async move {
        println!("Starting container check thread.");

        if(!containerNameClone.eq_ignore_ascii_case("unknown")) {
            let client = Client::try_default().await.unwrap();

            let api: Api<Pod> = Api::default_namespaced(client);
            loop {
                if(!running_container.load(Ordering::SeqCst)) {
                    exit(0);
                }

                let result = api.get_status(containerNameClone.as_str()).await;

                match result {
                    Ok(pod) => {
                        if(pod.status.is_some()) {
                            //Pod is running
                            sleep(Duration::from_secs(1));
                            continue;
                        }
                        running_container.store(false, Ordering::SeqCst);
                    },
                    Err(e) => {
                        running_container.store(false, Ordering::SeqCst);
                    }
                }
            }
        } else {
            loop {
                if(!running_container.load(Ordering::SeqCst)) {
                    exit(0);
                }

                //Check whether PID is still running
                let pidResult = kill(Pid::from_raw(pid as pid_t), None);

                if(pidResult.is_err()) {
                    running_container.store(false, Ordering::SeqCst);
                }

                sleep(Duration::from_millis(50));
            }
        }
    });

    unsafe { listener_thread_main(tx, running, raw_fd); }

    println!("Shutting down supervisor...");
}