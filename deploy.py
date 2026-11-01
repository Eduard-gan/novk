#!/usr/bin/env python3
"""Deploy script for novk service."""

import argparse
from pathlib import Path
import subprocess
import sys


def run_command(command: list[str]) -> None:
    """Execute a local shell command and raise on failure."""
    cmd_str = " ".join(command)
    print(f"[CMD] {cmd_str}")
    subprocess.run(args=command, check=True)


def run_remote_ssh(target: str, remote_command: str) -> subprocess.CompletedProcess:
    """Execute a command on the remote target host over SSH."""
    print(f"[SSH] {target} -> {remote_command}")
    return subprocess.run(
        args=["ssh", target, remote_command],
        check=True,
    )


def deploy_database(target: str, remote_db_path: str) -> None:
    """Check for local database, backup remote if present, and copy."""
    local_db = Path("novk.sqlite")
    if not local_db.is_file():
        print("[INFO] Database file novk.sqlite not found in ., skipping.")
        return

    print(f"[INFO] Found local database: {local_db}. Preparing remote backup...")
    backup_cmd = (
        f"if [ -f {remote_db_path} ]; then "
        f"cp {remote_db_path} {remote_db_path}.bak.$(date +%s) && "
        f"echo '[INFO] Remote database backup created.'; fi"
    )
    run_remote_ssh(target=target, remote_command=backup_cmd)

    print(f"[INFO] Copying {local_db} to {target}:{remote_db_path}...")
    run_command(command=["rsync", "-avz", str(local_db), f"{target}:{remote_db_path}"])


def deploy_audio(target: str, remote_audio_dir: str) -> None:
    """Check for local audio directory and sync without deleting existing files."""
    local_audio = Path("audio")
    if not local_audio.is_dir():
        print("[INFO] Audio directory ./audio not found in ., skipping.")
        return

    print(f"[INFO] Found local audio directory: {local_audio}. Syncing files...")
    run_command(
        command=[
            "rsync",
            "-av",
            f"{local_audio}/",
            f"{target}:{remote_audio_dir}/",
        ]
    )
    print("[INFO] Audio files synchronized successfully.")


def build_binary(skip_build: bool) -> None:
    """Compile the Rust binary in release mode if not skipped."""
    if skip_build:
        print("[INFO] Skipping local cargo build as requested.")
        return

    print("[INFO] Building Rust binary in release mode...")
    run_command(
        command=[
            "cargo",
            "build",
            "--release",
            "--manifest-path",
            "rs/Cargo.toml",
        ]
    )


def deploy_app_and_service(target: str, remote_app_dir: str) -> None:
    """Copy binary, configs, assets, and service file to server."""
    bin_path = Path("rs/target/release/novk-cli")
    if not bin_path.is_file():
        raise RuntimeError(f"Binary not found at {bin_path}. Run build first.")

    print(f"[INFO] Ensuring directories on {target}...")
    run_remote_ssh(
        target=target,
        remote_command=f"mkdir -p {remote_app_dir}/config {remote_app_dir}/assets",
    )

    print(f"[INFO] Copying binary and assets to {target}:{remote_app_dir}...")
    run_command(
        command=[
            "rsync",
            "-avz",
            str(bin_path),
            f"{target}:{remote_app_dir}/novk-cli",
        ]
    )
    run_command(
        command=[
            "rsync",
            "-avz",
            "rs/config/",
            f"{target}:{remote_app_dir}/config/",
        ]
    )
    run_command(
        command=[
            "rsync",
            "-avz",
            "rs/assets/",
            f"{target}:{remote_app_dir}/assets/",
        ]
    )

    print(f"[INFO] Copying novk.service to {target}:/etc/systemd/system/...")
    run_command(
        command=[
            "rsync",
            "-avz",
            "novk.service",
            f"{target}:/etc/systemd/system/novk.service",
        ]
    )


def manage_remote_service(target: str) -> None:
    """Reload systemd and start or restart the novk service based on status."""
    print("[INFO] Registering and managing remote systemd service...")
    remote_script = (
        "systemctl daemon-reload && "
        "systemctl enable novk.service && "
        "if systemctl is-active --quiet novk.service; then "
        "  echo '[INFO] Service is active. Restarting novk.service...' && "
        "  systemctl restart novk.service; "
        "else "
        "  echo '[INFO] Service is inactive. Starting novk.service...' && "
        "  systemctl start novk.service; "
        "fi && "
        "systemctl status novk.service --no-pager"
    )
    run_remote_ssh(target=target, remote_command=remote_script)


def parse_arguments() -> argparse.Namespace:
    """Parse command line arguments."""
    parser = argparse.ArgumentParser(description="Deploy novk service to server.")
    parser.add_argument(
        "target",
        nargs="?",
        default="root@novk.localplayer.dev",
        help="Target SSH host (default: root@novk.localplayer.dev)",
    )
    parser.add_argument(
        "--skip-build",
        action="store_true",
        help="Skip local cargo build step",
    )
    return parser.parse_args()


def main() -> None:
    """Main deployment workflow."""
    args = parse_arguments()
    target = args.target

    remote_app_dir = "/opt/novk"
    remote_data_dir = "/mnt/novk-data/data/novk"
    remote_db_path = f"{remote_data_dir}/novk.sqlite"
    remote_audio_dir = f"{remote_data_dir}/audio"

    print(f"[START] Deploying novk to {target}...")

    try:
        # Step 1: Ensure remote directories exist
        run_remote_ssh(
            target=target,
            remote_command=f"mkdir -p {remote_data_dir} {remote_audio_dir} {remote_app_dir}",
        )

        # Step 2: Handle database
        deploy_database(target=target, remote_db_path=remote_db_path)

        # Step 3: Handle audio files
        deploy_audio(target=target, remote_audio_dir=remote_audio_dir)

        # Step 4: Build locally
        build_binary(skip_build=args.skip_build)

        # Step 5: Transfer files
        deploy_app_and_service(target=target, remote_app_dir=remote_app_dir)

        # Step 6: Reload and start/restart service
        manage_remote_service(target=target)

        print("[SUCCESS] Deployment completed successfully.")

    except subprocess.CalledProcessError as e:
        print(f"[ERROR] Command failed with return code {e.returncode}: {repr(e)}", file=sys.stderr)
        sys.exit(e.returncode)
    except Exception as e:
        print(f"[ERROR] Deployment aborted: {repr(e)}", file=sys.stderr)
        sys.exit(1)


if __name__ == "__main__":
    main()
