use std::fmt::Debug;
use std::mem::MaybeUninit;
use std::os::fd::{BorrowedFd, RawFd};
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::mpsc::Sender;
use std::time::SystemTime;
use nix::errno::Errno;
use nix::{fcntl, ioctl_read, ioctl_readwrite, ioctl_write_ptr, libc, request_code_read, request_code_readwrite};
use nix::fcntl::{F_GETFD, fcntl};
use nix::poll::{poll, PollFd, PollFlags};
use nix::libc::{ioctl, SECCOMP_GET_NOTIF_SIZES, seccomp_notif_resp, SYS_seccomp};
use crate::supervisor::event::SyscallEvent;
use crate::supervisor::seccomp_notif;

const SECCOMP_USER_FLAG_CONTINUE: u32 = 1;
const SECCOMP_IOCTL_NOTIF_RECV: u8 = 0;
const SECCOMP_IOCTL_NOTIF_SEND: u8 = 1;

//SECCOMP_IOCTL_NOTIF_RECV
ioctl_readwrite!(receive_notif, b'!', SECCOMP_IOCTL_NOTIF_RECV, seccomp_notif);
//SECCOMP_IOCTL_NOTIF_SEND
ioctl_readwrite!(send_notif, b'!', SECCOMP_IOCTL_NOTIF_SEND, seccomp_notif_resp);

pub unsafe fn listener_thread_main(tx: Sender<SyscallEvent>, running: Arc<AtomicBool>, raw_fd: RawFd) {
    let fd = BorrowedFd::borrow_raw(raw_fd);
    let pollfd = PollFd::new(&fd, PollFlags::all());
    let mut pollfds = [pollfd];

    while running.load(Ordering::SeqCst) {
        let mut seccomp_notif_uninit: MaybeUninit<seccomp_notif> = unsafe { MaybeUninit::zeroed() };

        unsafe {
            let fcntl_result = fcntl(raw_fd, F_GETFD);

            if let Err(e) = fcntl_result {
                println!("Error while reading file descriptor: {}", e);
                return;
            }

            let poll_result = poll(&mut pollfds, 50);

            if let Err(e) = poll_result {
                println!("Error while polling file descriptor: {}", e);
                if(e == Errno::UnknownErrno) {
                    continue;
                }

                return;
            }

            if let Ok(results) = poll_result {
                if results > 0 {
                    //println!("Poll returned: {:?}", pollfd.revents().unwrap())
                }
            }

            let result = receive_notif(raw_fd, seccomp_notif_uninit.as_mut_ptr()).unwrap_or_else(|error| {
                println!("Couldn't read notif with error {}", error);

                if error == Errno::ENOTTY {
                    panic!("no tty");
                }

                if error == Errno::ENOENT {
                    return -1;
                }

                panic!("ERROR");
            });

            if result == -1 {
                println!("Test");
                continue;
            }

            let date = SystemTime::now();

            //println!("[{:?}] received notif", date);

            let seccomp_notif = seccomp_notif_uninit.assume_init();

            let event = SyscallEvent {
                pid: seccomp_notif.pid,
                syscall_no: seccomp_notif.data.nr as u32,
                args: seccomp_notif.data.args.clone()
            };

            tx.send(event).unwrap_or_else(|err| {
               println!("Error while sending over channel {}", err);
            });

            //Allow syscall
            
            let mut seccomp_resp: seccomp_notif_resp = seccomp_notif_resp {
                id: seccomp_notif.id,
                val: 0,
                error: 0,
                flags: SECCOMP_USER_FLAG_CONTINUE,
            };

            _ = send_notif(raw_fd, &mut seccomp_resp as *mut seccomp_notif_resp).unwrap_or_else(|error| {
                println!("Couldn't send notif with error {}", error);

                return -1;
            });

            //println!("Accepted notif");
        }
    }
}