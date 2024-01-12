use nix::libc::__u64;

pub struct SyscallEvent {
    pub pid: u32,
    pub syscall_no: u32,
    pub args: [__u64; 6],
}