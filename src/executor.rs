use std::ffi::CString;
use std::fs::{self, File};
use std::io::{self, Read, Write};
use std::mem;
use std::os::unix::io::FromRawFd;
use std::path::Path;

use libc::c_int;
use nix::errno::Errno;
use nix::fcntl::{self, FcntlArg, FdFlag};
use nix::mount::{self, MntFlags, MsFlags};
use nix::sched::{self, CloneFlags};
use nix::sys::signal::Signal;
use nix::sys::socket::{self, AddressFamily, SockFlag, SockType};
use nix::sys::wait;
use nix::unistd::{self, Gid, Pid, Uid};

use crate::container::Container;
use crate::invalid_command_error::InvalidCommandError;

const STACK_SIZE: usize = 1024 * 1024;

const UID_MAP_FILE_NAMES: [&str; 2] = ["uid_map", "gid_map"];
const USERNS_OFFSET: u32 = 0;
const USERNS_COUNT: u32 = 4294967295;

pub struct Executor;

impl Executor {
    pub fn new() -> Self {
        return Self;
    }

    pub fn execute(&self, container: &Container, command: &Vec<String>) -> anyhow::Result<()> {
        // Validate the command.
        if command.is_empty() {
            return Err(InvalidCommandError::Empty.into());
        }

        // Create a socketpair used to send messages between the parent and the child.
        let (mut parent_socket, mut child_socket) = Self::create_socketpair()?;

        // Clone a child process with all the relevant new namespaces.
        let mut clone_stack: Vec<u8> = vec![0; STACK_SIZE];
        let clone_flags = CloneFlags::CLONE_NEWNS
            | CloneFlags::CLONE_NEWCGROUP
            | CloneFlags::CLONE_NEWPID
            | CloneFlags::CLONE_NEWIPC
            | CloneFlags::CLONE_NEWNET
            | CloneFlags::CLONE_NEWUTS;
        let child_pid = sched::clone(
            Box::new(|| {
                if let Err(e) = Self::child(container, command, &mut child_socket) {
                    println!("error in child process: {}", e);
                    return 1;
                }

                0
            }),
            &mut clone_stack,
            clone_flags,
            Some(Signal::SIGCHLD as c_int),
        )?;

        // Defer waiting for the child process to exit.
        defer! { let _ = wait::waitpid(Some(child_pid), None); };

        // Close the child socket as it is not used by the parent.
        drop(child_socket);

        // Configure the child's user namespace.
        Self::handle_child_uid_map(child_pid, &mut parent_socket)?;

        Ok(())
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

    pub fn child(
        container: &Container,
        command: &Vec<String>,
        socket: &mut File,
    ) -> anyhow::Result<()> {
        // Set the container's host name.
        print!("=> setting host name... ");
        unistd::sethostname(&container.uuid)?;
        println!("done.");

        // Handle mounts.
        Self::mounts(&container)?;

        // Handle user namespaces and set UID / GID.
        Self::userns(socket)?;

        // Close the socket ahead of execvp.
        drop(socket);

        // Execute the requested command.
        let mut command_cstr = Vec::new();
        for arg in command {
            command_cstr.push(CString::new(&arg[..])?);
        }
        unistd::execvp(&command_cstr[0], &command_cstr)?;

        Err(Errno::last().into())
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

        // Flush stdout to sync with the child process's prints.
        io::stdout().flush().unwrap();

        // Send a success(0) result to the child.
        let result: u64 = 0;
        let result_bytes = result.to_le_bytes();
        socket.write_all(&result_bytes)?;

        Ok(())
    }

    fn mounts(container: &Container) -> anyhow::Result<()> {
        // Remount all mounts as private so that they will not be shared
        // with the parent process.
        print!("=> remounting everything with MS_PRIVATE... ");
        mount::mount::<str, str, str, str>(
            None,
            "/",
            None,
            MsFlags::MS_REC | MsFlags::MS_PRIVATE,
            None,
        )?;
        println!("remounted.");

        // Create a temporary directory to bind mount the image to.
        print!(
            "=> making a temp directory and bind mounting \"{}\" there... ",
            container.fs.path()?.as_os_str().to_str().unwrap()
        );
        let tmp_dir = tempfile::tempdir()?.into_path();

        // Bind mount the image to the temporary directory.
        mount::mount::<Path, Path, str, str>(
            Some(&container.fs.path()?),
            &tmp_dir,
            None,
            MsFlags::MS_BIND | MsFlags::MS_PRIVATE,
            None,
        )?;

        // Create a temporary directory inside the previously created temporary
        // directory, to which the old root directory will be mounted.
        let inner_tmp_dir = tempfile::tempdir_in(&tmp_dir)?;
        println!("done.");

        // Pivot the root directory to the temporary directory to which
        // the image has been mounted.
        print!("=> pivoting root... ");
        unistd::pivot_root(&tmp_dir, inner_tmp_dir.path())?;
        unistd::chroot("/")?;
        unistd::chdir("/")?;
        println!("done.");

        // Unmount the old root directory.
        let old_root_dir = Path::new("/").join(inner_tmp_dir.path().file_name().unwrap());
        print!("=> unmounting old root... ");
        mount::umount2(&old_root_dir, MntFlags::MNT_DETACH)?;
        fs::remove_dir(old_root_dir)?;
        println!("done.");

        // Mount /proc.
        print!("=> mounting /proc... ");
        mount::mount::<str, str, str, str>(
            Some("proc"),
            "/proc",
            Some("proc"),
            MsFlags::empty(),
            None,
        )?;
        println!("mounted.");

        Ok(())
    }

    fn userns(socket: &mut File) -> io::Result<()> {
        print!("=> trying a user namespace... ");

        // Flush stdout to sync wit the parent process's prints.
        io::stdout().flush().unwrap();

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

        // Set the process's group access list, UID and GID.
        println!("=> switching to uid 0 / gid 0...");
        let nix_uid = Uid::from_raw(0);
        let nix_gid = Gid::from_raw(0);
        unistd::setgroups(&[nix_gid])?;
        unistd::setresuid(nix_uid, nix_uid, nix_uid)?;
        unistd::setresgid(nix_gid, nix_gid, nix_gid)?;

        Ok(())
    }
}
