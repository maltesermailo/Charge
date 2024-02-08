mod container;

use std::ffi::{c_void, CString};
use std::io::IoSliceMut;
use std::os::fd::{AsRawFd, RawFd};
use std::os::unix::net::UnixStream;
use std::string::FromUtf8Error;
use crate::{Cli, config};
use nix::sys::socket::{self, accept, AddressFamily, bind, ControlMessageOwned, listen, MsgFlags, RecvMsg, SockFlag, SockType, UnixAddr};
use nix::sys::socket::socket;
use nix::{cmsg_space, Error};
use nix::errno::Errno;
use nix::libc::{iovec as IoVec, size_t};
use nix::unistd::{dup, execvp, fork, ForkResult};

fn receive_fd(unix_stream: &UnixStream) -> nix::Result<(RawFd, String)> {
    let fd = unix_stream.as_raw_fd();
    let mut buf = vec![0u8; 32768];
    let mut iov = [IoSliceMut::new(&mut buf)];
    let mut cmsg_space = cmsg_space!([RawFd; 2]);

    let msg: RecvMsg<'_, '_, UnixAddr> = socket::recvmsg(fd, &mut iov, Some(&mut cmsg_space), MsgFlags::empty())?;

    let state = parse_state(&msg);

    if let Err(err) = state {
        return Err(Error::ENODATA);
    }

    let state = state.unwrap();

    println!("State {}", &state);

    for cmsg in msg.cmsgs() {
        if let ControlMessageOwned::ScmRights(fd) = cmsg {
            return Ok((fd[0], state));
        }
    }

    Err(Errno::ENODATA)
}

fn parse_state(msg: &RecvMsg<UnixAddr>) -> Result<String, FromUtf8Error> {
    let bufsize = msg.bytes;
    let mut buf = vec![0u8; bufsize];

    if bufsize == 4 {
        return Ok(String::from("12345"));
    }

    return String::from_utf8(buf);
}

fn fork_and_run(fd: RawFd) {
    match std::env::current_exe() {
        Ok(exe_path) => unsafe {
            let dup_fd = dup(fd).unwrap();

            let fork_result = fork();

            if let Err(e) = fork_result {
                panic!("Couldn't fork process {}", e);
            }

            match fork_result.unwrap() {
                ForkResult::Parent { child, .. } => {
                    return;
                },
                ForkResult::Child => {
                    let cmd = CString::new(exe_path.to_str().expect("unless someone fucked up the filesystem, this wont happen")).expect("will not fail");
                    let args = [cmd.clone(), CString::new(String::from("--fd=") + dup_fd.to_string().as_str()).expect("will not fail")];

                    execvp(&cmd, &args).expect("execvp failed");
                }
            }
        }
        Err(e) => panic!("Failed to to get exe name {}", e),
    };
}

pub fn daemon_main(cmd: Cli) {
    let config = config::load_config();

    println!("Starting Charge daemon...");
    println!("Config socket path: {}", config.socket_path);

    let unix_addr = UnixAddr::new(config.socket_path.as_str()).unwrap();

    //let result_socket = socket(AddressFamily::Unix, SockType::Stream, SockFlag::empty(), None);

    let stream = UnixStream::connect(unix_addr.path().unwrap());
    match stream {
        Ok(stream) => {
            loop {
                let seccompfd = receive_fd(&stream);

                if let Err(errno) = seccompfd {
                    println!("ENODATA");

                    continue;
                }

                fork_and_run(seccompfd.unwrap().0.as_raw_fd());
            }
        }
        Err(errno) => {
            println!("Couldn't create socket, aborting. Error: {}", errno);
        }
    }

    //Maybe socket for CLI
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