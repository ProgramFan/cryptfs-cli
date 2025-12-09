mod cli;
mod ops;
mod process;

use anyhow::Result;
use clap::Parser;
use cli::{Cli, Commands};

fn main() {
    if let Err(err) = run() {
        eprintln!("Error: {err}");
        for cause in err.chain().skip(1) {
            eprintln!("Caused by: {cause}");
        }
        std::process::exit(1);
    }
}

fn run() -> Result<()> {
    let cli = Cli::parse();

    match cli.command {
        Commands::Create { user, repo_dir } => ops::create(&user, &repo_dir)?,
        Commands::Mount {
            options,
            repo_dir,
            mount_point,
        } => {
            let normalized_mount = ops::normalize_mount_point(&mount_point)?;
            ops::mount(&repo_dir, &normalized_mount, options.as_deref())?;
        }
        Commands::Umount { mount_point } => {
            let normalized_mount = ops::normalize_mount_point(&mount_point)?;
            ops::umount(&normalized_mount)?;
        }
    }

    Ok(())
}
