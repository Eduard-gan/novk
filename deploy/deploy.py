#!/usr/bin/env python3
"""Deploy script for novk service."""

import argparse
from pathlib import Path
import subprocess
import sys

DEPLOY_DIR = Path(__file__).resolve().parent
REPO_ROOT = DEPLOY_DIR.parent


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
    """Check for database in deploy directory, backup remote if present, and copy."""
    local_db = DEPLOY_DIR / "novk.sqlite"
    if not local_db.is_file():
        print(f"[INFO] Database file {local_db.name} not found in deploy/, skipping.")
        return

    print(f"[INFO] Found local database: {local_db}. Preparing remote backup...")
    backup_cmd = (
        f"if [ -f {remote_db_path} ]; then "
        f"cp {remote_db_path} {remote_db_path}.bak.$(date +%s) && "
        f"echo '[INFO] Remote database backup created.'; fi"
    )
    run_remote_ssh(target=target, remote_command=backup_cmd)

    print(f"[INFO] Copying {local_db} to {target}:{remote_db_path}...")
    run_command(command=["scp", str(local_db), f"{target}:{remote_db_path}"])


def deploy_audio(target: str, remote_audio_dir: str) -> None:
    """Check for audio directory in deploy/ and copy files without deleting remote ones."""
    local_audio = DEPLOY_DIR / "audio"
    if not local_audio.is_dir():
        print("[INFO] Audio directory deploy/audio not found, skipping.")
        return

    audio_files = [str(p) for p in local_audio.iterdir() if p.is_file()]
    if not audio_files:
        print("[INFO] No files found in deploy/audio, skipping.")
        return

    print(f"[INFO] Found {len(audio_files)} audio file(s) in deploy/audio. Copying via scp...")
    run_command(command=["scp", *audio_files, f"{target}:{remote_audio_dir}/"])
    print("[INFO] Audio files copied successfully.")


def build_binary(skip_build: bool) -> None:
    """Compile the Rust binary in release mode if not skipped."""
    if skip_build:
        print("[INFO] Skipping local cargo build as requested.")
        return

    cargo_manifest = REPO_ROOT / "rs" / "Cargo.toml"
    print("[INFO] Building Rust binary in release mode...")
    run_command(
        command=[
            "cargo",
            "build",
            "--release",
            "--manifest-path",
            str(cargo_manifest),
        ]
    )


def deploy_app_and_service(target: str, remote_app_dir: str) -> None:
    """Copy binary, configs, assets, and service file to server."""
    bin_path = REPO_ROOT / "rs" / "target" / "release" / "novk-cli"
    if not bin_path.is_file():
        raise RuntimeError(f"Binary not found at {bin_path}. Run build first.")

    config_dir = REPO_ROOT / "rs" / "config"
    assets_dir = REPO_ROOT / "rs" / "assets"
    service_file = DEPLOY_DIR / "novk.service"

    print(f"[INFO] Ensuring directories on {target}...")
    run_remote_ssh(
        target=target,
        remote_command=f"mkdir -p {remote_app_dir}/config {remote_app_dir}/assets",
    )

    print(f"[INFO] Copying binary to {target}:{remote_app_dir}/novk-cli...")
    run_command(command=["scp", str(bin_path), f"{target}:{remote_app_dir}/novk-cli"])

    print(f"[INFO] Copying config files to {target}:{remote_app_dir}/config/...")
    config_items = [str(p) for p in config_dir.iterdir()]
    if config_items:
        run_command(command=["scp", "-r", *config_items, f"{target}:{remote_app_dir}/config/"])

    print(f"[INFO] Copying assets to {target}:{remote_app_dir}/assets/...")
    assets_items = [str(p) for p in assets_dir.iterdir()]
    if assets_items:
        run_command(command=["scp", "-r", *assets_items, f"{target}:{remote_app_dir}/assets/"])

    print(f"[INFO] Copying novk.service to {target}:/etc/systemd/system/...")
    run_command(command=["scp", str(service_file), f"{target}:/etc/systemd/system/novk.service"])


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
