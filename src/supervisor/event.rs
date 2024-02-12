use nix::libc::__u64;
use serde::Serialize;

#[derive(Serialize)]
pub struct SyscallEvent {
    pub pid: u32,
    pub syscall_no: u32,
    pub args: [__u64; 6],
}