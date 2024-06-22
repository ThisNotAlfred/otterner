use std::{
    env,
    io::{BufRead, BufReader},
    path::PathBuf,
    process::{Command, Stdio},
};

use nix::{
    libc::SIGCHLD,
    mount::{mount, umount, MsFlags},
    sched::{clone, unshare, CloneFlags},
    unistd::chroot,
};

pub struct Container {
    stack_size: usize,
    memory_limit: usize,
    number_of_processes: usize,
    path_of_rootfs: PathBuf,
    command_to_run: String,
}

impl Container {
    pub fn new(
        stack_size: usize,
        memory_limit: usize,
        number_of_processes: usize,
        path_of_rootfs: PathBuf,
        command_to_run: String,
    ) -> Self {
        Self {
            stack_size,
            memory_limit,
            number_of_processes,
            path_of_rootfs,
            command_to_run,
        }
    }

    pub fn container_creator(&mut self) {
        let container_block = Box::new(|| self.container());
        match unsafe {
            clone(
                container_block,
                &mut vec![0; self.stack_size * 1024],
                CloneFlags::CLONE_NEWCGROUP
                    | CloneFlags::CLONE_NEWIPC
                    | CloneFlags::CLONE_NEWNET
                    | CloneFlags::CLONE_NEWNS
                    | CloneFlags::CLONE_NEWPID
                    | CloneFlags::CLONE_NEWUSER
                    | CloneFlags::CLONE_NEWUTS
                    | CloneFlags::CLONE_VFORK,
                Some(SIGCHLD),
            )
        } {
            Ok(_) => self.container_cleaner(),

            Err(_) => {
                eprintln!("failed to execute the container");
            }
        }
    }

    fn container(&self) -> isize {
        if let Err(_) = unshare(CloneFlags::CLONE_NEWNS | CloneFlags::CLONE_NEWPID) {
            eprintln!("couldn't unshare pid and namespaces");
            return -1;
        }

        if let Err(_) = env::set_current_dir(self.path_of_rootfs.as_path()) {
            eprintln!("couldn't find the designated directory");
            return -1;
        }

        if let Err(_) = chroot(".") {
            eprintln!("chroot failed");
            return -1;
        }

        if let Err(_) = mount(
            Some("proc"),
            "proc",
            Some("proc"),
            MsFlags::empty(),
            None::<&str>,
        ) {
            eprintln!("couln't mount the /proc");
            return -1;
        }

        let command = Command::new(self.command_to_run.to_owned())
            .stderr(Stdio::inherit())
            .stdout(Stdio::piped())
            .stdin(Stdio::inherit())
            .spawn();

        match command {
            Ok(o) => {
                let reader = BufReader::new(o.stdout.unwrap());

                reader
                    .lines()
                    .filter_map(|line| line.ok())
                    .for_each(|line| println!("{line}"));
            }
            Err(e) => {
                eprintln!("running command filed {}", e);
                return -1;
            }
        }

        umount("proc").unwrap();

        return 0;
    }

    fn container_cleaner(&self) {}
}
