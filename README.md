# cryptfs-cli

`cryptfs-cli` is a Rust CLI for creating, mounting, and unmounting encrypted repositories. It generates a random passphrase, encrypts it with **GPG**, and passes it to **gocryptfs** (Linux) or **cppcryptfs** (Windows) for the actual filesystem work.

## Requirements

- Rust (see `rust-toolchain.toml`, tested with stable)
- GPG with a reachable key for the `-u/--user` flag
- Linux: `gpg`, `gocryptfs`, `fusermount`
- Windows: `gpg` (e.g., Gpg4win), `cppcryptfs.exe`, `cppcryptfsctl.exe`

## Installation

Clone the repository and build the binary:

```bash
git clone https://github.com/programfan/cryptfs-cli.git
cd cryptfs-cli
cargo build --release
```

On Windows:

```powershell
cargo build --release
```

The resulting binary lives at `target/release/cryptfs-cli`.

### Tooling

- `rust-toolchain.toml` pins the toolchain to stable with `rustfmt` and `clippy`.
- `cargo fmt` formats the codebase.
- `cargo clippy -- -D warnings` lints.
- `cargo build` or `cargo build --release` produces the binary.

## Repository layout

`cryptfs-cli create` produces the following structure:

- `objects/`: the ciphertext directory passed to gocryptfs/cppcryptfs
- `passphrase.gpg`: GPG-encrypted random passphrase (0600)
- `gocryptfs.conf`: backend config generated during init (0600)

## Usage

`cryptfs-cli` provides three main commands: `create`, `mount`, and `umount`. Each command comes with built-in help (`--help`) for usage details.

### Commands

1. **Create a Repository**

   Initializes a new encrypted repository.

   ```bash
   cryptfs-cli create -u <gpg_user> <repo_dir>
   ```

   - `-u, --user <gpg_user>`: Specifies the GPG user/email for encryption.
   - `<repo_dir>`: The directory for the encrypted repository.

   The command generates a random passphrase, encrypts it to the provided GPG identity, and initializes the backend with deterministic names (`gocryptfs -init` on Linux, `cppcryptfsctl --init` on Windows). Config files are written with 0600 permissions.

2. **Mount a Repository**

   Mounts an encrypted repository to a specified directory.

   ```bash
   cryptfs-cli mount [flags] <repo_dir> <mount_point>
   ```

   - `-o, --options <options>`: Options to pass to the cryptfs tool (comma-separated).
   - `<repo_dir>`: The repository directory.
   - `<mount_point>`: The directory to mount the decrypted content.

On Linux, the mount point is created if missing and `gocryptfs` is invoked with `-extpass "gpg --decrypt passphrase.gpg"` and the provided options (passed directly to `-o`). On Windows, supply a drive letter (e.g., `X:`) or absolute path; `cppcryptfs` is called with the decrypted passphrase, and the `-o/--options` flag is currently ignored on that platform.

3. **Unmount a Repository**

   Unmounts a previously mounted repository.

   ```bash
   cryptfs-cli umount <mount_point>
   ```

   - `<mount_point>`: The mount point to unmount. Uses `fusermount -u` on Linux and `cppcryptfs --unmount` on Windows.

## Example (Linux)

```bash
# Create
cryptfs-cli create -u alice@example.com ~/secure-notes

# Mount
cryptfs-cli mount ~/secure-notes ~/secure-notes-plain

# Unmount
cryptfs-cli umount ~/secure-notes-plain
```

## License

This project is licensed under the MIT License - see the [LICENSE](LICENSE) file for details.

## Contributing

Contributions are welcome! Please open issues or submit pull requests for any improvements or bug fixes.

## Notes

- **Security**: This tool manages encrypted repositories and passphrases, so ensure your GPG keys and repository files are secure.
- **Cross-platform paths**: Use appropriate path syntax for each operating system.
