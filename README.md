# NOVK

NOVK is a server-rendered music library and streaming service built with Rust and Loco.rs (Axum, SeaORM, SQLite, and Tera templates).

---

## 1. Stack and Architecture

- **Language & Framework:** Rust (edition 2021), Loco.rs 0.16.4.
- **Database:** SQLite with SeaORM.
- **UI:** Server-side rendered HTML (Tera templates), vanilla JavaScript audio player.
- **Media Storage:** Filesystem storage for MP3 files (`UPLOADED_DIR`).
- **Web Server:** Caddy as reverse proxy (TLS termination).
- **Deployment:** Bare-metal via local build, `scp`, and `systemd` (no containers).

---

## 2. Prerequisites

### Local Machine (Development & Deployment)
- **Rust:** Stable toolchain (`rustup`, `cargo`).
- **Python 3:** Required to execute the deployment script (`deploy/deploy.py`).
- **SSH access:** SSH key access to the target host (e.g. `ssh root@novk.localplayer.dev`).

### Target Server
- **OS:** Linux with `systemd`.
- **Web Server:** Caddy installed and running.
- **Storage:** Persistent directories for data and media (e.g. `/mnt/novk-data/`).

---

## 3. Local Development

### 1. Build and verify
```bash
cargo check --manifest-path rs/Cargo.toml
```

### 2. Apply database migrations
From repository root:
```bash
cargo loco db migrate --manifest-path rs/Cargo.toml
```

### 3. Create an initial user
```bash
cargo loco task users.create email:admin@novk.local password:secretpassword name:Admin --manifest-path rs/Cargo.toml
```

### 4. Start local server
```bash
cd rs && cargo loco start
```
The application will listen on `http://127.0.0.1:8000`.

---

## 4. Deployment

Deployment is performed from your local machine directly to the target server via systemd and SSH.

### Step 1: Configure Environment
Ensure `deploy/.env` exists locally (ignored by Git). If not, create it from the template:
```bash
cp deploy/.env.example deploy/.env
```
Generate a secure JWT secret:
```bash
openssl rand -hex 64
```
Fill in `deploy/.env`:
```ini
LOCO_ENV=production
PORT=8000
BINDING=172.18.0.1
DOMAIN=novk.localplayer.dev
DATABASE_URL=sqlite:///mnt/novk-data/data/novk/novk.sqlite?mode=rwc
JWT_SECRET=<your-generated-hex-key>
TZ=CET
UPLOADED_DIR=/mnt/novk-data/data/novk/static/uploaded
```

### Step 2: Run Deployment
Execute the deployment script:
```bash
python3 deploy/deploy.py root@novk.localplayer.dev
```

The script automatically executes the following pipeline:
1. Verifies and creates remote directories (`/opt/novk`, `/mnt/novk-data/...`).
2. Backs up remote database and uploads local `deploy/novk.sqlite` or `deploy/audio/` (if present).
3. Compiles the release binary locally (`cargo build --release`).
4. Stops the running remote service (`systemctl stop novk.service`).
5. Uploads `deploy/.env` (`chmod 600`), unit file `deploy/novk.service`, binary (`chmod 755`), configs, and assets to `/opt/novk`.
6. Reloads `systemd`, starts `novk.service`, and outputs service status.

---

## 5. Caddy Configuration

Caddy handles HTTPS and forwards all traffic to the application:

```caddyfile
novk.localplayer.dev {
	reverse_proxy 172.18.0.1:8000
}
```

> **Network Binding & Security:**  
> The service binds to `172.18.0.1` (the internal Docker bridge gateway IP). Because Caddy runs inside a container, it connects to the host service via `172.18.0.1:8000`. This enables reverse proxying from Caddy while keeping port `8000` completely closed to the public internet (external interfaces do not listen on port 8000).
