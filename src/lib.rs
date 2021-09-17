#[macro_use(defer)]
extern crate scopeguard;

use std::error::Error;
use std::fs;
use std::fs::File;
use std::io;
use std::io::{Read, Write};
use std::mem;
use std::os::unix::io::FromRawFd;
use std::time::{SystemTime, UNIX_EPOCH};

use libc::c_int;
use nix::fcntl;
use nix::fcntl::{FcntlArg, FdFlag};
use nix::sched;
use nix::sched::CloneFlags;
use nix::sys::signal::Signal;
use nix::sys::socket;
use nix::sys::socket::{AddressFamily, SockFlag, SockType};
use nix::sys::wait;
use nix::unistd::Pid;

mod opts;
pub use opts::Opts;

const STACK_SIZE: usize = 1024 * 1024;

const MAJOR: [&str; 22] = [
    "fool",
    "magician",
    "high-priestess",
    "empress",
    "emperor",
    "hierophant",
    "lovers",
    "chariot",
    "strength",
    "hermit",
    "wheel",
    "justice",
    "hanged-man",
    "death",
    "temperance",
    "devil",
    "tower",
    "star",
    "moon",
    "sun",
    "judgment",
    "world",
];
const MINOR: [&str; 14] = [
    "ace", "two", "three", "four", "five", "six", "seven", "eight", "nine", "ten", "page",
    "knight", "queen", "king",
];
const SUITS: [&str; 4] = ["swords", "wands", "pentacles", "cups"];

const UID_MAP_FILE_NAMES: [&str; 2] = ["uid_map", "gid_map"];
const USERNS_OFFSET: u32 = 10000;
const USERNS_COUNT: u32 = 2000;

pub fn run(opts: Opts) -> Result<(), Box<dyn Error>> {
    // Choose the container's hostname.
    let hostname = choose_hostname();

    // Create a socketpair used to send messages from the parent to the child.
    let (mut parent_socket, mut child_socket) = create_socketpair()?;

    // Clone a child process with all the relevant new namespaces.
    let mut clone_stack: Vec<u8> = vec![0; STACK_SIZE];
    let clone_flags = CloneFlags::CLONE_NEWNS
        | CloneFlags::CLONE_NEWCGROUP
        | CloneFlags::CLONE_NEWPID
        | CloneFlags::CLONE_NEWIPC
        | CloneFlags::CLONE_NEWNET
        | CloneFlags::CLONE_NEWUTS;
    let child_pid = sched::clone(
        Box::new(|| child(&mut child_socket)),
        &mut clone_stack,
        clone_flags,
        Some(Signal::SIGCHLD as c_int),
    )?;

    // Defer waiting for the child process to exit.
    defer! { let _ = wait::waitpid(Some(child_pid), None); };

    // // Close the child socket as it is not required from the parent.
    // let _ = unistd::close(child_socket);
    // child_socket = -1;

    // Configure the child's user namespace.
    handle_child_uid_map(child_pid, &mut parent_socket)?;

    Ok(())
}

fn choose_hostname() -> String {
    // Retrieve the current time since the epoch in nanoseconds.
    let now = SystemTime::now();
    let duration_since_epoch = now.duration_since(UNIX_EPOCH).expect("Time went backwards");
    let seconds = duration_since_epoch.as_secs();
    let seconds_hex = format!("{:x}", seconds);
    let nanos = duration_since_epoch.as_nanos();

    // Calculate the IX value.
    let mut ix = (nanos as usize) % 78;

    // Pick a hostname.
    if ix < MAJOR.len() {
        return format!("{:0>5.5}-{}", seconds_hex, MAJOR[ix]);
    }

    ix -= MAJOR.len();
    format!(
        "{:0>5.5}c-{}-of-{}",
        seconds_hex,
        MINOR[ix % MINOR.len()],
        SUITS[ix / MINOR.len()]
    )
}

fn create_socketpair() -> nix::Result<(File, File)> {
    // Create the socketpair.
    let sockets = socket::socketpair(
        AddressFamily::Unix,
        SockType::SeqPacket,
        None,
        SockFlag::empty(),
    )?;

    // Set the first socket in the pair to close on exec.
    fcntl::fcntl(sockets.0, FcntlArg::F_SETFD(FdFlag::FD_CLOEXEC))?;

    let socket_0 = unsafe { File::from_raw_fd(sockets.0) };
    let socket_1 = unsafe { File::from_raw_fd(sockets.1) };

    Ok((socket_0, socket_1))
}

fn handle_child_uid_map(child_pid: Pid, socket: &mut File) -> io::Result<()> {
    // Read the has_userns value from the child process.
    let mut has_userns_bytes: [u8; mem::size_of::<u64>()] = [0; mem::size_of::<u64>()];
    socket.read(&mut has_userns_bytes)?;
    let has_userns = 0 != u64::from_le_bytes(has_userns_bytes);

    if has_userns {
        // The host OS supports user namespaces.
        // Set the container's user namespace offset and count.
        for file_name in UID_MAP_FILE_NAMES {
            let path = format!("/proc/{}/{}", child_pid.as_raw(), file_name);
            let contents = format!("0 {} {}\n", USERNS_OFFSET, USERNS_COUNT);
            print!("writing {}... ", path);
            fs::write(path, contents)?;
        }
    }

    // Send a success(0) result to the child.
    let result: u64 = 0;
    let result_bytes = result.to_le_bytes();
    socket.write_all(&result_bytes)?;

    Ok(())
}

fn child(socket: &mut File) -> isize {
    if let Err(e) = userns(socket) {
        println!("userns failed, error: {}", e);
        return 1;
    }

    0
}

fn userns(socket: &mut File) -> io::Result<()> {
    print!("=> trying a user namespace... ");

    // Check if the host OS supports user namespaces.
    let mut has_userns = true;
    if let Err(_) = sched::unshare(CloneFlags::CLONE_NEWUSER) {
        has_userns = false;
    }

    // Write the check's result to the parent process for further handling.
    let has_userns_bytes = (has_userns as u64).to_le_bytes();
    socket.write_all(&has_userns_bytes)?;

    // Read the parent's result user namespace handling result.
    let mut result_bytes: [u8; mem::size_of::<u64>()] = [0; mem::size_of::<u64>()];
    socket.read(&mut result_bytes)?;

    if has_userns {
        println!("done.");
    } else {
        println!("unsupported? continuing.");
    }

    Ok(())
}
