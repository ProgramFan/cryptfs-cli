use clap::{Parser, Subcommand};
use std::path::PathBuf;

#[derive(Parser)]
#[command(name = "cryptfs-cli")]
#[command(about = "Manage encrypted repositories with GPG + gocryptfs/cppcryptfs")]
#[command(version)]
pub struct Cli {
    #[command(subcommand)]
    pub command: Commands,
}

#[derive(Subcommand)]
pub enum Commands {
    /// Create a new encrypted repository
    Create {
        /// GPG user/email for encryption (required)
        #[arg(short, long)]
        user: String,
        /// Target repository directory
        repo_dir: PathBuf,
    },
    /// Mount an encrypted repository
    Mount {
        /// Options passed through to the cryptfs backend
        #[arg(short, long)]
        options: Option<String>,
        /// Repository directory (containing passphrase.gpg + objects)
        repo_dir: PathBuf,
        /// Mount point or drive letter (Windows)
        mount_point: String,
    },
    /// Unmount a repository
    Umount {
        /// Mount point or drive letter (Windows)
        mount_point: String,
    },
}
