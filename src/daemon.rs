mod container;

use std::ffi::{c_void, CStr, CString};
use std::io::{IoSliceMut, Write};
use std::os::fd::{AsRawFd, RawFd};
use std::os::unix::net::UnixStream;
use std::process::exit;
use std::string::FromUtf8Error;
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::atomic::Ordering::{Relaxed, SeqCst};
use std::thread;
use std::thread::sleep;
use std::time::Duration;
use crate::{Cli, config};
use nix::sys::socket::{self, accept, AddressFamily, bind, ControlMessageOwned, listen, MsgFlags, RecvMsg, socket, SockFlag, SockType, UnixAddr};
use nix::{cmsg_space, Error};
use nix::errno::Errno;
use nix::errno::Errno::ENODATA;
use nix::libc::{iovec as IoVec, size_t};
use nix::unistd::{dup, execvp, fork, ForkResult, unlink};
use signal_hook::consts::TERM_SIGNALS;
use signal_hook::iterator::exfiltrator::SignalOnly;
use signal_hook::iterator::SignalsInfo;
use tokio::signal::unix::{signal, SignalKind};
use crate::daemon::container::{ContainerProcessState, State};

fn receive_fd(fd: RawFd) -> nix::Result<(RawFd, String)> {
    let mut buf = vec![0u8; 32768];
    let mut iov = [IoSliceMut::new(&mut buf)];
    let mut cmsg_space = cmsg_space!([RawFd; 2]);

    let msg: RecvMsg<'_, '_, UnixAddr> = socket::recvmsg(fd, &mut iov, Some(&mut cmsg_space), MsgFlags::empty())?;

    let state = read_string(&msg);

    if let Err(err) = state {
        return Err(Error::ENODATA);
    }

    let state = state.unwrap();

    for cmsg in msg.cmsgs() {
        if let ControlMessageOwned::ScmRights(fd) = cmsg {
            return Ok((fd[0], state));
        }
    }

    Err(Errno::ENODATA)
}

fn read_string(msg: &RecvMsg<UnixAddr>) -> Result<String, FromUtf8Error> {
    let bufsize = msg.bytes;
    let mut buf = vec![0u8; bufsize];

    for iov in msg.iovs() {
        buf.write(iov).expect("Works or abort");
    }

    return String::from_utf8(buf);
}

fn parse_state(state: String) -> serde_json::Result<ContainerProcessState> {
    let state = serde_json::from_str(state.as_str());

    return state;
}

fn fork_and_run(fd: RawFd, state: String) {
    //Parse container state
    let state = parse_state(state).unwrap_or_else(|err| {
        println!("error parsing state: {}", err);

        ContainerProcessState {
            version: "unknown".to_string(),
            fds: Default::default(),
            pid: 0,
            metadata: Default::default(),
            state: State {
                version: "".to_string(),
                id: Default::default(),
                status: "".to_string(),
                pid: 0,
                bundle: "".to_string(),
                annotations: Default::default(),
            },
        }
    });

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
                    let mut name = "unknown";

                    println!("{:?}", state);

                    if(state.state.annotations.contains_key("io.kubernetes.cri.sandbox-name")) {
                        name = state.state.annotations.get("io.kubernetes.cri.sandbox-name").unwrap();
                    }

                    let cmd = CString::new(exe_path.to_str().expect("unless someone fucked up the filesystem, this wont happen")).expect("will not fail");
                    let args = [cmd.clone(), CString::new(String::from("--fd=") + dup_fd.to_string().as_str()).unwrap(), CString::new(String::from("--pid=") + state.pid.to_string().as_str()).unwrap(), CString::new(String::from("--id=") + state.state.id.as_str()).expect("will not fail"), CString::new(String::from("--containerName=") + name).expect("will not fail either")];

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

    let mut signals = SignalsInfo::<SignalOnly>::new(TERM_SIGNALS).unwrap();

    let unix_addr = UnixAddr::new(config.socket_path.as_str()).unwrap();

    let result_socket = socket(AddressFamily::Unix, SockType::Stream, SockFlag::empty(), None);

    /*loop {
        println!("Connecting to socket...");

        let stream = UnixStream::connect(unix_addr.path().unwrap());
        match stream {
            Ok(stream) => {
                loop {
                    let seccompfd = receive_fd(&stream);

                    if let Err(errno) = seccompfd {
                        println!("Error: {}", errno);

                        break;
                    }

                    let seccompfd = seccompfd.unwrap();

                    println!("State {}", &seccompfd.1);

                    fork_and_run(seccompfd.0.as_raw_fd(), seccompfd.1);
                }
            }
            Err(errno) => {
                println!("Couldn't create socket, retrying in 5s. Error: {}", errno);
                sleep(Duration::from_secs(5));
            }
        }
    }*/

    //Spawn signal thread for systemd
    thread::spawn(move || async move {
        for signal in &mut signals {
            // Will print info about signal + where it comes from.
            match signal {
                _ => { // These are all the ones left
                    eprintln!("Terminating");
                    unlink(config.socket_path.as_str()).expect("Unlink failed");

                    exit(0);
                    break;
                }
            }
        }
    });

    //Maybe socket for CLI
    match result_socket {
        Ok(socket) => {
            let error = bind(socket.as_raw_fd(), &unix_addr).err();

            if let Some(errno) = error {
                println!("Couldn't bind socket, aborting. Error: {}", errno);
                panic!("error");
            }

            let error = listen(&socket, 1);

            if let Err(errno) = error {
                println!("Couldn't listen on socket, aborting. Error: {}", errno);
                panic!("error");
            }

            loop {
                let sock_result = accept(socket.as_raw_fd());

                if let Err(errno) = sock_result {
                    println!("Couldn't accept on socket, aborting. Error: {}", errno);
                    panic!("test");
                }

                match(sock_result) {
                    Ok(socket) => {
                        thread::spawn(move || {
                            println!("Receiving file descriptor...");
                            let seccompfd = receive_fd(socket);
                            println!("Received file descriptor");

                            if let Err(errno) = seccompfd {
                                println!("Error: {}", errno);

                                return;
                            }

                            let seccompfd = seccompfd.unwrap();

                            println!("State {}", &seccompfd.1);

                            fork_and_run(seccompfd.0.as_raw_fd(), seccompfd.1);
                        });
                    }
                    Err(errno) => {
                        println!("Error: {}", errno);
                    }
                }
            }
        }
        Err(errno) => {
            println!("Couldn't create socket, aborting. Error: {}", errno);
        }
    }
}