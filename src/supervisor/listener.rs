use std::os::fd::RawFd;
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::mpsc::Sender;
use nix::errno::Errno;
use nix::{ioctl_read, ioctl_write_ptr};
use nix::libc::{seccomp_data, seccomp_notif, seccomp_notif_resp, seccomp_notif_sizes};
use crate::supervisor::event::SyscallEvent;

const SECCOMP_USER_FLAG_CONTINUE: u32 = 1;
const SECCOMP_IOCTL_NOTIF_RECV: u8 = 0;
const SECCOMP_IOCTL_NOTIF_SEND: u8 = 1;

//SECCOMP_IOCTL_NOTIF_RECV
ioctl_read!(receive_notif, b'!', SECCOMP_IOCTL_NOTIF_RECV, seccomp_notif);
//SECCOMP_IOCTL_NOTIF_SEND
ioctl_write_ptr!(send_notif, b'!', SECCOMP_IOCTL_NOTIF_SEND, seccomp_notif_resp);

pub fn listener_thread_main(tx: Sender<SyscallEvent>, running: Arc<AtomicBool>, raw_fd: RawFd) {
    while running.load(Ordering::SeqCst) {
        let mut seccomp_notif = seccomp_notif {
          ..Default::default()
        };

        unsafe {
            let result = receive_notif(raw_fd, &mut seccomp_notif as *mut seccomp_notif).unwrap_or_else(|error| {
                println!("Couldn't read notif with error {}", error);

                return -1;
            });

            if(result == -1) {
                continue;
            }

            let event = SyscallEvent {
                pid: seccomp_notif.pid,
                syscall_no: seccomp_notif.data.nr as u32,
                args: seccomp_notif.data.args.clone()
            };

            tx.send(event).unwrap_or_else(|err| {
               println!("Error while sending over channel {}", err);
            });

            //Allow syscall
            
            let seccomp_resp: seccomp_notif_resp = seccomp_notif_resp {
                id: seccomp_notif.id,
                val: 0,
                error: 0,
                flags: SECCOMP_USER_FLAG_CONTINUE,
            };

            _ = send_notif(raw_fd, &seccomp_resp as *const seccomp_notif_resp).unwrap_or_else(|error| {
                println!("Couldn't send notif with error {}", error);

                return -1;
            });
        }
    }
}