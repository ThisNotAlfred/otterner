use std::{
    env,
    fs::{create_dir_all, write},
    io::{self, BufRead, BufReader},
    path::PathBuf,
    process::{Command, Stdio},
};

use nix::{
    libc::SIGCHLD,
    mount::{mount, umount, MsFlags},
    sched::{clone, unshare, CloneFlags},
    unistd::{chroot, getpid},
};

pub struct Container {
    stack_size: usize,
    memory_limit: usize,
    number_of_processes: usize,
    path_of_rootfs: PathBuf,
    command_to_run: String,
    path_to_cgroup: String,
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
            path_to_cgroup: String::new(),
        }
    }

    pub fn container_creator(&mut self) -> Result<(), nix::Error> {
        let mut stack = vec![0; self.stack_size * 1024];
        let container_block = Box::new(|| self.container());

        match unsafe {
            clone(
                container_block,
                &mut stack,
                CloneFlags::CLONE_NEWIPC
                    | CloneFlags::CLONE_NEWNET
                    | CloneFlags::CLONE_NEWNS
                    | CloneFlags::CLONE_NEWPID
                    | CloneFlags::CLONE_NEWUSER
                    | CloneFlags::CLONE_NEWUTS
                    | CloneFlags::CLONE_VFORK,
                Some(SIGCHLD),
            )
        } {
            Ok(_) => {
                self.container_cleaner().unwrap();
                Ok(())
            }

            Err(_) => Err(nix::Error::ECANCELED),
        }
    }

    fn container(&mut self) -> isize {
        if let Err(err) = unshare(CloneFlags::CLONE_NEWNS) {
            eprintln!("couldn't unshare pid and namespaces: {err}");
            return -1;
        }

        if let Err(err) = env::set_current_dir(self.path_of_rootfs.as_path()) {
            eprintln!("couldn't find the designated directory: {err}");
            return -1;
        }

        if let Err(err) = chroot(".") {
            eprintln!("chroot failed: {err}");
            return -1;
        }

        // TODO: needs event handling
        if let Err(err) = self.setup_cgroups(getpid().as_raw() as usize) {
            eprintln!("cgroups failed: {err}");
            return -1;
        }

        if let Err(err) = mount(
            Some("proc"),
            "proc",
            Some("proc"),
            MsFlags::empty(),
            None::<&str>,
        ) {
            eprintln!("couln't mount the /proc: {err}");
            return -1;
        }

        let command = Command::new(self.command_to_run.to_owned())
            .stderr(Stdio::inherit())
            .stdout(Stdio::piped())
            .stdin(Stdio::inherit())
            .spawn();

        match command {
            Ok(output) => {
                let reader = BufReader::new(output.stdout.unwrap());

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

    fn setup_cgroups(&mut self, pid: usize) -> Result<(), io::Error> {
        self.path_to_cgroup = format!("/sys/fs/cgroup/otterner_{}", pid);
        create_dir_all(&self.path_to_cgroup)?;

        write(
            self.path_to_cgroup.to_string() + "/cgroup.procs",
            format!("{}", pid).as_bytes(),
        )?;

        write(
            self.path_to_cgroup.to_string() + "/pids.max",
            format!("{}", self.number_of_processes).as_bytes(),
        )?;

        write(
            self.path_to_cgroup.to_string() + "/memory.limit_in_bytes",
            format!("{}", self.memory_limit).as_bytes(),
        )?;

        Ok(())
    }

    fn container_cleaner(&self) -> Result<(), io::Error> {
        Ok(())
    }
}
