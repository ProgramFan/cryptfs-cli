#!/usr/bin/env python3

import argparse
import os
import subprocess
import sys

def parse_args():
    parser = argparse.ArgumentParser(description='Manage gocryptfs-encrypted repositories.')
    subparsers = parser.add_subparsers(dest='command', required=True)

    # Create command
    create_parser = subparsers.add_parser('create', help='Create a new encrypted repository')
    create_parser.add_argument('-u', '--user', required=True, help='GPG user/email for encryption')
    create_parser.add_argument('repo_dir', help='Repository directory')

    # Mount command
    mount_parser = subparsers.add_parser('mount', help='Mount an encrypted repository')
    mount_parser.add_argument('-o', '--options', help='Options to pass to gocryptfs (comma-separated)')
    mount_parser.add_argument('repo_dir', help='Repository directory')
    mount_parser.add_argument('mount_point', help='Mount point directory')

    # Umount command
    umount_parser = subparsers.add_parser('umount', help='Unmount a repository')
    umount_parser.add_argument('mount_point', help='Mount point directory')

    return parser.parse_args()

def create_repository(user, repo_dir):
    # Ensure repo_dir does not exist
    if os.path.exists(repo_dir):
        print(f"Error: Directory '{repo_dir}' already exists.")
        sys.exit(1)

    os.makedirs(repo_dir)
    os.makedirs(os.path.join(repo_dir, 'objects'))

    # Generate and encrypt passphrase in one step using pipes
    print("Generating and encrypting passphrase with GPG...")
    passphrase_file = os.path.join(repo_dir, 'passphrase.gpg')
    try:
        gen_random = subprocess.Popen(['gpg', '--gen-random', '--armor', '0', '64'], stdout=subprocess.PIPE)
        encrypt_process = subprocess.Popen(
            ['gpg', '--encrypt', '--sign', '-r', user, '-o', passphrase_file],
            stdin=gen_random.stdout,
            stderr=subprocess.PIPE
        )
        gen_random.stdout.close()
        _, err = encrypt_process.communicate()
        if encrypt_process.returncode != 0:
            print("Error encrypting passphrase:", err.decode())
            sys.exit(1)
    except Exception as e:
        print("Encryption failed:", str(e))
        sys.exit(1)

    # Set permissions on passphrase.gpg
    os.chmod(passphrase_file, 0o600)

    # Initialize gocryptfs using the encrypted passphrase directly
    print("Initializing gocryptfs...")
    try:
        extpass_cmd = ['gpg', '--decrypt', passphrase_file]
        gocryptfs_cmd = [
            'gocryptfs',
            '-init',
            '--deterministic-names',
            '--config', os.path.join(repo_dir, 'gocryptfs.conf'),
        ]

        # Add the extpass arguments using -extpass=arg
        for arg in extpass_cmd:
            gocryptfs_cmd.append(f'-extpass={arg}')

        # Add the cipher directory
        gocryptfs_cmd.append(os.path.join(repo_dir, 'objects'))

        init_process = subprocess.Popen(
            gocryptfs_cmd,
            stderr=subprocess.PIPE
        )
        _, err = init_process.communicate()
        if init_process.returncode != 0:
            print("Error initializing gocryptfs:", err.decode())
            sys.exit(1)
    except Exception as e:
        print("gocryptfs initialization failed:", str(e))
        sys.exit(1)

    # Set permissions on gocryptfs.conf
    os.chmod(os.path.join(repo_dir, 'gocryptfs.conf'), 0o600)

    print(f"Repository created successfully at '{repo_dir}'")

def mount_repository(repo_dir, mount_point, options):
    # Check repository layout
    passphrase_file = os.path.join(repo_dir, 'passphrase.gpg')
    gocryptfs_conf_file = os.path.join(repo_dir, 'gocryptfs.conf')
    cipher_dir = os.path.join(repo_dir, 'objects')

    if not os.path.exists(passphrase_file) or not os.path.exists(gocryptfs_conf_file) or not os.path.exists(cipher_dir):
        print("Error: Repository layout is invalid.")
        sys.exit(1)

    # Ensure mount_point exists
    if not os.path.exists(mount_point):
        os.makedirs(mount_point)

    # Use gpg --decrypt as extpass command
    extpass_cmd = ['gpg', '--decrypt', passphrase_file]

    # Prepare gocryptfs command
    gocryptfs_cmd = [
        'gocryptfs',
        '--config', gocryptfs_conf_file
    ]

    # Add the extpass arguments using -extpass=arg
    for arg in extpass_cmd:
        gocryptfs_cmd.append(f'-extpass={arg}')

    # Add mount options if provided
    if options:
        # Split the options by comma and remove any surrounding whitespace
        mount_options = [opt.strip() for opt in options.split(',')]
        gocryptfs_cmd.extend(['-o', ','.join(mount_options)])

    # Add cipher_dir and mount_point
    gocryptfs_cmd.extend([cipher_dir, mount_point])

    # Mount using gocryptfs
    print("Mounting gocryptfs...")
    try:
        mount_process = subprocess.Popen(
            gocryptfs_cmd,
            stderr=subprocess.PIPE
        )
        _, err = mount_process.communicate()
        if mount_process.returncode != 0:
            print("Error mounting gocryptfs:", err.decode())
            sys.exit(1)
    except Exception as e:
        print("Mounting failed:", str(e))
        sys.exit(1)

    print(f"Mounted '{repo_dir}' at '{mount_point}'")

def umount_repository(mount_point):
    print(f"Unmounting '{mount_point}'...")
    try:
        subprocess.check_call(['fusermount', '-u', mount_point])
        print(f"Unmounted '{mount_point}' successfully.")
    except subprocess.CalledProcessError as e:
        print(f"Error unmounting '{mount_point}':", e)
        sys.exit(1)

def main():
    args = parse_args()

    if args.command == 'create':
        create_repository(args.user, args.repo_dir)
    elif args.command == 'mount':
        mount_repository(args.repo_dir, args.mount_point, args.options)
    elif args.command == 'umount':
        umount_repository(args.mount_point)
    else:
        print("Invalid command. Use --help for more information.")
        sys.exit(1)

if __name__ == "__main__":
    main()

