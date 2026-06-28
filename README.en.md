<div align="center">

# As It Sounds

> 🌐 [한국어](README.md) · **English**

**An offline-first desktop app that phonetically transcribes audio/video in any language into Korean Hangul — as it actually sounds — with a Korean translation alongside.**

Tauri 2 · Rust · React 19 · TypeScript · Whisper.cpp

</div>

---

### Overview

**As It Sounds** is a desktop app that transcribes audio/video in **any language** locally with **Whisper** (auto language detection), converts each segment into **Korean phonetic Hangul exactly as it sounds**, and shows the **Korean translation** beside it. No language selection needed — the app detects and handles it automatically. It runs fully offline by default; only translation optionally calls an external HTTP API (MyMemory or LibreTranslate).

> Example — English: `A. I don't think we've received it.` → **에이. 아이 돈트 씽크 위브 리시브드 잇.**
> Example — English: `C. Right around the corner.` → **씨. 라이트 어라운드 더 코너.**
> Example — Japanese: `こんにちは` → **콩니치하**
> Example — French: `bonjour` → **봉주흐**
> Example — Russian: `Москва` → **모스크바**

### Features

- 🎙️ **Fully offline ASR** — Whisper multilingual (tiny/base/small/medium/large) runs locally with **automatic language detection**
- 🔤 **Sound-driven transliteration (not dictionary-driven)** — phonetic rules extract the actual sound, not the word's meaning. Context-aware:
  - Single uppercase letter → letter name (A → 에이, B → 비)
  - Context-sensitive `c` (corner=코너, city=시티)
  - English unstressed `re-`, past-tense `-ed`, long `ei`, `we'` contractions, open-syllable long `a`
  - Japanese kana direct mapping, French nasals & silent finals, Spanish jota, German umlauts, Cyrillic syllable assembly
- 🌐 **Korean translation** — MyMemory (free, key-less) by default; LibreTranslate self-host supported
- 🎧 **Audio & video** — MP3, WAV, M4A, FLAC, OGG, MP4, MOV, MKV, WEBM, AVI
- 🖱️ **Multi-file drag & drop** — OS-level native drops
- ▶️ **Player-linked segment view** — current segment is highlighted as media plays; click a segment to seek
- 💾 **Local SQLite persistence** — projects, files, segments, settings
- 🖥️ **Cross-platform** — macOS (Universal), Windows, Linux
- 🌍 **Web deployment** — same Rust core reused via an axum HTTP/SSE server, self-host with one Docker command → [README-Web.md](README-Web.md)

### Requirements

| Item | Minimum | Recommended |
| :-- | :-- | :-- |
| OS | macOS 11+, Windows 10+, Ubuntu 20.04+ | Latest |
| RAM | 4 GB | 8 GB |
| Disk | 500 MB + Whisper model (75 MB–3 GB) | 5 GB |
| CPU | x86_64 / arm64 | Apple Silicon or AVX2 |
| Network | Only for first-time model download & translation | — |

### CLI test (transliteration accuracy)

Quick pipeline test for verifying phonetic accuracy:

```bash
./test.sh .samples/01_19_04.mp3              # default model: base
./test.sh .samples/01_19_04.mp3 small        # specify model
```

Sample output:
```
시작          종료          원문 (STT)                                  한글 음차
─────────────────────────────────────────────────────────────────────────────
00:00.000   00:04.540    Number 4. I sent the catering order to ...   넘버 4. 아이 센트 더 케이터잉 오더 ...
00:04.540   00:07.700    A. I don't think we've received it.          에이. 아이 돈트 씽크 위브 리시브드 잇.
00:07.700   00:10.440    B. They're out of order.                     비. 데어 아웃 오브 오더.
00:10.440   00:13.140    C. Right around the corner.                  씨. 라이트 어라운드 더 코너.
```

Internally invokes `cargo run --example ais_test` (release) → ffmpeg PCM extract → Whisper inference → translit.

### Full test suite (`test/`)

Scenario-driven verification across core, web, and desktop. See [`test/SCENARIOS.md`](test/SCENARIOS.md); latest run in [`test/results/RESULTS.md`](test/results/RESULTS.md).

| ID | Scope | How | Status |
| :-- | :-- | :-- | :-- |
| D1.1 | Rust core units (translit / asr / media) | `cargo test --manifest-path src-tauri/Cargo.toml --release` | ✅ 27/27 |
| D1.2 | CLI pipeline (STT → Hangul) | `./test.sh .samples/01_19_04.mp3 base` | ✅ pass |
| D2.1–2.3 | Web build + serve | `pnpm typecheck` · `pnpm build` · `pnpm preview` | ✅ pass |
| D2.4 | Web full-stack (browser) | `as-it-sounds-web` (axum `:8080`) + `VITE_AIS_API_BASE` preview, headless render | ✅ pass |
| D3.1 | Desktop bundle | `pnpm tauri:build:mac` (universal) → `.app` + `.dmg` | ✅ pass |
| D3.2 | Desktop launch | open built `.app`, no-crash | ✅ pass |

> Web mode requires the axum `/api` backend (prod serves it behind nginx-proxy). Bare `vite preview` with no backend intentionally shows "초기화에 실패했습니다".

### Quick start (developers)

#### 1. Prerequisites

**macOS**
```bash
# Node / pnpm / cmake / ffmpeg
brew install node pnpm cmake ffmpeg

# Rust — install via rustup (required for cross-compile). Homebrew rust supports only the host target.
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
source "$HOME/.cargo/env"
```

> **If you already ran `brew install rust`** — replace it with rustup to enable cross-compilation (Intel / Windows / Linux):
> ```bash
> brew uninstall rust
> curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
> source "$HOME/.cargo/env"
> ```

**Add cross-compile targets (rustup required)**
```bash
# macOS Universal (ARM + Intel)
rustup target add aarch64-apple-darwin x86_64-apple-darwin

# Windows (MSVC) — needs Windows host or cargo-xwin
rustup target add x86_64-pc-windows-msvc

# Linux — Linux host recommended (webkit2gtk linkage)
rustup target add x86_64-unknown-linux-gnu aarch64-unknown-linux-gnu
```

**Windows (PowerShell as Administrator)**
```powershell
winget install OpenJS.NodeJS.LTS
winget install pnpm.pnpm
winget install Rustlang.Rustup
winget install Kitware.CMake
winget install Gyan.FFmpeg
# Microsoft C++ Build Tools (Visual Studio 2022) are also required
winget install Microsoft.VisualStudio.2022.BuildTools --override "--add Microsoft.VisualStudio.Workload.VCTools --includeRecommended"
# WebView2 Runtime (usually already included with Windows 11)
```

**Linux (Ubuntu/Debian)**
```bash
sudo apt update
sudo apt install -y \
  curl build-essential pkg-config cmake ffmpeg \
  libwebkit2gtk-4.1-dev libappindicator3-dev librsvg2-dev patchelf
curl -fsSL https://get.pnpm.io/install.sh | sh -
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
```

#### 2. Install & run

```bash
pnpm install
pnpm tauri:dev
```

On first run the Whisper `base` multilingual model (≈142 MB) is auto-downloaded from Hugging Face; progress is shown at the top of the project page.

### Usage

1. **Create a project** — *New project* in the sidebar, or *Start empty project* on the Home page.
2. **Add files** — drag & drop or click *Add files* (multi-select supported).
3. **Transcribe** — click *Start* in the header.
4. **Review** — each segment shows the original text, the Korean phonetic transliteration, and the Korean translation. Click a segment to seek the player.

### Settings (⚙️ bottom of sidebar)

- **Whisper model**: tiny (75 MB) … large-v3 (2.9 GB) — prefer multilingual models (no `.en` suffix); bigger = more accurate, slower.
- **Translator**: MyMemory · LibreTranslate · *Disabled*.
- **Translation endpoint**: configurable for self-hosted instances.
- **LibreTranslate API key**: (optional).
- **ffmpeg path**: custom path if ffmpeg is not in `PATH`.

### Data locations

| OS | Path |
| :-- | :-- |
| macOS | `~/Library/Application Support/com.asitsounds.app/` |
| Windows | `%APPDATA%\com.asitsounds.app\` |
| Linux | `~/.local/share/com.asitsounds.app/` |

Layout:
- `as-it-sounds.sqlite` — projects/files/segments/settings
- `models/ggml-*.bin` — downloaded Whisper models

### Production builds

```bash
# Default bundle for the current platform
pnpm tauri:build

# Debug bundle (self-signed, quick preview)
pnpm tauri:build:debug

# macOS Universal (Apple Silicon + Intel)
pnpm tauri:build:mac
# macOS Apple Silicon only
pnpm tauri:build:mac:arm
# macOS Intel only
pnpm tauri:build:mac:intel

# Windows
pnpm tauri:build:win

# Linux
pnpm tauri:build:linux
pnpm tauri:build:linux:arm

# All platforms (macOS Universal + Windows + Linux x64/arm64)
# Note: requires rustup targets installed for each. Win/Linux usually need CI or native host.
pnpm tauri:build:all
```

> **⚠️ Cross-compilation limits (important)**
>
> This project depends on C/C++ native crates (`whisper-rs`/whisper.cpp, `ring`, `rusqlite`). The following is effectively **not supported**:
> - macOS → Windows MSVC (`pnpm tauri:build:win`): C++ header absence breaks `ring`/`whisper-rs-sys`/`libsqlite3-sys`
> - macOS/Windows → Linux: webkit2gtk linkage failure
>
> **Recommended build matrix:**
>
> | Host | Buildable | Command |
> | :-- | :-- | :-- |
> | macOS (Apple Silicon) | macOS ARM, macOS Intel, Universal | `tauri:build:mac:arm` / `:intel` / `:mac` |
> | Windows | Windows x64 | `tauri:build:win` |
> | Linux x86_64 | Linux x64 | `tauri:build:linux` |
> | Linux ARM64 | Linux arm64 | `tauri:build:linux:arm` |
>
> **For all-platform builds use [GitHub Actions CI](.github/workflows/release.yml).** Push a tag (`git tag v0.1.0 && git push --tags`) to trigger auto build/release.

### Multi-platform builds without CI (host-by-host)

Each OS build must run on a **native environment for that OS** (VM, real machine, or container). From a macOS host, only macOS targets build directly; others require one of the methods below.

#### A. Windows x64 build

| Method | Host | Notes |
| :-- | :-- | :-- |
| **Parallels Desktop + Windows 11** | macOS (ARM/Intel) | Most stable. Windows 11 ARM or x64. Requires license |
| **UTM + Windows 11 ARM** | macOS (Apple Silicon) | Free. ISO at [microsoft.com](https://www.microsoft.com/software-download/windows11) |
| **VMware Fusion** | macOS | Free for Personal Use |
| **Boot Camp** | macOS (Intel only) | Not available on Apple Silicon |
| **Physical Windows PC** | Separate box | Fastest |

**Inside the VM (Windows 11, PowerShell as Administrator):**
```powershell
# 1. Package manager (winget built in to Win 11)
winget install OpenJS.NodeJS.LTS pnpm.pnpm Rustlang.Rustup Kitware.CMake Gyan.FFmpeg Git.Git
# 2. Visual Studio C++ Build Tools (required for ring/whisper.cpp/rusqlite)
winget install Microsoft.VisualStudio.2022.BuildTools --override "--add Microsoft.VisualStudio.Workload.VCTools --includeRecommended"
# 3. WebView2 (bundled in Win 11; install only on Win 10)
# winget install Microsoft.EdgeWebView2Runtime
# 4. Reboot, then:
git clone https://github.com/WisemanLim/As-It-Sounds.git
cd As-It-Sounds
pnpm install
pnpm tauri:build:win
```
**Artifacts:** `src-tauri\target\x86_64-pc-windows-msvc\release\bundle\msi\As It Sounds_0.1.0_x64_en-US.msi` and `nsis\*.exe`

#### B. Linux x64 build

| Method | Host | Notes |
| :-- | :-- | :-- |
| **Docker container (recommended)** | Any host | Isolated, reproducible. Requires Docker Desktop or OrbStack |
| **UTM + Ubuntu 22.04 x64** | macOS | qemu emulation (slow on Apple Silicon → x86_64 guest) |
| **Multipass** | Any host | Lightweight Ubuntu VM. `brew install multipass` |
| **Parallels + Ubuntu** | macOS | Fast. License required |
| **Physical Linux box** | Separate | Fastest |

**Docker build (fastest path):**

`Dockerfile.linux-build` (project root):
```dockerfile
FROM ubuntu:22.04
ENV DEBIAN_FRONTEND=noninteractive
RUN apt-get update && apt-get install -y \
    curl build-essential pkg-config cmake ffmpeg git \
    libwebkit2gtk-4.1-dev libappindicator3-dev librsvg2-dev patchelf
# Node 20 + pnpm
RUN curl -fsSL https://deb.nodesource.com/setup_20.x | bash - && apt-get install -y nodejs && \
    npm install -g pnpm@10
# Rust
RUN curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y
ENV PATH="/root/.cargo/bin:${PATH}"
WORKDIR /work
CMD ["bash", "-c", "pnpm install && pnpm tauri:build:linux"]
```

Run:
```bash
docker buildx build --platform linux/amd64 -t ais-linux-build -f Dockerfile.linux-build .
docker run --rm -v "$(pwd)":/work --platform linux/amd64 ais-linux-build
# Artifacts: ./src-tauri/target/x86_64-unknown-linux-gnu/release/bundle/{deb,rpm,appimage}/
```

> On Apple Silicon, an amd64 container runs under qemu emulation → very slow (10–30 min). Prefer a native Linux box or GitHub Codespaces.

#### C. Linux ARM64 build (aarch64)

| Method | Host | Notes |
| :-- | :-- | :-- |
| **Docker buildx (ARM64 container)** | Apple Silicon | **Native speed** (Mac arm64 = Linux arm64). Fastest |
| **UTM + Ubuntu 22.04 arm64** | macOS Apple Silicon | KVM accelerated, near-native |
| **Asahi Linux dual-boot** | Apple Silicon | Best performance. Heavier setup |
| **Raspberry Pi 4/5 (8 GB)** | Separate device | Real hardware. Long compile time (whisper.cpp is heavy) |

**Docker build (Apple Silicon → Linux ARM64, native speed):**
```bash
docker buildx build --platform linux/arm64 -t ais-linux-arm-build -f Dockerfile.linux-build .
docker run --rm -v "$(pwd)":/work --platform linux/arm64 ais-linux-arm-build \
  bash -c "pnpm install && pnpm tauri:build:linux:arm"
# Artifacts: ./src-tauri/target/aarch64-unknown-linux-gnu/release/bundle/{deb,rpm,appimage}/
```

#### D. macOS Intel build (from Apple Silicon host)

```bash
rustup target add x86_64-apple-darwin
pnpm tauri:build:mac:intel
# Artifact: src-tauri/target/x86_64-apple-darwin/release/bundle/dmg/As It Sounds_0.1.0_x64.dmg
```

#### E. macOS Universal (ARM + Intel)

```bash
rustup target add aarch64-apple-darwin x86_64-apple-darwin
pnpm tauri:build:mac
# Artifact: src-tauri/target/universal-apple-darwin/release/bundle/dmg/As It Sounds_0.1.0_universal.dmg
```

#### F. Move artifacts from VM/container to host

| Method | Command |
| :-- | :-- |
| **Parallels shared folder** | macOS Documents auto-mounted — `\\Mac\Home\Documents` (Win), `/media/psf/Home` (Linux) |
| **UTM SPICE share** | VM settings → Sharing → Directory Share |
| **scp** | `scp user@vm-ip:/path/to/installer.msi ~/Downloads/` |
| **Docker volume mount** | `-v "$(pwd)":/work` (already wired up in examples above) |

---

Artifacts are written to `src-tauri/target/release/bundle/`:
- macOS: `.app`, `.dmg`
- Windows: `.msi`, `.exe` (NSIS)
- Linux: `.deb`, `.rpm`, `.AppImage`

### Code signing

**macOS (Apple Developer ID)**
```bash
export APPLE_SIGNING_IDENTITY="Developer ID Application: YOUR NAME (TEAMID)"
export APPLE_ID="you@example.com"
export APPLE_PASSWORD="@keychain:AC_PASSWORD"  # app-specific password
export APPLE_TEAM_ID="TEAMID"
pnpm tauri:build:mac
```
Hardened runtime + entitlements are preconfigured in [src-tauri/entitlements.plist](src-tauri/entitlements.plist). Notarization is performed automatically by the Tauri CLI when `APPLE_*` env vars are set.

**Windows (Authenticode)**
Set `bundle.windows.certificateThumbprint` in `src-tauri/tauri.conf.json`, or supply Tauri signing env vars:
```powershell
$env:TAURI_SIGNING_PRIVATE_KEY = "..."
$env:TAURI_SIGNING_PRIVATE_KEY_PASSWORD = "..."
pnpm tauri:build:win
```

**Linux**
Signing is typically optional; AppImage supports GPG signatures if needed.

### GitHub Actions release pipeline

- Pushing a tag matching `v*` triggers [.github/workflows/release.yml](.github/workflows/release.yml), which builds macOS/Windows/Linux in parallel and publishes to a GitHub Release (draft).
- Required secrets for signing: `APPLE_CERTIFICATE`, `APPLE_CERTIFICATE_PASSWORD`, `APPLE_SIGNING_IDENTITY`, `APPLE_ID`, `APPLE_TEAM_ID`, `APPLE_PASSWORD`.

### Tech stack

**Backend (Rust / `src-tauri/`)**
- `tauri` 2.x + `tauri-plugin-dialog` + `tauri-plugin-fs`
- `whisper-rs` — whisper.cpp binding (Apple Silicon Metal acceleration is off by default; enable via feature flag)
- `rusqlite` (bundled) — local database
- `reqwest` (rustls-tls) — model download & translation API
- `tokio` — async runtime

**Frontend (`src/`)**
- React 19 · TypeScript · Vite 7
- React Router · Zustand · Tailwind CSS (dark theme by default)

### Project structure

```
.
├── .doc/                           # Research & PRD
│   ├── PRD.md
│   └── Perplexity-As it sounds 구현을 위한 조사.md
├── .github/workflows/              # CI + release workflows
├── Usage.md                        # Detailed usage guide
├── README-Web.md                   # Web deployment guide
├── docker-compose.yml              # Web: nginx-proxy + acme + backend + frontend
├── docker-compose.local.yml        # Web: local (no HTTPS) deployment
├── Dockerfile.web                  # Web backend (axum) image
├── Dockerfile.web-frontend         # Web frontend (nginx) image
├── Dockerfile.linux-build          # Desktop Linux cross-build image
├── deploy/nginx.app.conf           # Web frontend container nginx config
├── .env.example                    # Web: DOMAIN / LETSENCRYPT_EMAIL template
├── src/                            # React frontend (shared by desktop & web)
│   ├── components/                 # UI components
│   ├── hooks/                      # Hooks (native drop, etc.)
│   ├── ipc/                        # IPC switcher: client.ts / tauriClient.ts / webClient.ts
│   ├── lib/                        # File picker, format utils, env.ts (isTauri detection)
│   ├── pages/                      # Home / project / settings routes
│   ├── store/                      # Zustand global state
│   └── types.ts                    # Frontend types
├── web-server/                     # Web backend axum crate (reuses core via #[path])
│   └── src/
│       ├── main.rs                 # axum router, REST/SSE handlers
│       ├── pipeline.rs             # Broadcast-event-driven pipeline
│       └── core.rs                 # Re-includes src-tauri/src/core/*
└── src-tauri/                      # Rust backend (desktop)
    ├── src/
    │   ├── commands.rs             # Tauri command definitions
    │   ├── lib.rs                  # App entry point, state registration
    │   └── core/
    │       ├── asr.rs              # Whisper wrapper + model download
    │       ├── db.rs               # SQLite (rusqlite)
    │       ├── error.rs            # Shared error types
    │       ├── media.rs            # ffmpeg PCM extraction
    │       ├── models.rs           # Shared data models
    │       ├── pipeline.rs         # Pipeline orchestration
    │       ├── settings.rs         # Settings load/save
    │       ├── translate.rs        # Translation (MyMemory / LibreTranslate)
    │       └── translit.rs         # Multilingual → Korean transliteration (sound-based phonetic rules)
    ├── examples/
    │   └── ais_test.rs             # CLI pipeline test (cargo run --example ais_test)
    ├── capabilities/default.json   # Tauri capability allowlist
    ├── entitlements.plist          # macOS sandbox / notarization
    └── tauri.conf.json
└── test.sh                         # CLI transliteration verification wrapper
```

### Pipeline

```
MediaFile (any language: English, Japanese, French, Spanish, German, Russian…)
  └─▶ ffmpeg  : decode to 16 kHz mono f32 PCM
        └─▶ Whisper ASR (auto language detect) : segments[{start, end, text}]
              ├─▶ translit.rs : script detect → language phonetic rules → Hangul
              └─▶ translate.rs: → Korean (async HTTP)
                    └─▶ SQLite write + `pipeline://event` stream
```

### Troubleshooting

| Symptom | Cause / Fix |
| :-- | :-- |
| `ffmpeg not found: install it or set the path in settings` | Install ffmpeg (`brew install ffmpeg` / `apt install ffmpeg` / `choco install ffmpeg`) or set an absolute path in Settings. |
| `is cmake not installed?` during build | whisper.cpp needs cmake; install it (`brew`/`apt`/`choco install cmake`). |
| Whisper model download fails | Check firewall/proxy, or manually place `ggml-<name>.bin` into the app data `models/` dir from Hugging Face. |
| Empty translations | MyMemory daily limit (1000 words) hit, or network blocked — switch engine in Settings. |
| Video plays without audio | Some codecs aren't supported by the system WebView, but ASR always uses ffmpeg so transcription still works. |
| macOS "app is damaged" | Running an unsigned build — `xattr -cr "/Applications/As It Sounds.app"` or allow it under Privacy & Security. |
| Windows "Unknown publisher" | No Authenticode signature — click *More info → Run anyway* in SmartScreen. |

### Roadmap

- [x] Phase 1 UI shell — sidebar / dropzone / segment view
- [x] Phase 2 Media player — seek / highlight sync
- [x] Phase 3 ASR — whisper-rs + ffmpeg sidecar
- [x] Phase 4 Transliteration — sound-driven multilingual phonetic rules (English, Japanese, French, Spanish, German, Russian), script auto-detect, CLI verifier (`test.sh`)
- [x] Phase 5 Translation — MyMemory / LibreTranslate
- [x] Phase 6 Persistence — SQLite
- [x] Phase 7 Packaging — bundle / entitlements / CI / GitHub Release
- [x] **F1 Real-time progress** — per-stage (decode · asr · translate) status display + per-segment progress bar during translation
- [x] **F2 File retry** — retry button for error/stuck files (`file_reset` → pipeline re-run)
- [x] **F3 Better models** — medium · large-v3-turbo added; English-only vs. multilingual variants labelled (10 options)
- [ ] Phase 8 Editing & export — SRT/VTT export, segment editing, re-run
- [ ] Phase 9 Live recording STT

### License

MIT — see [LICENSE](LICENSE).

Third-party components:
- [whisper.cpp](https://github.com/ggerganov/whisper.cpp) — MIT
- [ffmpeg](https://ffmpeg.org) — LGPL (installed separately)
- Whisper model weights — MIT (Hugging Face `ggerganov/whisper.cpp`)
