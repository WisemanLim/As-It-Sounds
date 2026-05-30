# As It Sounds — Web 배포

> 🌐 **한국어** · [English](README-Web.en.md)

> 데스크탑 Tauri 앱과 **동일한 Rust 코어**(whisper-rs + ffmpeg + 다국어 한글 음차 + 번역)를 그대로 재사용하는 웹앱 배포 가이드.
> nginx-proxy + acme-companion 로 Let's Encrypt 인증서를 자동 발급/갱신하며, `docker compose up -d` 한 줄로 띄울 수 있습니다.

---

## 목차

1. [구성 개요](#구성-개요)
2. [시스템 요구사항](#시스템-요구사항)
3. [빠른 시작 (5분 배포)](#빠른-시작-5분-배포)
4. [디렉터리 / 파일 구조](#디렉터리--파일-구조)
5. [환경 변수](#환경-변수)
6. [빌드](#빌드)
7. [배포](#배포)
8. [운영](#운영)
9. [TLS / 도메인](#tls--도메인)
10. [업로드 / 미디어 처리](#업로드--미디어-처리)
11. [데스크탑 코드와의 관계](#데스크탑-코드와의-관계)
12. [트러블슈팅](#트러블슈팅)
13. [로컬 개발(컨테이너 없이)](#로컬-개발컨테이너-없이)
14. [보안 / 인증 메모](#보안--인증-메모)

---

## 구성 개요

```
                ┌────────────────────────────────────────────────┐
public 80/443   │   nginx-proxy  ──┐                            │
                │                  │  VIRTUAL_HOST 라우팅       │
                │   acme-companion │  + Let's Encrypt 자동 발급 │
                │                  ▼                            │
                │            ┌──────────────┐                   │
                │            │  frontend    │  React/Vite SPA   │
                │            │  (nginx)     │  + /api 리버스    │
                │            └──────┬───────┘                   │
                │                   │  http://backend:8080      │
                │            ┌──────▼───────┐                   │
                │            │  backend     │  axum + whisper   │
                │            │  (Rust)      │  + ffmpeg + sqlite│
                │            └──────────────┘                   │
                └────────────────────────────────────────────────┘
```

- **백엔드 (`backend`)** — `web-server/` 크레이트. axum 0.7 기반 HTTP/SSE 서버. 데스크탑의 `src-tauri/src/core/*` 를 `#[path]` 로 그대로 재사용 (Whisper ASR, ffmpeg 디코드, 음차, 번역, SQLite). 이벤트는 `tokio::sync::broadcast` 채널을 `/api/events` SSE 로 송출.
- **프론트엔드 (`frontend`)** — 기존 React 19 + Vite 빌드를 그대로 사용. 런타임에 `window.__TAURI_INTERNALS__` 존재 여부로 데스크탑/웹을 감지해 IPC 어댑터를 스위치 ([`src/ipc/client.ts`](src/ipc/client.ts)). 웹 모드에서는 REST + SSE + 멀티파트 업로드.
- **TLS** — `nginxproxy/nginx-proxy` 가 라벨 (`VIRTUAL_HOST`, `LETSENCRYPT_HOST`) 기반으로 가상 호스트를 생성하고, `acme-companion` 이 HTTP-01 챌린지로 인증서를 발급·갱신.

---

## 시스템 요구사항

| 항목 | 최소 | 권장 |
| :-- | :-- | :-- |
| OS | Linux x86_64 / arm64 (Docker 가능한 어떤 것이든) | Ubuntu 22.04 LTS |
| CPU | 2 vCPU | 4 vCPU + AVX2 (Whisper inference) |
| RAM | 4 GB | 8 GB (large 모델 사용 시 16 GB) |
| 디스크 | 5 GB (base 모델 + SQLite + 업로드 일부) | 50 GB+ (long-running) |
| 네트워크 | 80/443 인바운드 | 도메인 1개, 정방향 DNS A/AAAA |
| Docker | 24.x + `docker compose` v2 | 최신 |

> Apple Silicon 호스트에서 `linux/amd64` 이미지로 빌드하면 qemu 에뮬레이션으로 동작하지만 Whisper 추론이 매우 느립니다. 가능하면 `linux/arm64` (Apple Silicon, AWS Graviton) 또는 native x86_64 에서 빌드/실행하세요.

---

## 빠른 시작

두 가지 모드. 도메인이 있으면 **공개 배포**, 없으면 **로컬/사설망**.

### 1) 로컬 (localhost, HTTPS 없음)

```bash
git clone https://github.com/WisemanLim/As-It-Sounds.git
cd As-It-Sounds

# nginx-proxy / acme 빼고 frontend 만 호스트 :8080 에 노출
docker compose -f docker-compose.local.yml up -d --build

open http://localhost:8080
```

> Apple Silicon 호스트 기본은 `linux/amd64` (Rosetta 에뮬). arm64 네이티브로 빌드하고 싶으면 `BACKEND_PLATFORM=linux/arm64` 를 `.env` 또는 명령행 변수로 지정.

### 2) 공개 배포 (도메인 + Let's Encrypt HTTPS)

```bash
cp .env.example .env
$EDITOR .env   # DOMAIN, LETSENCRYPT_EMAIL 수정

# DNS 가 서버 IP 로 가리키도록 먼저 설정 (HTTP-01 챌린지에 필요)
docker compose up -d --build

# 인증서 발급은 1~2분
docker compose logs -f acme nginx-proxy backend
```

브라우저에서 `https://<DOMAIN>` 접속 → 음성/영상 파일을 끌어다 놓으면 끝. 첫 변환 시 Whisper `base` 모델(≈142 MB)이 자동 다운로드되어 `ais-data` 볼륨에 캐시됩니다.

> `docker-compose.yml` 만 쓰고 `.env` 의 `DOMAIN` 이 비어 있거나 `localhost` 면 nginx-proxy 가 라우팅할 vhost 를 못 찾아 **503 Service Temporarily Unavailable** 이 나옵니다. 로컬은 위의 1) 방식을 사용하세요.

---

## 디렉터리 / 파일 구조

```
.
├── docker-compose.yml          # nginx-proxy + acme + backend + frontend
├── .env.example                # DOMAIN, LETSENCRYPT_EMAIL 템플릿
├── Dockerfile.web              # 백엔드 (Rust → debian-slim + ffmpeg)
├── Dockerfile.web-frontend     # 프론트엔드 (pnpm build → nginx alpine)
├── deploy/
│   └── nginx.app.conf          # 프론트 컨테이너 내부 nginx 설정
├── web-server/                 # 새 axum 크레이트 (코어 재사용)
│   ├── Cargo.toml
│   └── src/
│       ├── main.rs             # axum 라우터, REST/SSE 핸들러
│       ├── pipeline.rs         # 새 파이프라인 (broadcast 이벤트)
│       └── core.rs             # #[path]로 src-tauri/src/core/* 재포함
├── src/                        # React 프론트엔드 (기존 + IPC 스위처)
│   ├── lib/env.ts              # isTauri 런타임 감지
│   ├── ipc/
│   │   ├── client.ts           # 스위처 (tauri ↔ web)
│   │   ├── tauriClient.ts      # 데스크탑 (invoke / listen)
│   │   └── webClient.ts        # 웹 (fetch + EventSource + multipart)
│   └── ...
└── src-tauri/                  # 기존 데스크탑 (변경 없음)
    └── src/core/               # 백엔드/데스크탑 공통 코어
```

---

## 환경 변수

`.env` 에 정의되어 `docker compose` 가 자동 로드합니다.

| 변수 | 기본값 | 용도 |
| :-- | :-- | :-- |
| `DOMAIN` | — (필수) | 공개 도메인. nginx-proxy 의 `VIRTUAL_HOST`, acme 의 `LETSENCRYPT_HOST` |
| `LETSENCRYPT_EMAIL` | — (필수) | Let's Encrypt 갱신 알림 메일 |
| `CLIENT_MAX_BODY_SIZE` | `4096m` | nginx-proxy(에지) 최대 업로드 크기 |
| `AIS_MAX_UPLOAD_MB` | `2048` | axum 백엔드 최대 multipart body (MB) |
| `RUST_LOG` | `info` | 백엔드 로그 레벨 (`info`/`debug`/`warn`) |

백엔드 컨테이너 내부에서 추가로:

| 변수 | 기본값 | 용도 |
| :-- | :-- | :-- |
| `AIS_DATA_DIR` | `/data` | SQLite · Whisper 모델 · 업로드 보관 디렉터리 |
| `AIS_BIND` | `0.0.0.0:8080` | 리스닝 주소 |

---

## 빌드

```bash
# 두 이미지 빌드
docker compose build

# 백엔드만
docker compose build backend

# 프론트엔드만
docker compose build frontend

# 캐시 무시
docker compose build --no-cache backend
```

### 백엔드 (`Dockerfile.web`)

- 빌드 스테이지: `rust:1.82-bookworm` + `cmake`/`clang` (whisper.cpp 빌드용)
- 런타임 스테이지: `debian:bookworm-slim` + `ffmpeg` + `ca-certificates` + `libgomp1`
- BuildKit 캐시 마운트로 `cargo` 레지스트리 / `target/` 을 재사용
- 결과 바이너리: `/usr/local/bin/as-it-sounds-web`

> **첫 빌드는 10–25분** (whisper.cpp 컴파일 포함). 이후 캐시되면 1분 이내.

### 프론트엔드 (`Dockerfile.web-frontend`)

- `node:20-bookworm-slim` + `pnpm@10.33` (corepack)
- `pnpm install --frozen-lockfile && pnpm build` → 정적 dist
- 런타임: `nginx:1.27-alpine` + [deploy/nginx.app.conf](deploy/nginx.app.conf)
- 빌드 ARG `VITE_AIS_API_BASE` 로 API base 지정 가능 (기본: 빈 문자열 = 같은 오리진)

---

## 배포

```bash
# 최초 기동
docker compose up -d --build

# 상태 확인
docker compose ps

# 인증서 발급 진행 확인
docker compose logs -f acme

# 백엔드 / 프론트엔드 / 프록시 분리 모니터링
docker compose logs -f backend
docker compose logs -f frontend
docker compose logs -f nginx-proxy

# 코드 수정 후 재배포
docker compose up -d --build

# 정지
docker compose stop

# 완전 삭제 (데이터 볼륨 유지)
docker compose down

# 데이터까지 전부 삭제 (주의 — SQLite, Whisper 모델, 업로드, 인증서 전부 날아감)
docker compose down -v
```

---

## 운영

### 데이터 위치 (Docker 볼륨)

| 볼륨 | 컨테이너 경로 | 내용 |
| :-- | :-- | :-- |
| `ais-data` | `backend:/data` | `as-it-sounds.sqlite`, `models/`, `uploads/` |
| `certs` | `*:/etc/nginx/certs` | Let's Encrypt 인증서 |
| `vhost` | `*:/etc/nginx/vhost.d` | nginx-proxy 동적 vhost |
| `html` | `*:/usr/share/nginx/html` | ACME HTTP-01 챌린지 파일 |
| `acme` | `acme:/etc/acme.sh` | acme.sh 계정/상태 |

볼륨 위치 (호스트):
```bash
docker volume inspect ais-data | grep Mountpoint
# 보통 /var/lib/docker/volumes/<프로젝트>_ais-data/_data
```

### 백업

```bash
# SQLite + 업로드 + 모델 백업
docker run --rm -v ais-data:/data -v "$PWD":/backup \
  alpine tar czf /backup/ais-data-$(date +%F).tgz -C /data .

# 복원
docker run --rm -v ais-data:/data -v "$PWD":/backup \
  alpine sh -c "cd /data && tar xzf /backup/ais-data-2026-05-19.tgz"
```

### Whisper 모델 변경

기본은 `base`. 더 큰 모델을 쓰려면 UI 의 **설정 → Whisper 모델** 에서 변경(`tiny` ~ `large-v3`). 모델 파일은 백엔드의 `/data/models/ggml-<name>.bin` 으로 자동 다운로드됩니다.

수동 배치(에어갭 환경):

```bash
docker compose exec backend mkdir -p /data/models
docker cp ggml-medium.bin ais-backend:/data/models/ggml-medium.bin
```

### 로그

```bash
# 전체
docker compose logs -f

# 백엔드 trace 레벨
docker compose run --rm -e RUST_LOG=debug backend

# 영구 변경: .env 의 RUST_LOG=debug 로 수정 후
docker compose up -d backend
```

---

## TLS / 도메인

1. **DNS** — `DOMAIN` 이 서버의 공개 IP 로 `A` (또는 `AAAA`) 레코드를 가리키도록 설정.
2. **포트** — 호스트의 `80`, `443` 이 비어 있어야 함 (다른 nginx 등이 점유하면 충돌).
3. **방화벽** — `ufw allow 80/tcp && ufw allow 443/tcp` 등.
4. **이메일** — `LETSENCRYPT_EMAIL` 은 실제 받을 수 있는 주소 권장 (만료 30일 전 알림).
5. **첫 발급** — `docker compose up -d` 후 30초~수분 내 자동. `docker compose logs acme` 로 `Creating/renewal of <DOMAIN> ...` 확인.
6. **갱신** — acme-companion 이 매시간 만료 30일 미만 인증서 자동 갱신.

### 스테이징(테스트) 인증서 (rate limit 회피)

`.env` 에 추가:
```env
ACME_CA_URI=https://acme-staging-v02.api.letsencrypt.org/directory
```
정식 운영 직전에 다시 제거 → `docker compose up -d acme` 로 재발급.

### 와일드카드 / DNS-01

기본은 HTTP-01 (도메인 1개). 와일드카드 / 다중 SAN 은 acme-companion 의 [DNS-01 가이드](https://github.com/nginx-proxy/acme-companion/blob/main/docs/Let's-Encrypt-and-ACME.md) 참고.

---

## 업로드 / 미디어 처리

### 흐름

```
브라우저
  ├─▶ <input type="file"> 또는 HTML5 drag-drop
  │      → multipart POST /api/projects/:id/files
  │
백엔드 axum
  ├─▶ 각 파트를 /data/uploads/<uuid>.<ext> 로 저장
  ├─▶ media_files DB row 생성 (path = 절대경로)
  └─▶ MediaFile[] JSON 반환

브라우저
  └─▶ POST /api/pipeline/run { fileIds }

백엔드 (백그라운드 tokio task)
  ├─▶ Whisper 모델 확인/다운로드 → SSE: modelDownloading
  ├─▶ ffmpeg 디코드 (16kHz mono f32)  → SSE: progress decode
  ├─▶ Whisper 추론                     → SSE: progress asr
  ├─▶ translit + 번역                  → SSE: progress translate
  └─▶ 세그먼트별                       → SSE: segment, fileDone
```

### 큰 파일 업로드

기본 한도:
- nginx-proxy (에지): `CLIENT_MAX_BODY_SIZE=4096m`
- 컨테이너 nginx (프론트): `client_max_body_size 4096m;` ([deploy/nginx.app.conf](deploy/nginx.app.conf))
- axum: `AIS_MAX_UPLOAD_MB=2048` → `DefaultBodyLimit::max(...)`

세 곳 모두 늘려야 합니다. 4 GB 넘는 단일 업로드는 분할/리샘플링 권장.

### 미디어 재생

`<video>`/`<audio>` 는 백엔드의 `GET /api/files/:id/media` 를 가리키며, `tower_http::services::ServeFile` 이 `Range` 헤더를 처리하므로 탐색(seek)이 정상 동작합니다.

---

## 데스크탑 코드와의 관계

핵심은 **데스크탑 코드를 단 한 줄도 옮기지 않고** 재사용합니다.

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

`pipeline.rs` 만 새로 작성 — 데스크탑은 `tauri::AppHandle::emit` 으로 이벤트를 보내지만, 웹은 `tokio::sync::broadcast::Sender<PipelineEvent>` 로 보내고 SSE 핸들러가 구독합니다. 그 외 ASR/ffmpeg/SQLite/번역/음차 로직은 완전히 동일.

프론트엔드도 동일 코드 베이스:

```ts
// src/ipc/client.ts
export const ipc = isTauri ? tauriIpc : webIpc;
```

| 호출 | 데스크탑 | 웹 |
| :-- | :-- | :-- |
| `ipc.projectList()` | `invoke("project_list")` | `GET /api/projects` |
| `ipc.fileAddMany(...)` | `invoke("file_add_many")` | (사용 안 함) |
| `ipc.fileUpload(...)` | (사용 안 함) | `POST /api/projects/:id/files` (multipart) |
| `ipc.pipelineRun(...)` | `invoke("pipeline_run")` | `POST /api/pipeline/run` |
| `onPipelineEvent(...)` | Tauri `listen()` | `EventSource('/api/events')` |
| `ipc.mediaSrc(file)` | `convertFileSrc(path)` (asset proto) | `GET /api/files/:id/media` |

---

## 트러블슈팅

| 증상 | 원인 / 해결 |
| :-- | :-- |
| `acme` 가 `Connection refused` 로 챌린지 실패 | DNS 가 아직 전파 안 됨, 또는 80 포트가 막혀 있음. `dig +short $DOMAIN`, `curl -I http://$DOMAIN/.well-known/acme-challenge/test` 로 확인 |
| `502 Bad Gateway` | `backend` 컨테이너가 죽었거나 부팅 중 → `docker compose logs backend` |
| `cmake: not found` 류 빌드 실패 | 백엔드 이미지 빌드용. `Dockerfile.web` 에 cmake 가 들어 있음 — `--no-cache` 로 재빌드 |
| `Whisper 모델 다운로드 실패` | 컨테이너에서 huggingface.co 차단? 프록시 환경이면 `HTTPS_PROXY` 를 backend 서비스 environment 에 추가 |
| `payload too large` | 세 곳 모두 늘려야 함: `.env` 의 `CLIENT_MAX_BODY_SIZE`/`AIS_MAX_UPLOAD_MB`, 프론트 nginx 의 `client_max_body_size` |
| SSE 연결이 30초 만에 끊김 | nginx 의 `proxy_read_timeout`/`proxy_buffering off` 확인 — [deploy/nginx.app.conf](deploy/nginx.app.conf) 가 24h 로 설정되어 있음 |
| `ERR_CONNECTION_REFUSED` (http://) | http→https 리디렉트는 nginx-proxy 가 자동 처리. 인증서 발급이 완료된 후 자동으로 적용됨. acme 로그 확인 |
| Apple Silicon 빌드 매우 느림 | `linux/amd64` qemu 에뮬레이션. 호스트가 arm64 이면 그대로 (Tauri docker BuildKit 은 자동으로 native arch 선택) |
| Whisper 추론이 매우 느림 | `tiny` 모델로 시작 → CPU AVX2 / 코어 수 부족이면 모델을 줄이거나 GPU 호스트 (별도 빌드 필요) |
| `MyMemory` 번역 빈 결과 | 일일 1000 단어 한도 초과 — UI 설정에서 LibreTranslate 셀프호스트로 전환 |

---

## 로컬 개발(컨테이너 없이)

### 백엔드만 (호스트 ffmpeg/cmake 필요)

```bash
brew install ffmpeg cmake          # macOS
sudo apt install -y ffmpeg cmake build-essential pkg-config clang  # Ubuntu

cd web-server
AIS_DATA_DIR=./.dev-data RUST_LOG=info cargo run --release
# → 0.0.0.0:8080 에 axum 기동
```

### 프론트엔드만 (개발 서버)

```bash
VITE_AIS_API_BASE=http://localhost:8080 pnpm dev
# → http://localhost:1420 (Vite). 브라우저에서 열면 web 모드로 동작
```

VITE 환경변수는 빌드 시점에 정적 치환됩니다. 같은 빌드를 컨테이너에 배포하려면 `VITE_AIS_API_BASE` 를 비워두세요(같은 오리진 = nginx 가 `/api` 를 backend 로 프록시).

### 데스크탑은 그대로

웹 작업 중에도 `pnpm tauri:dev` 는 변함없이 작동합니다. 두 모드는 같은 `src/` 를 공유하며 `isTauri` 로 분기될 뿐입니다.

---

## 보안 / 인증 메모

본 배포는 **단일 인스턴스 / 인증 없음** 기본형입니다 (`/api/*` 가 누구에게나 열려 있음).
- 사설망/VPN 뒤에서만 노출하거나
- nginx-proxy 에 HTTP Basic Auth 추가 (`/etc/nginx/htpasswd/<DOMAIN>` 파일 생성 후 `BASIC_AUTH=true` 환경변수)
- 또는 Cloudflare Access / Tailscale Funnel / Authelia 등 외부 인증 레이어 전치

추후 사용자 계정 기반 격리가 필요해지면 `web-server/` 에 미들웨어 + DB 컬럼(user_id) 추가가 자연스러운 진화 경로입니다.

---

## 라이선스 / 출처

상위 [README.md](README.md) 와 동일 (MIT). 추가 외부 컴포넌트:
- [nginxproxy/nginx-proxy](https://github.com/nginx-proxy/nginx-proxy) — MIT
- [nginxproxy/acme-companion](https://github.com/nginx-proxy/acme-companion) — MIT
- [axum](https://github.com/tokio-rs/axum), [tower-http](https://github.com/tower-rs/tower-http) — MIT
