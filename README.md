# cryptfs-cli

`cryptfs-cli` is a cross-platform command-line tool for creating, mounting, and unmounting encrypted repositories using **GPG** for passphrase management, **gocryptfs** (Linux), and **cppcryptfs** (Windows).

## Features

- **Cross-platform**: Runs on both Linux and Windows.
- **Flexible Encryption**: Uses `gocryptfs` on Linux and `cppcryptfs` on Windows.
- **Secure Passphrase Management**: Leverages GPG for passphrase encryption.
- **User-friendly CLI**: Built with the Cobra library for comprehensive help and documentation.

## Installation

### Prerequisites

Ensure the following tools are installed and accessible in your system’s PATH:

- **Linux**: `gocryptfs`, `gpg`, `fusermount`
- **Windows**: `cppcryptfs.exe`, `cppcryptfsctl.exe`, `gpg` (from Gpg4win)

### Build

Clone the repository and build the binary:

```bash
git clone https://github.com/yourusername/cryptfs-cli.git
cd cryptfs-cli
go build -o cryptfs-cli
```

On Windows, use the following to create the executable:

```powershell
go build -o cryptfs-cli.exe
```

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

2. **Mount a Repository**

   Mounts an encrypted repository to a specified directory.

   ```bash
   cryptfs-cli mount [flags] <repo_dir> <mount_point>
   ```

   - `-o, --options <options>`: Options to pass to the cryptfs tool (comma-separated).
   - `<repo_dir>`: The repository directory.
   - `<mount_point>`: The directory to mount the decrypted content.

3. **Unmount a Repository**

   Unmounts a previously mounted repository.

   ```bash
   cryptfs-cli umount <mount_point>
   ```

   - `<mount_point>`: The mount point to unmount.

## License

This project is licensed under the MIT License - see the [LICENSE](LICENSE) file for details.

## Contributing

Contributions are welcome! Please open issues or submit pull requests for any improvements or bug fixes.

## Notes

- **Security**: This tool manages encrypted repositories and passphrases, so ensure your GPG keys and repository files are secure.
- **Cross-platform paths**: Use appropriate path syntax for each operating system.
