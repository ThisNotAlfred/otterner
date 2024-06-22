use std::{
    io::{BufRead, BufReader}, os::unix::fs::chroot, process::{Command, Stdio}
};

use nix::{
    libc::{getpid, sethostname}, mount::{mount, MsFlags}, sched::{clone, unshare, CloneFlags}
};

fn main() {
    let mut stack: Vec<u8> = vec![0; 1024 * 1024];

    let cbk = Box::new(|| child());

    match unsafe {
        clone(
            cbk,
            &mut stack,
            CloneFlags::CLONE_NEWUSER
                | CloneFlags::CLONE_NEWUTS
                | CloneFlags::CLONE_NEWPID
                | CloneFlags::CLONE_VFORK,
            None,
        )
    } {
        Ok(_) => {
            // clean-ups after child happen here!
        }
        Err(_) => {
            eprint!("child failed");
        }
    }
}

fn child() -> isize {
    unshare(CloneFlags::CLONE_NEWPID | CloneFlags::CLONE_NEWNS).unwrap();
    std::env::set_current_dir("ubuntu-fs").unwrap();
    chroot(".").unwrap();
    mount(Some("proc"), "proc", Some("proc"), MsFlags::empty(), Some("")).unwrap();
    unsafe { sethostname("otterner".as_ptr() as *const i8, 8) };

    println!("{}", unsafe { getpid() });

    let console = Command::new("/bin/bash")
        .stdin(Stdio::inherit())
        .stdout(Stdio::piped())
        .spawn()
        .unwrap();

    let reader = BufReader::new(console.stdout.unwrap());

    reader
        .lines()
        .filter_map(|line| line.ok())
        .for_each(|line| println!("{line}"));

    return 0;
}
