use std::os::fd::{AsRawFd, RawFd};
use std::os::unix::net::UnixStream;
use crate::{Cli, config};
use nix::sys::socket::{self, accept, AddressFamily, bind, ControlMessageOwned, listen, MsgFlags, SockFlag, SockType, UnixAddr};
use nix::sys::socket::socket;
use nix::cmsg_space;
use nix::errno::Errno;
use nix::libc::{iovec as IoVec};

fn receive_fd(unix_stream: &UnixStream) -> nix::Result<RawFd> {
    let mut fd = unix_stream.as_raw_fd();
    let mut buf = [0u8; 1024];
    let mut iov = [IoVec::from_mut_slice(&mut buf)];
    let mut cmsg_space = cmsg_space!([RawFd; 2]);

    let msg = socket::recvmsg(fd, &mut iov, Some(&mut cmsg_space), MsgFlags::empty())?;

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

    let unix_addr = UnixAddr::new(&config.socket_path)?;

    let result_socket = socket(AddressFamily::Unix, SockType::Stream, SockFlag::empty(), None);

    let stream = UnixStream::connect(unix_addr.path());
    match stream {
        Ok(stream) => {
            loop {
                let seccompfd = receive_fd(&stream);

                if let Err(errno) = seccompfd {
                    println!("ENODATA");

                    continue;
                }

                //START SUPERVISOR
            }
        }
        Err(errno) => {
            println!("Couldn't create socket, aborting. Error: {}", errno);
        }
    }

    /*match result_socket {
        Ok(socket) => {
            let error = bind(socket.as_raw_fd(), &unix_addr).err();

            if let Some(errno) = error {
                println!("Couldn't bind socket, aborting. Error: {}", errno);
            }

            let error = listen(&socket, 1);

            if let Some(errno) = error {
                println!("Couldn't listen on socket, aborting. Error: {}", errno);
            }

            loop {
                let sock_result = accept(socket.as_raw_fd());

                if let Some(errno) = sock_result {
                    println!("Couldn't listen on socket, aborting. Error: {}", errno);
                }

            }
        }
        Err(errno) => {
            println!("Couldn't create socket, aborting. Error: {}", errno);
        }
    }*/
}