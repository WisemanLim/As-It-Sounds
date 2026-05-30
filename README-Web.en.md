# As It Sounds — Web Deployment

> 🌐 [한국어](README-Web.md) · **English**

> A web deployment guide that reuses the **same Rust core** (whisper-rs + ffmpeg + multilingual Korean romanization + translation) as the desktop Tauri app without any changes.
> nginx-proxy + acme-companion automatically issues and renews Let's Encrypt certificates, and the entire stack starts with a single `docker compose up -d`.

---

## Table of Contents

1. [Architecture Overview](#architecture-overview)
2. [System Requirements](#system-requirements)
3. [Quick Start (5-minute deploy)](#quick-start-5-minute-deploy)
4. [Directory / File Structure](#directory--file-structure)
5. [Environment Variables](#environment-variables)
6. [Build](#build)
7. [Deploy](#deploy)
8. [Operations](#operations)
9. [TLS / Domain](#tls--domain)
10. [Upload / Media Processing](#upload--media-processing)
11. [Relationship with Desktop Code](#relationship-with-desktop-code)
12. [Troubleshooting](#troubleshooting)
13. [Local Development (without containers)](#local-development-without-containers)
14. [Security / Auth Notes](#security--auth-notes)

---

## Architecture Overview

```
                ┌────────────────────────────────────────────────┐
public 80/443   │   nginx-proxy  ──┐                            │
                │                  │  VIRTUAL_HOST routing      │
                │   acme-companion │  + Let's Encrypt auto-cert │
                │                  ▼                            │
                │            ┌──────────────┐                   │
                │            │  frontend    │  React/Vite SPA   │
                │            │  (nginx)     │  + /api reverse   │
                │            └──────┬───────┘                   │
                │                   │  http://backend:8080      │
                │            ┌──────▼───────┐                   │
                │            │  backend     │  axum + whisper   │
                │            │  (Rust)      │  + ffmpeg + sqlite│
                │            └──────────────┘                   │
                └────────────────────────────────────────────────┘
```

- **Backend (`backend`)** — The `web-server/` crate. An HTTP/SSE server based on axum 0.7. Reuses `src-tauri/src/core/*` from the desktop app as-is via `#[path]` (Whisper ASR, ffmpeg decode, romanization, translation, SQLite). Events are emitted over a `tokio::sync::broadcast` channel to `/api/events` as SSE.
- **Frontend (`frontend`)** — Uses the existing React 19 + Vite build unchanged. Detects desktop vs. web at runtime by checking for `window.__TAURI_INTERNALS__` and switches the IPC adapter accordingly ([`src/ipc/client.ts`](src/ipc/client.ts)). In web mode: REST + SSE + multipart upload.
- **TLS** — `nginxproxy/nginx-proxy` creates virtual hosts based on labels (`VIRTUAL_HOST`, `LETSENCRYPT_HOST`), and `acme-companion` issues and renews certificates via HTTP-01 challenge.

---

## System Requirements

| Item | Minimum | Recommended |
| :-- | :-- | :-- |
| OS | Linux x86_64 / arm64 (any Docker-capable host) | Ubuntu 22.04 LTS |
| CPU | 2 vCPU | 4 vCPU + AVX2 (Whisper inference) |
| RAM | 4 GB | 8 GB (16 GB when using the large model) |
| Disk | 5 GB (base model + SQLite + some uploads) | 50 GB+ (long-running) |
| Network | 80/443 inbound | 1 domain, forward DNS A/AAAA |
| Docker | 24.x + `docker compose` v2 | Latest |

> Building with a `linux/amd64` image on an Apple Silicon host works via qemu emulation, but Whisper inference will be very slow. Where possible, build and run on `linux/arm64` (Apple Silicon, AWS Graviton) or native x86_64.

---

## Quick Start

Two modes: **public deploy** if you have a domain, **local/private network** if you don't.

### 1) Local (localhost, no HTTPS)

```bash
git clone https://github.com/WisemanLim/As-It-Sounds.git
cd As-It-Sounds

# Expose only the frontend on host :8080, without nginx-proxy / acme
docker compose -f docker-compose.local.yml up -d --build

open http://localhost:8080
```

> The default on Apple Silicon hosts is `linux/amd64` (Rosetta emulation). To build arm64 native, set `BACKEND_PLATFORM=linux/arm64` as an `.env` or command-line variable.

### 2) Public Deploy (domain + Let's Encrypt HTTPS)

```bash
cp .env.example .env
$EDITOR .env   # set DOMAIN and LETSENCRYPT_EMAIL

# First configure DNS to point to your server IP (required for HTTP-01 challenge)
docker compose up -d --build

# Certificate issuance takes 1–2 minutes
docker compose logs -f acme nginx-proxy backend
```

Open `https://<DOMAIN>` in a browser → drag and drop an audio/video file to get started. On the first transcription, the Whisper `base` model (≈142 MB) is automatically downloaded and cached in the `ais-data` volume.

> If you use only `docker-compose.yml` with `DOMAIN` empty or set to `localhost` in `.env`, nginx-proxy cannot find a vhost to route to and returns **503 Service Temporarily Unavailable**. Use mode 1) above for local usage.

---

## Directory / File Structure

```
.
├── docker-compose.yml          # nginx-proxy + acme + backend + frontend
├── .env.example                # DOMAIN, LETSENCRYPT_EMAIL template
├── Dockerfile.web              # backend (Rust → debian-slim + ffmpeg)
├── Dockerfile.web-frontend     # frontend (pnpm build → nginx alpine)
├── deploy/
│   └── nginx.app.conf          # nginx config inside the frontend container
├── web-server/                 # new axum crate (reuses core)
│   ├── Cargo.toml
│   └── src/
│       ├── main.rs             # axum router, REST/SSE handlers
│       ├── pipeline.rs         # new pipeline (broadcast events)
│       └── core.rs             # re-includes src-tauri/src/core/* via #[path]
├── src/                        # React frontend (existing + IPC switcher)
│   ├── lib/env.ts              # isTauri runtime detection
│   ├── ipc/
│   │   ├── client.ts           # switcher (tauri ↔ web)
│   │   ├── tauriClient.ts      # desktop (invoke / listen)
│   │   └── webClient.ts        # web (fetch + EventSource + multipart)
│   └── ...
└── src-tauri/                  # existing desktop code (unchanged)
    └── src/core/               # shared core for backend and desktop
```

---

## Environment Variables

Defined in `.env` and automatically loaded by `docker compose`.

| Variable | Default | Purpose |
| :-- | :-- | :-- |
| `DOMAIN` | — (required) | Public domain. Used as `VIRTUAL_HOST` for nginx-proxy and `LETSENCRYPT_HOST` for acme |
| `LETSENCRYPT_EMAIL` | — (required) | Email for Let's Encrypt renewal notifications |
| `CLIENT_MAX_BODY_SIZE` | `4096m` | Maximum upload size at the nginx-proxy edge |
| `AIS_MAX_UPLOAD_MB` | `2048` | Maximum multipart body size for the axum backend (MB) |
| `RUST_LOG` | `info` | Backend log level (`info`/`debug`/`warn`) |

Additional variables inside the backend container:

| Variable | Default | Purpose |
| :-- | :-- | :-- |
| `AIS_DATA_DIR` | `/data` | Directory for SQLite, Whisper models, and uploads |
| `AIS_BIND` | `0.0.0.0:8080` | Listening address |

---

## Build

```bash
# Build both images
docker compose build

# Backend only
docker compose build backend

# Frontend only
docker compose build frontend

# Ignore cache
docker compose build --no-cache backend
```

### Backend (`Dockerfile.web`)

- Build stage: `rust:1.82-bookworm` + `cmake`/`clang` (for whisper.cpp compilation)
- Runtime stage: `debian:bookworm-slim` + `ffmpeg` + `ca-certificates` + `libgomp1`
- BuildKit cache mounts reuse the `cargo` registry and `target/`
- Output binary: `/usr/local/bin/as-it-sounds-web`

> **The first build takes 10–25 minutes** (includes whisper.cpp compilation). Subsequent builds with cache complete in under a minute.

### Frontend (`Dockerfile.web-frontend`)

- `node:20-bookworm-slim` + `pnpm@10.33` (corepack)
- `pnpm install --frozen-lockfile && pnpm build` → static dist
- Runtime: `nginx:1.27-alpine` + [deploy/nginx.app.conf](deploy/nginx.app.conf)
- Build ARG `VITE_AIS_API_BASE` can specify the API base URL (default: empty string = same origin)

---

## Deploy

```bash
# Initial startup
docker compose up -d --build

# Check status
docker compose ps

# Monitor certificate issuance
docker compose logs -f acme

# Monitor backend / frontend / proxy separately
docker compose logs -f backend
docker compose logs -f frontend
docker compose logs -f nginx-proxy

# Redeploy after code changes
docker compose up -d --build

# Stop
docker compose stop

# Full removal (keeps data volumes)
docker compose down

# Remove everything including data (WARNING — deletes SQLite, Whisper models, uploads, and certificates)
docker compose down -v
```

---

## Operations

### Data Locations (Docker Volumes)

| Volume | Container Path | Contents |
| :-- | :-- | :-- |
| `ais-data` | `backend:/data` | `as-it-sounds.sqlite`, `models/`, `uploads/` |
| `certs` | `*:/etc/nginx/certs` | Let's Encrypt certificates |
| `vhost` | `*:/etc/nginx/vhost.d` | nginx-proxy dynamic vhosts |
| `html` | `*:/usr/share/nginx/html` | ACME HTTP-01 challenge files |
| `acme` | `acme:/etc/acme.sh` | acme.sh account and state |

Volume location on host:
```bash
docker volume inspect ais-data | grep Mountpoint
# typically /var/lib/docker/volumes/<project>_ais-data/_data
```

### Backup

```bash
# Backup SQLite + uploads + models
docker run --rm -v ais-data:/data -v "$PWD":/backup \
  alpine tar czf /backup/ais-data-$(date +%F).tgz -C /data .

# Restore
docker run --rm -v ais-data:/data -v "$PWD":/backup \
  alpine sh -c "cd /data && tar xzf /backup/ais-data-2026-05-19.tgz"
```

### Changing the Whisper Model

The default is `base`. To use a larger model, change it in the UI under **Settings → Whisper Model** (`tiny` through `large-v3`). The model file is automatically downloaded to `/data/models/ggml-<name>.bin` on the backend.

Manual placement (air-gapped environments):

```bash
docker compose exec backend mkdir -p /data/models
docker cp ggml-medium.bin ais-backend:/data/models/ggml-medium.bin
```

### Logs

```bash
# All services
docker compose logs -f

# Backend at trace level
docker compose run --rm -e RUST_LOG=debug backend

# Persistent change: edit RUST_LOG=debug in .env, then
docker compose up -d backend
```

---

## TLS / Domain

1. **DNS** — Configure an `A` (or `AAAA`) record for `DOMAIN` pointing to the server's public IP.
2. **Ports** — Host ports `80` and `443` must be free (conflicts occur if another nginx or similar service is already using them).
3. **Firewall** — e.g. `ufw allow 80/tcp && ufw allow 443/tcp`.
4. **Email** — `LETSENCRYPT_EMAIL` should be a real address you can receive (expiry notifications are sent 30 days before expiration).
5. **First issuance** — Happens automatically within 30 seconds to a few minutes after `docker compose up -d`. Confirm with `docker compose logs acme` — look for `Creating/renewal of <DOMAIN> ...`.
6. **Renewal** — acme-companion automatically renews certificates expiring in less than 30 days, checked every hour.

### Staging (test) Certificates (to avoid rate limits)

Add to `.env`:
```env
ACME_CA_URI=https://acme-staging-v02.api.letsencrypt.org/directory
```
Remove it just before going to production → re-issue with `docker compose up -d acme`.

### Wildcard / DNS-01

The default is HTTP-01 (single domain). For wildcard or multi-SAN certificates, refer to the acme-companion [DNS-01 guide](https://github.com/nginx-proxy/acme-companion/blob/main/docs/Let's-Encrypt-and-ACME.md).

---

## Upload / Media Processing

### Flow

```
Browser
  ├─▶ <input type="file"> or HTML5 drag-drop
  │      → multipart POST /api/projects/:id/files
  │
Backend axum
  ├─▶ saves each part to /data/uploads/<uuid>.<ext>
  ├─▶ creates media_files DB row (path = absolute path)
  └─▶ returns MediaFile[] JSON

Browser
  └─▶ POST /api/pipeline/run { fileIds }

Backend (background tokio task)
  ├─▶ check/download Whisper model  → SSE: modelDownloading
  ├─▶ ffmpeg decode (16kHz mono f32) → SSE: progress decode
  ├─▶ Whisper inference              → SSE: progress asr
  ├─▶ translit + translation         → SSE: progress translate
  └─▶ per segment                    → SSE: segment, fileDone
```

### Large File Uploads

Default limits:
- nginx-proxy (edge): `CLIENT_MAX_BODY_SIZE=4096m`
- Container nginx (frontend): `client_max_body_size 4096m;` ([deploy/nginx.app.conf](deploy/nginx.app.conf))
- axum: `AIS_MAX_UPLOAD_MB=2048` → `DefaultBodyLimit::max(...)`

All three must be increased together. For single uploads over 4 GB, splitting or resampling is recommended.

### Media Playback

`<video>`/`<audio>` elements point to `GET /api/files/:id/media` on the backend. `tower_http::services::ServeFile` handles `Range` headers, so seeking works correctly.

---

## Relationship with Desktop Code

The key principle is reusing the **desktop code without moving a single line**.

```rust
// web-server/src/core.rs
#[path = "../../src-tauri/src/core/error.rs"]   pub mod error;
#[path = "../../src-tauri/src/core/models.rs"]  pub mod models;
#[path = "../../src-tauri/src/core/translit.rs"] pub mod translit;
#[path = "../../src-tauri/src/core/translate.rs"] pub mod translate;
#[path = "../../src-tauri/src/core/db.rs"]      pub mod db;
#[path = "../../src-tauri/src/core/asr.rs"]     pub mod asr;
#[path = "../../src-tauri/src/core/media.rs"]   pub mod media;
#[path = "../../src-tauri/src/core/settings.rs"] pub mod settings;
```

Only `pipeline.rs` is written from scratch — the desktop sends events via `tauri::AppHandle::emit`, while the web sends them via `tokio::sync::broadcast::Sender<PipelineEvent>` and the SSE handler subscribes to them. All other ASR/ffmpeg/SQLite/translation/romanization logic is identical.

The frontend also shares the same codebase:

```ts
// src/ipc/client.ts
export const ipc = isTauri ? tauriIpc : webIpc;
```

| Call | Desktop | Web |
| :-- | :-- | :-- |
| `ipc.projectList()` | `invoke("project_list")` | `GET /api/projects` |
| `ipc.fileAddMany(...)` | `invoke("file_add_many")` | (not used) |
| `ipc.fileUpload(...)` | (not used) | `POST /api/projects/:id/files` (multipart) |
| `ipc.pipelineRun(...)` | `invoke("pipeline_run")` | `POST /api/pipeline/run` |
| `onPipelineEvent(...)` | Tauri `listen()` | `EventSource('/api/events')` |
| `ipc.mediaSrc(file)` | `convertFileSrc(path)` (asset proto) | `GET /api/files/:id/media` |

---

## Troubleshooting

| Symptom | Cause / Fix |
| :-- | :-- |
| `acme` challenge fails with `Connection refused` | DNS not yet propagated, or port 80 is blocked. Check with `dig +short $DOMAIN` and `curl -I http://$DOMAIN/.well-known/acme-challenge/test` |
| `502 Bad Gateway` | The `backend` container has crashed or is still starting → `docker compose logs backend` |
| Build fails with `cmake: not found` | Needed for the backend image build. cmake is included in `Dockerfile.web` — rebuild with `--no-cache` |
| `Whisper model download failed` | huggingface.co blocked from the container? Add `HTTPS_PROXY` to the backend service environment in a proxy setup |
| `payload too large` | All three limits must be raised: `CLIENT_MAX_BODY_SIZE`/`AIS_MAX_UPLOAD_MB` in `.env`, and `client_max_body_size` in the frontend nginx |
| SSE connection drops after 30 seconds | Check `proxy_read_timeout`/`proxy_buffering off` in nginx — [deploy/nginx.app.conf](deploy/nginx.app.conf) is set to 24h |
| `ERR_CONNECTION_REFUSED` (http://) | http→https redirect is handled automatically by nginx-proxy. It takes effect once certificate issuance completes. Check acme logs |
| Apple Silicon build is very slow | `linux/amd64` qemu emulation. On an arm64 host it runs natively (Tauri docker BuildKit automatically selects the native arch) |
| Whisper inference is very slow | Start with the `tiny` model → if CPU AVX2 / core count is insufficient, use a smaller model or a GPU host (separate build required) |
| `MyMemory` translation returns empty results | Daily 1000-word limit exceeded — switch to a self-hosted LibreTranslate instance in the UI settings |

---

## Local Development (without containers)

### Backend only (requires host ffmpeg/cmake)

```bash
brew install ffmpeg cmake          # macOS
sudo apt install -y ffmpeg cmake build-essential pkg-config clang  # Ubuntu

cd web-server
AIS_DATA_DIR=./.dev-data RUST_LOG=info cargo run --release
# → starts axum on 0.0.0.0:8080
```

### Frontend only (dev server)

```bash
VITE_AIS_API_BASE=http://localhost:8080 pnpm dev
# → http://localhost:1420 (Vite). Opens in web mode in the browser
```

VITE environment variables are statically substituted at build time. To deploy the same build to a container, leave `VITE_AIS_API_BASE` empty (same origin = nginx proxies `/api` to the backend).

### Desktop unchanged

`pnpm tauri:dev` continues to work normally during web development. Both modes share the same `src/` and branch only on `isTauri`.

---

## Security / Auth Notes

This deployment is a **single-instance / no-auth** baseline (`/api/*` is open to anyone).
- Expose only behind a private network/VPN, or
- Add HTTP Basic Auth to nginx-proxy (create `/etc/nginx/htpasswd/<DOMAIN>` and set the `BASIC_AUTH=true` environment variable)
- Or prepend an external auth layer such as Cloudflare Access / Tailscale Funnel / Authelia

When per-user isolation is needed in the future, adding middleware + a `user_id` DB column to `web-server/` is the natural evolution path.

---

## License / Attribution

Same as the parent [README.md](README.md) (MIT). Additional external components:
- [nginxproxy/nginx-proxy](https://github.com/nginx-proxy/nginx-proxy) — MIT
- [nginxproxy/acme-companion](https://github.com/nginx-proxy/acme-companion) — MIT
- [axum](https://github.com/tokio-rs/axum), [tower-http](https://github.com/tower-rs/tower-http) — MIT
