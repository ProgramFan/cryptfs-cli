use super::{decrypt_passphrase, set_secret_mode};
use crate::process::run_with_output;
use anyhow::{bail, Context, Result};
use std::fs;
use std::io::Write;
use std::path::Path;
use std::process::{Command, Stdio};

pub fn init_repository(repo_dir: &Path, objects_dir: &Path, passphrase_file: &Path) -> Result<()> {
    println!("Initializing cppcryptfs...");

    let passphrase = decrypt_passphrase(passphrase_file)?;
    let volume_name = repo_dir
        .file_name()
        .and_then(|name| name.to_str())
        .unwrap_or("cryptfs");

    let mut cmd = Command::new("cppcryptfsctl.exe");
    cmd.arg(format!("--init={}", objects_dir.display()));
    cmd.arg(format!("--volumename={}", volume_name));
    cmd.arg("--deterministicnames");
    cmd.stdin(Stdio::piped());
    cmd.stderr(Stdio::piped());

    let mut child = cmd.spawn().context("failed to start cppcryptfsctl.exe")?;
    if let Some(stdin) = child.stdin.as_mut() {
        stdin
            .write_all(passphrase.trim_end().as_bytes())
            .context("failed to send passphrase to cppcryptfsctl.exe")?;
    }

    let output = child
        .wait_with_output()
        .context("cppcryptfsctl.exe did not exit cleanly")?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        bail!("cppcryptfsctl.exe failed: {}", stderr.trim());
    }

    let src_conf = objects_dir.join("gocryptfs.conf");
    let dst_conf = repo_dir.join("gocryptfs.conf");
    fs::rename(&src_conf, &dst_conf).context("failed to move gocryptfs.conf")?;
    set_secret_mode(&dst_conf)?;

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
    println!("Decrypting passphrase...");
    let passphrase = decrypt_passphrase(passphrase_file)?;

    let mut cmd = Command::new("cppcryptfs.exe");
    cmd.arg(format!("--mount={}", cipher_dir.display()));
    cmd.arg(format!(
        "--drive={}",
        mount_point
            .to_str()
            .context("mount point is not valid UTF-8")?
    ));
    cmd.arg(format!("--password={}", passphrase.trim_end()));
    cmd.arg(format!(
        "--config={}",
        repo_dir.join("gocryptfs.conf").display()
    ));
    cmd.args(["-t", "-x"]);

    if let Some(opts) = options {
        println!("Note: mount options are ignored on Windows (received: {opts})");
    }

    run_with_output(&mut cmd).context("cppcryptfs.exe mount failed")?;

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

    let mut cmd = Command::new("cppcryptfs.exe");
    cmd.arg(format!("--unmount={}", mount_str));

    run_with_output(&mut cmd).context("cppcryptfs.exe unmount failed")?;

    println!("Unmounted '{}' successfully.", mount_point.display());
    Ok(())
}
