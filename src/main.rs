use std::{
    io::{BufRead, BufReader},
    process::{Command, Stdio},
};

use nix::sched::{clone, CloneFlags};

fn main() {
    let mut stack: Vec<u8> = vec![0; 1024 * 1024];

    let cbk = Box::new(|| child());

    match unsafe {
        clone(
            cbk,
            &mut stack,
            CloneFlags::CLONE_NEWCGROUP
                | CloneFlags::CLONE_NEWIPC
                | CloneFlags::CLONE_NEWUSER
                | CloneFlags::CLONE_NEWPID
                | CloneFlags::CLONE_NEWNS
                | CloneFlags::CLONE_VFORK,
            None,
        )
    } {
        Ok(_) => {
            // clean-ups after child happen here!
            println!("this is parent");
        }
        Err(_) => {
            eprint!("child failed");
        }
    }
}

fn child() -> isize {
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
