// cryptfs-cli.go

package main

import (
	"bytes"
	"fmt"
	"os"
	"os/exec"
	"path/filepath"
	"runtime"
	"strings"

	"github.com/spf13/cobra"
)

func main() {
	var rootCmd = &cobra.Command{
		Use:     "cryptfs-cli",
		Short:   "cryptfs-cli manages encrypted repositories using gpg and cryptfs tools",
		Long:    "cryptfs-cli allows you to create, mount, and unmount encrypted repositories using GPG for passphrase management and cryptfs tools (gocryptfs or cppcryptfs) for the backends.",
		Version: "0.1.0",
	}

	rootCmd.AddCommand(createCmd)
	rootCmd.AddCommand(mountCmd)
	rootCmd.AddCommand(umountCmd)

	if err := rootCmd.Execute(); err != nil {
		fmt.Println(err)
		os.Exit(1)
	}
}

var createCmd = &cobra.Command{
	Use:   "create -u <user> <repo_dir>",
	Short: "Create a new encrypted repository",
	Args:  cobra.ExactArgs(1),
	Run: func(cmd *cobra.Command, args []string) {
		user, _ := cmd.Flags().GetString("user")
		if user == "" {
			fmt.Println("Error: GPG user/email is required")
			cmd.Help()
			os.Exit(1)
		}
		repoDir := args[0]
		err := createRepository(user, repoDir)
		if err != nil {
			fmt.Printf("Error: %v\n", err)
			os.Exit(1)
		}
	},
}

var mountCmd = &cobra.Command{
	Use:   "mount [flags] <repo_dir> <mount_point>",
	Short: "Mount an encrypted repository",
	Args:  cobra.ExactArgs(2),
	Run: func(cmd *cobra.Command, args []string) {
		options, _ := cmd.Flags().GetString("options")
		repoDir := args[0]
		mountPoint := args[1]
		err := mountRepository(repoDir, mountPoint, options)
		if err != nil {
			fmt.Printf("Error: %v\n", err)
			os.Exit(1)
		}
	},
}

var umountCmd = &cobra.Command{
	Use:   "umount <mount_point>",
	Short: "Unmount a repository",
	Args:  cobra.ExactArgs(1),
	Run: func(_ *cobra.Command, args []string) {
		mountPoint := args[0]
		err := umountRepository(mountPoint)
		if err != nil {
			fmt.Printf("Error: %v\n", err)
			os.Exit(1)
		}
	},
}

func init() {
	createCmd.Flags().StringP("user", "u", "", "GPG user/email for encryption (required)")
	mountCmd.Flags().StringP("options", "o", "", "Options to pass to the cryptfs tool (comma-separated)")
}

func createRepository(user string, repoDir string) error {
	// Ensure repoDir does not exist
	if _, err := os.Stat(repoDir); err == nil {
		return fmt.Errorf("directory '%s' already exists", repoDir)
	}

	objectsDir := filepath.Join(repoDir, "objects")
	err := os.MkdirAll(objectsDir, 0700)
	if err != nil {
		return fmt.Errorf("failed to create directories: %v", err)
	}

	passphraseFile := filepath.Join(repoDir, "passphrase.gpg")

	// Generate and encrypt passphrase using pipes
	fmt.Println("Generating and encrypting passphrase with GPG...")
	genRandomCmd := exec.Command("gpg", "--gen-random", "--armor", "0", "64")
	encryptCmd := exec.Command("gpg", "--encrypt", "--sign", "-r", user, "-o", passphraseFile)

	pipe, err := genRandomCmd.StdoutPipe()
	if err != nil {
		return fmt.Errorf("failed to create pipe: %v", err)
	}
	encryptCmd.Stdin = pipe

	var encryptStderr bytes.Buffer
	encryptCmd.Stderr = &encryptStderr

	err = encryptCmd.Start()
	if err != nil {
		return fmt.Errorf("failed to start encryption command: %v", err)
	}
	err = genRandomCmd.Run()
	if err != nil {
		return fmt.Errorf("failed to generate random data: %v", err)
	}
	err = encryptCmd.Wait()
	if err != nil {
		return fmt.Errorf("error encrypting passphrase: %v, %s", err, encryptStderr.String())
	}

	// Set permissions on passphrase.gpg
	err = os.Chmod(passphraseFile, 0600)
	if err != nil {
		return fmt.Errorf("failed to set permissions on passphrase.gpg: %v", err)
	}

	// Platform-specific initialization
	if runtime.GOOS == "windows" {
		// Windows: Use cppcryptfs
		return createRepositoryWindows(repoDir, objectsDir, passphraseFile)
	} else {
		// Linux and others: Use gocryptfs
		return createRepositoryLinux(repoDir, objectsDir, passphraseFile)
	}
}

func createRepositoryLinux(repoDir, objectsDir, passphraseFile string) error {
	fmt.Println("Initializing gocryptfs...")
	extpassCmd := []string{"gpg", "--decrypt", passphraseFile}
	gocryptfsCmd := []string{
		"gocryptfs",
		"-init",
		"--deterministic-names",
		"--config", filepath.Join(repoDir, "gocryptfs.conf"),
	}

	for _, arg := range extpassCmd {
		gocryptfsCmd = append(gocryptfsCmd, fmt.Sprintf("-extpass=%s", arg))
	}

	gocryptfsCmd = append(gocryptfsCmd, objectsDir)

	initCmd := exec.Command(gocryptfsCmd[0], gocryptfsCmd[1:]...)

	var initStderr bytes.Buffer
	initCmd.Stderr = &initStderr

	err := initCmd.Run()
	if err != nil {
		return fmt.Errorf("error initializing gocryptfs: %v, %s", err, initStderr.String())
	}

	// Set permissions on gocryptfs.conf
	gocryptfsConf := filepath.Join(repoDir, "gocryptfs.conf")
	err = os.Chmod(gocryptfsConf, 0600)
	if err != nil {
		return fmt.Errorf("failed to set permissions on gocryptfs.conf: %v", err)
	}

	fmt.Printf("Repository created successfully at '%s'\n", repoDir)
	return nil
}

func createRepositoryWindows(repoDir, objectsDir, passphraseFile string) error {
	fmt.Println("Initializing cppcryptfs...")
	cppcryptfsCtlPath := "cppcryptfsctl.exe" // Ensure this is in PATH or specify full path

	volumeName := filepath.Base(repoDir)
	initArgs := []string{
		"--init=" + objectsDir,
		"--volumename=" + volumeName,
		"--deterministicnames",
	}

	// Decrypt the passphrase
	fmt.Println("Decrypting passphrase...")
	passphraseBytes, err := decryptPassphrase(passphraseFile)
	if err != nil {
		return fmt.Errorf("failed to decrypt passphrase: %v", err)
	}

	initCmd := exec.Command(cppcryptfsCtlPath, initArgs...)
	initCmd.Stdin = bytes.NewReader(passphraseBytes)

	var initStderr bytes.Buffer
	initCmd.Stderr = &initStderr

	err = initCmd.Run()
	if err != nil {
		return fmt.Errorf("error initializing cppcryptfs: %v, %s", err, initStderr.String())
	}

	srcConfPath := filepath.Join(objectsDir, "gocryptfs.conf")
	dstConfPath := filepath.Join(repoDir, "gocryptfs.conf")
	err = os.Rename(srcConfPath, dstConfPath)
	if err != nil {
		return fmt.Errorf("failed to move gocryptfs.conf: %v", err)
	}

	fmt.Printf("Repository created successfully at '%s'\n", filepath.Dir(objectsDir))
	return nil
}

func decryptPassphrase(passphraseFile string) ([]byte, error) {
	gpgCmd := exec.Command("gpg", "--decrypt", passphraseFile)
	var out bytes.Buffer
	var stderr bytes.Buffer
	gpgCmd.Stdout = &out
	gpgCmd.Stderr = &stderr

	err := gpgCmd.Run()
	if err != nil {
		return nil, fmt.Errorf("gpg decryption failed: %v, %s", err, stderr.String())
	}

	return out.Bytes(), nil
}

func isWindowsDriveLetter(path string) bool {
	if len(path) == 2 && path[1] == ':' {
		// Check if first character is a letter (A-Z or a-z)
		return (path[0] >= 'A' && path[0] <= 'Z') || (path[0] >= 'a' && path[0] <= 'z')
	}
	return false
}

func mountRepository(repoDir string, mountPoint string, options string) error {
	passphraseFile := filepath.Join(repoDir, "passphrase.gpg")
	repoAbsDir, _ := filepath.Abs(repoDir)
	cipherDir := filepath.Join(repoAbsDir, "objects")

	// Check repository layout
	if !fileExists(passphraseFile) || !dirExists(cipherDir) {
		return fmt.Errorf("repository layout is invalid")
	}

	// Ensure mountPoint exists
	if runtime.GOOS != "windows" || !isWindowsDriveLetter(mountPoint) {
		// Convert to absolute path (important for cppcryptfs)
		absPath, err := filepath.Abs(mountPoint)
		if err != nil {
			return fmt.Errorf("failed to get absolute path: %v", err)
		}
		// Check if directory already exists
		if _, err := os.Stat(absPath); os.IsNotExist(err) {
			// Directory doesn't exist, create it
			err = os.MkdirAll(absPath, 0700)
			if err != nil && !os.IsExist(err) {
				return fmt.Errorf("failed to create mount point: %v", err)
			}
		} else if err != nil && !os.IsExist(err) {
			// Some other error occurred during stat
			return fmt.Errorf("failed to check mount point: %v", err)
		}
		mountPoint = absPath
	}

	if runtime.GOOS == "windows" {
		// Windows: Use cppcryptfs
		return mountRepositoryWindows(cipherDir, passphraseFile, mountPoint, options, repoDir)
	} else {
		// Linux and others: Use gocryptfs
		return mountRepositoryLinux(cipherDir, passphraseFile, mountPoint, options, repoDir)
	}
}

func mountRepositoryLinux(cipherDir, passphraseFile, mountPoint, options, repoDir string) error {
	fmt.Println("Mounting gocryptfs...")
	extpassCmd := []string{"gpg", "--decrypt", passphraseFile}
	gocryptfsCmd := []string{
		"gocryptfs",
		"--config", filepath.Join(repoDir, "gocryptfs.conf"),
	}

	for _, arg := range extpassCmd {
		gocryptfsCmd = append(gocryptfsCmd, fmt.Sprintf("-extpass=%s", arg))
	}

	if options != "" {
		gocryptfsCmd = append(gocryptfsCmd, "-o", options)
	}

	gocryptfsCmd = append(gocryptfsCmd, cipherDir, mountPoint)

	mountCmd := exec.Command(gocryptfsCmd[0], gocryptfsCmd[1:]...)

	var mountStderr bytes.Buffer
	mountCmd.Stderr = &mountStderr

	err := mountCmd.Run()
	if err != nil {
		return fmt.Errorf("error mounting gocryptfs: %v, %s", err, mountStderr.String())
	}

	fmt.Printf("Mounted '%s' at '%s'\n", cipherDir, mountPoint)
	return nil
}

func mountRepositoryWindows(cipherDir, passphraseFile, mountPoint, options, repoDir string) error {
	fmt.Println("Decrypting passphrase...")
	passphraseBytes, err := decryptPassphrase(passphraseFile)
	if err != nil {
		return fmt.Errorf("failed to decrypt passphrase: %v", err)
	}

	cppcryptfsPath := "cppcryptfs.exe" // Ensure this is in PATH or specify full path
	mountArgs := []string{
		"--mount=" + cipherDir,
		"--drive=" + mountPoint,
		"--password=" + strings.TrimSpace(string(passphraseBytes)),
		"--config=" + filepath.Join(repoDir, "gocryptfs.conf"),
		"-t", "-x",
	}

	// Add mount options if cppcryptfs supports them
	if options != "" {
		// Parse and add options as needed
	}

	fmt.Println("Mounting cppcryptfs...")
	mountCmd := exec.Command(cppcryptfsPath, mountArgs...)

	var mountStderr bytes.Buffer
	mountCmd.Stderr = &mountStderr

	err = mountCmd.Run()
	if err != nil {
		return fmt.Errorf("error mounting cppcryptfs: %v, %s", err, mountStderr.String())
	}

	fmt.Printf("Mounted '%s' at '%s'\n", cipherDir, mountPoint)
	return nil
}

func umountRepository(mountPoint string) error {
	if !isWindowsDriveLetter(mountPoint) {
		mountPoint, _ = filepath.Abs(mountPoint)
	}
	if runtime.GOOS == "windows" {
		return umountRepositoryWindows(mountPoint)
	} else {
		return umountRepositoryLinux(mountPoint)
	}
}

func umountRepositoryLinux(mountPoint string) error {
	fmt.Printf("Unmounting '%s'...\n", mountPoint)
	umountCmd := exec.Command("fusermount", "-u", mountPoint)

	var umountStderr bytes.Buffer
	umountCmd.Stderr = &umountStderr

	err := umountCmd.Run()
	if err == nil {
		fmt.Printf("Unmounted '%s' successfully.\n", mountPoint)
		return nil
	} else {
		return fmt.Errorf("error unmounting '%s': %v, %s", mountPoint, err, umountStderr.String())
	}
}

func umountRepositoryWindows(mountPoint string) error {
	fmt.Printf("Unmounting '%s'...\n", mountPoint)
	cppcryptfsPath := "cppcryptfs.exe" // Ensure this is in PATH or specify full path
	umountArgs := []string{
		"--unmount=" + mountPoint,
	}

	umountCmd := exec.Command(cppcryptfsPath, umountArgs...)

	var umountStderr bytes.Buffer
	umountCmd.Stderr = &umountStderr

	err := umountCmd.Run()
	if err != nil {
		return fmt.Errorf("error unmounting '%s': %v, %s", mountPoint, err, umountStderr.String())
	}

	fmt.Printf("Unmounted '%s' successfully.\n", mountPoint)
	return nil
}

func fileExists(filename string) bool {
	info, err := os.Stat(filename)
	if err != nil {
		return false
	}
	return !info.IsDir()
}

func dirExists(dirname string) bool {
	info, err := os.Stat(dirname)
	if err != nil {
		return false
	}
	return info.IsDir()
}
