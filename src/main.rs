use clap::Parser;
use otterner::{
    cli::{Cli, Commands},
    container::Container,
};

fn main() {
    let cli = Cli::parse();

    match cli.command.unwrap() {
        Commands::Run {
            memory_size,
            stack_size,
            pid_limit,
            rootfs,
            cmd,
        } => {
            let _ = Container::new(
                stack_size * 1024,
                memory_size * 1024 * 1204,
                pid_limit,
                rootfs,
                cmd,
            )
            .container_creator();
        }
    }
}
