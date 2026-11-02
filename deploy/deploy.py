#!/usr/bin/env python3
"""Deploy script for novk service."""

import argparse
from pathlib import Path
import shlex
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
    """Execute a command on the remote target host over SSH via POSIX sh."""
    print(f"[SSH] {target} -> {remote_command}")
    sh_command = f"sh -c {shlex.quote(remote_command)}"
    return subprocess.run(
        args=["ssh", target, sh_command],
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


def build_binary() -> None:
    """Compile the Rust binary in release mode."""
    cargo_manifest = REPO_ROOT / "Cargo.toml"
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


def deploy_env(target: str, remote_app_dir: str) -> None:
    """Check for .env in deploy directory and copy with restricted permissions."""
    local_env = DEPLOY_DIR / ".env"
    if not local_env.is_file():
        print(f"[WARN] Environment file {local_env} not found, skipping.")
        return

    print(f"[INFO] Copying {local_env} to {target}:{remote_app_dir}/.env...")
    run_command(command=["scp", str(local_env), f"{target}:{remote_app_dir}/.env"])
    run_remote_ssh(target=target, remote_command=f"chmod 600 {remote_app_dir}/.env")


def stop_remote_service(target: str) -> None:
    """Stop the remote service before updating files."""
    print("[INFO] Stopping novk.service before update...")
    run_remote_ssh(target=target, remote_command="systemctl stop novk.service 2>/dev/null || true")


def deploy_app_and_service(target: str, remote_app_dir: str) -> None:
    """Copy service file, binary, configs, and assets to server."""
    bin_path = REPO_ROOT / "target" / "release" / "novk-cli"
    if not bin_path.is_file():
        raise RuntimeError(f"Binary not found at {bin_path}. Run build first.")

    config_dir = REPO_ROOT / "config"
    assets_dir = REPO_ROOT / "assets"
    service_file = DEPLOY_DIR / "novk.service"

    print(f"[INFO] Ensuring directories on {target}...")
    run_remote_ssh(
        target=target,
        remote_command=f"mkdir -p {remote_app_dir}/config {remote_app_dir}/assets",
    )

    print(f"[INFO] Updating novk.service on {target}:/etc/systemd/system/...")
    run_command(command=["scp", str(service_file), f"{target}:/etc/systemd/system/novk.service"])
    run_remote_ssh(target=target, remote_command="systemctl daemon-reload && systemctl enable novk.service")

    print(f"[INFO] Copying binary to {target}:{remote_app_dir}/novk-cli...")
    run_command(command=["scp", str(bin_path), f"{target}:{remote_app_dir}/novk-cli"])
    run_remote_ssh(target=target, remote_command=f"chmod 755 {remote_app_dir}/novk-cli")

    print(f"[INFO] Copying config files to {target}:{remote_app_dir}/config/...")
    config_items = [str(p) for p in config_dir.iterdir()]
    if config_items:
        run_command(command=["scp", "-r", *config_items, f"{target}:{remote_app_dir}/config/"])

    print(f"[INFO] Copying assets to {target}:{remote_app_dir}/assets/...")
    assets_items = [str(p) for p in assets_dir.iterdir()]
    if assets_items:
        run_command(command=["scp", "-r", *assets_items, f"{target}:{remote_app_dir}/assets/"])


def start_remote_service(target: str) -> None:
    """Start the novk service and check status."""
    print("[INFO] Starting novk.service and checking status...")
    remote_script = (
        "systemctl start novk.service && "
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
    return parser.parse_args()


def main() -> None:
    """Main deployment workflow."""
    args = parse_arguments()
    target = args.target

    remote_app_dir = "/opt/novk"
    remote_data_dir = "/mnt/novk-data/data/novk"
    remote_db_path = f"{remote_data_dir}/novk.sqlite"
    remote_audio_dir = f"{remote_data_dir}/static/uploaded/audio"

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
        build_binary()

        # Step 5: Stop running service
        stop_remote_service(target=target)

        # Step 6: Transfer environment, service file, binary, configs, and assets
        deploy_env(target=target, remote_app_dir=remote_app_dir)
        deploy_app_and_service(target=target, remote_app_dir=remote_app_dir)

        # Step 7: Start service and verify status
        start_remote_service(target=target)

        print("[SUCCESS] Deployment completed successfully.")

    except subprocess.CalledProcessError as e:
        print(f"[ERROR] Command failed with return code {e.returncode}: {repr(e)}", file=sys.stderr)
        sys.exit(e.returncode)
    except Exception as e:
        print(f"[ERROR] Deployment aborted: {repr(e)}", file=sys.stderr)
        sys.exit(1)


if __name__ == "__main__":
    main()
