use crate::{Cli, config};
use nix::sys::socket::{self, ControlMessageOwned, MsgFlags, UnixAddr};
use std::os::unix::net::UnixStream;

fn receive_fd(stream: UnixStream) -> nix::Result<RawFd> {
    let fd = stream.as_raw_fd();
    let mut buf = [0u8; 100];
    let iov = [IoVec::from_mut_slice(&mut buf)];

    let msg = socket::recvmsg(fd, &iov, Some(100), MsgFlags::empty())?;

    for cmsg in msg.cmsgs() {
        if let ControlMessageOwned::ScmRights(fd) = cmsg {
            return Ok(fd[0]);
        }
    }

    Err(nix::Error::Sys(Errno::ENODATA))
}

pub fn daemon_main(cmd: Cli) {
    let config = config::load_config();

    println!("Starting Charge daemon...");
    println!("Config socket path: {}", config.socket_path);

    let unixAddr = UnixAddr::new(config.socket_path)?;
}