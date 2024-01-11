use std::os::fd::{RawFd};
use nix::libc;
use nix::libc::{SECCOMP_GET_NOTIF_SIZES, seccomp_notif_sizes, SYS_seccomp};
use crate::Cli;

pub fn supervisor_main(cmd: Cli) {
    println!("Running in supervisor mode!");

    let _raw_fd: RawFd = cmd.fd as RawFd;
    let mut notif_sizes: seccomp_notif_sizes = seccomp_notif_sizes {
        seccomp_notif: 0,
        seccomp_notif_resp: 0,
        seccomp_data: 0,
    };

    unsafe {
        let raw_notif_sizes = &mut notif_sizes as *mut seccomp_notif_sizes;

        let error = libc::syscall(SYS_seccomp, SECCOMP_GET_NOTIF_SIZES, 0, raw_notif_sizes);

        if(error != 0) {
            panic!("Error {} at SECCOMP_GET_NOTIF_SIZES syscall", error);
        }
    }

    println!("SECCOMP NOTIF SIZE: {}", notif_sizes.seccomp_notif);
    println!("SECCOMP NOTIF RESPONSE SIZE: {}", notif_sizes.seccomp_notif_resp);
    println!("SECCOMP NOTIF DATA SIZE: {}", notif_sizes.seccomp_data);
}