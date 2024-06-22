use std::path::PathBuf;

use clap::{Parser, Subcommand};

#[derive(Parser)]
#[command(name = "Otterner")]
#[command(about= "minimal container", long_about= None)]
#[command(version, about, long_about = None)]
pub struct Cli {
    #[command(subcommand)]
    pub command: Option<Commands>,
}

#[derive(Subcommand)]
pub enum Commands {
    /// runs container
    Run {
        #[arg(short, long)]
        /// specifies the amount of memory that container uses
        memory_size: usize,

        #[arg(short, long)]
        /// pid limit in a container
        pid_limit: usize,

        #[arg(short, long)]
        /// specifies the amount of memory that container stack uses
        stack_size: usize,

        #[arg(short, long)]
        /// path to rootfs
        rootfs: PathBuf,

        #[arg(short, long)]
        /// command to be run by the container
        cmd: String,
    },
}
