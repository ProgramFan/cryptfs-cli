#[cfg(not(target_os = "windows"))]
mod linux;
#[cfg(target_os = "windows")]
mod windows;

#[cfg(target_os = "windows")]
use crate::process::run_with_output;
use anyhow::{bail, Context, Result};
use std::env;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};

pub fn create(user: &str, repo_dir: &Path) -> Result<()> {
    if user.trim().is_empty() {
        bail!("GPG user/email is required (-u/--user)");
    }

    let repo_dir = absolute_path(repo_dir)?;
    if repo_dir.exists() {
        bail!("directory '{}' already exists", repo_dir.display());
    }

    let objects_dir = repo_dir.join("objects");
    fs::create_dir_all(&objects_dir).with_context(|| {
        format!(
            "failed to create repository directories under '{}'",
            repo_dir.display()
        )
    })?;
    set_dir_mode(&repo_dir)?;
    set_dir_mode(&objects_dir)?;

    let passphrase_file = repo_dir.join("passphrase.gpg");
    generate_encrypted_passphrase(user, &passphrase_file)?;
    set_secret_mode(&passphrase_file)?;

    #[cfg(target_os = "windows")]
    {
        windows::init_repository(&repo_dir, &objects_dir, &passphrase_file)?;
    }

    #[cfg(not(target_os = "windows"))]
    {
        linux::init_repository(&repo_dir, &objects_dir, &passphrase_file)?;
    }

    Ok(())
}

pub fn mount(repo_dir: &Path, mount_point: &Path, options: Option<&str>) -> Result<()> {
    let repo_dir = absolute_path(repo_dir)?;
    let passphrase_file = repo_dir.join("passphrase.gpg");
    let cipher_dir = repo_dir.join("objects");

    if !passphrase_file.is_file() || !cipher_dir.is_dir() {
        bail!(
            "repository layout is invalid under '{}': expected passphrase.gpg and objects/",
            repo_dir.display()
        );
    }

    ensure_mount_point_exists(mount_point)?;

    #[cfg(target_os = "windows")]
    {
        windows::mount_repository(&cipher_dir, &passphrase_file, mount_point, options, &repo_dir)?;
    }

    #[cfg(not(target_os = "windows"))]
    {
        linux::mount_repository(&cipher_dir, &passphrase_file, mount_point, options, &repo_dir)?;
    }

    Ok(())
}

pub fn umount(mount_point: &Path) -> Result<()> {
    #[cfg(target_os = "windows")]
    {
        windows::umount_repository(mount_point)?;
    }

    #[cfg(not(target_os = "windows"))]
    {
        linux::umount_repository(mount_point)?;
    }

    Ok(())
}

pub fn normalize_mount_point(input: &str) -> Result<PathBuf> {
    let candidate = PathBuf::from(input);

    if cfg!(target_os = "windows") {
        if is_windows_drive_letter(input) || candidate.is_absolute() {
            return Ok(candidate);
        }
    } else if candidate.is_absolute() {
        return Ok(candidate);
    }

    Ok(env::current_dir()?.join(candidate))
}

#[cfg(target_os = "windows")]
fn is_windows_drive_letter(input: &str) -> bool {
    let bytes = input.as_bytes();
    bytes.len() == 2
        && (bytes[0].is_ascii_alphabetic())
        && (bytes[1] == b':')
}

#[cfg(not(target_os = "windows"))]
fn is_windows_drive_letter(_: &str) -> bool {
    false
}

fn ensure_mount_point_exists(mount_point: &Path) -> Result<()> {
    if cfg!(target_os = "windows") && is_windows_drive_letter(&mount_point.to_string_lossy()) {
        return Ok(()); // Drive letters do not need creation.
    }

    if mount_point.is_dir() {
        return Ok(());
    }

    fs::create_dir_all(mount_point).with_context(|| {
        format!(
            "failed to create mount point '{}'",
            mount_point.display()
        )
    })?;

    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        let perms = fs::Permissions::from_mode(0o700);
        fs::set_permissions(mount_point, perms)
            .context("unable to set mount point permissions to 700")?;
    }

    Ok(())
}

fn absolute_path(path: &Path) -> Result<PathBuf> {
    if path.is_absolute() {
        Ok(path.to_path_buf())
    } else {
        Ok(env::current_dir()?.join(path))
    }
}

fn generate_encrypted_passphrase(user: &str, passphrase_file: &Path) -> Result<()> {
    println!("Generating and encrypting passphrase with GPG...");

    let mut random_child = Command::new("gpg")
        .args(["--gen-random", "--armor", "0", "64"])
        .stdout(Stdio::piped())
        .spawn()
        .context("failed to start gpg --gen-random")?;

    let random_stdout = random_child
        .stdout
        .take()
        .context("failed to capture gpg --gen-random stdout")?;

    let encrypt_child = Command::new("gpg")
        .args(["--encrypt", "--sign", "-r", user, "-o"])
        .arg(passphrase_file)
        .stdin(Stdio::from(random_stdout))
        .stderr(Stdio::piped())
        .spawn()
        .context("failed to start gpg --encrypt pipeline")?;

    let encrypt_output = encrypt_child
        .wait_with_output()
        .context("failed waiting for gpg --encrypt")?;
    let random_status = random_child
        .wait()
        .context("failed waiting for gpg --gen-random")?;

    if !random_status.success() {
        bail!("gpg --gen-random failed with status {}", random_status);
    }

    if !encrypt_output.status.success() {
        let stderr = String::from_utf8_lossy(&encrypt_output.stderr);
        bail!("gpg failed to encrypt passphrase: {}", stderr.trim());
    }

    Ok(())
}

#[cfg(target_os = "windows")]
fn decrypt_passphrase(passphrase_file: &Path) -> Result<String> {
    let mut cmd = Command::new("gpg");
    cmd.arg("--decrypt").arg(passphrase_file);
    let output = run_with_output(&mut cmd)?;
    let passphrase = String::from_utf8(output.stdout).context("passphrase is not valid UTF-8")?;
    Ok(passphrase)
}

#[cfg(unix)]
fn set_secret_mode(path: &Path) -> Result<()> {
    use std::os::unix::fs::PermissionsExt;
    let perms = fs::Permissions::from_mode(0o600);
    fs::set_permissions(path, perms)
        .with_context(|| format!("failed to set permissions on '{}'", path.display()))?;
    Ok(())
}

#[cfg(not(unix))]
fn set_secret_mode(_: &Path) -> Result<()> {
    Ok(())
}

#[cfg(unix)]
fn set_dir_mode(path: &Path) -> Result<()> {
    use std::os::unix::fs::PermissionsExt;
    let perms = fs::Permissions::from_mode(0o700);
    fs::set_permissions(path, perms)
        .with_context(|| format!("failed to set directory permissions on '{}'", path.display()))?;
    Ok(())
}

#[cfg(not(unix))]
fn set_dir_mode(_: &Path) -> Result<()> {
    Ok(())
}
