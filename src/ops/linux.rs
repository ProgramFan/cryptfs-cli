use super::set_secret_mode;
use crate::process::run_with_output;
use anyhow::{Context, Result};
use std::path::Path;
use std::process::Command;

pub fn init_repository(repo_dir: &Path, objects_dir: &Path, passphrase_file: &Path) -> Result<()> {
    println!("Initializing gocryptfs...");

    let config_path = repo_dir.join("gocryptfs.conf");
    let config_str = config_path
        .to_str()
        .context("repository path is not valid UTF-8")?;

    let mut cmd = Command::new("gocryptfs");
    cmd.args([
        "-init",
        "--deterministic-names",
        "--config",
        config_str,
    ]);
    cmd.arg("-extpass");
    cmd.arg(format!(
        "gpg --decrypt \"{}\"",
        passphrase_file.display()
    ));
    cmd.arg(objects_dir);

    run_with_output(&mut cmd).context("gocryptfs init failed")?;
    set_secret_mode(&config_path)?;

    println!("Repository created successfully at '{}'", repo_dir.display());
    Ok(())
}

pub fn mount_repository(
    cipher_dir: &Path,
    passphrase_file: &Path,
    mount_point: &Path,
    options: Option<&str>,
    repo_dir: &Path,
) -> Result<()> {
    println!("Mounting gocryptfs...");

    let config_path = repo_dir.join("gocryptfs.conf");
    let config_str = config_path
        .to_str()
        .context("repository path is not valid UTF-8")?;

    let mut cmd = Command::new("gocryptfs");
    cmd.arg("--config").arg(config_str);
    cmd.arg("-extpass");
    cmd.arg(format!(
        "gpg --decrypt \"{}\"",
        passphrase_file.display()
    ));

    if let Some(opts) = options {
        cmd.arg("-o").arg(opts);
    }

    cmd.arg(cipher_dir);
    cmd.arg(mount_point);

    run_with_output(&mut cmd).context("gocryptfs mount failed")?;

    println!(
        "Mounted '{}' at '{}'",
        cipher_dir.display(),
        mount_point.display()
    );
    Ok(())
}

pub fn umount_repository(mount_point: &Path) -> Result<()> {
    println!("Unmounting '{}'...", mount_point.display());

    let mount_str = mount_point
        .to_str()
        .context("mount point is not valid UTF-8")?;

    let mut cmd = Command::new("fusermount");
    cmd.args(["-u", mount_str]);
    run_with_output(&mut cmd).context("fusermount failed")?;

    println!("Unmounted '{}' successfully.", mount_point.display());
    Ok(())
}
