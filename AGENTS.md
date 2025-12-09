# Agent Briefing

This repository ships a Rust CLI (`cryptfs-cli`) for managing encrypted repositories using GPG plus either gocryptfs (Linux) or cppcryptfs (Windows). This note is written for AI agents such as Codex or Antigravity.

## Mission
- Create new repositories by generating a random passphrase, encrypting it with GPG, and initializing the backend.
- Mount repositories by decrypting the passphrase and invoking the correct backend.
- Unmount mounted repositories cleanly.

## Required tools
- Rust stable toolchain (see `rust-toolchain.toml`).
- GPG with access to the recipient specified via `-u/--user`.
- Linux: `gpg`, `gocryptfs`, `fusermount`.
- Windows: `gpg`, `cppcryptfs.exe`, `cppcryptfsctl.exe` (drive letters supported).

## Directory contract
- Repositories contain `objects/`, `passphrase.gpg`, and `gocryptfs.conf`.
- Mount points are created automatically on Linux/macOS; on Windows you can pass a drive letter like `X:`.

## Operations
- `cryptfs-cli create -u <gpg_user> <repo_dir>`: fails if the target exists, writes secrets with 0600 permissions (Unix), and initializes the backend.
- `cryptfs-cli mount [-o opts] <repo_dir> <mount_point>`: validates layout, passes `-extpass "gpg --decrypt passphrase.gpg"` to gocryptfs, and calls cppcryptfs with the decrypted passphrase on Windows.
- `cryptfs-cli umount <mount_point>`: uses `fusermount -u` (Linux) or `cppcryptfs --unmount` (Windows).

## Toolchain habits
- Build with `cargo build --release`; format with `cargo fmt` and lint with `cargo clippy -- -D warnings`.
- Keep secret-bearing files at 0600 (Unix) and avoid writing plaintext passphrases to disk.

## Notes for agents
- Prefer absolute paths or normalized mount points; Windows drive letters are accepted as-is.
- Surface backend or GPG stderr when commands fail to aid debugging.
- Treat the binary names (`gocryptfs`, `cppcryptfs.exe`, `cppcryptfsctl.exe`) as requirements rather than bundled tooling.
