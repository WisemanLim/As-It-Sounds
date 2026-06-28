<div align="center">

# As It Sounds

> 🌐 **한국어** · [English](README.en.md)

**어떤 언어의 음성/영상이든 실제 발음을 한글로 옮기고, 한국어 번역을 함께 보여주는 오프라인 우선 데스크탑 앱**

Tauri 2 · Rust · React 19 · TypeScript · Whisper.cpp

</div>

---

### 소개

**As It Sounds** 는 **어떤 언어**의 음성·영상 파일이든 로컬에서 **Whisper** 로 인식하고, 그 결과를 **실제 발음 그대로 한글 표기(음차)** 와 **한국어 번역** 으로 나란히 보여주는 데스크탑 앱입니다. 언어를 별도로 지정할 필요 없이 자동으로 감지합니다. 파일을 외부로 전송하지 않고 오프라인에서 동작하는 것이 기본이며, 번역만 선택적으로 외부 API(MyMemory, LibreTranslate)를 사용합니다.

> 예: 영어 `A. I don't think we've received it.` → **에이. 아이 돈트 씽크 위브 리시브드 잇.**
> 예: 영어 `C. Right around the corner.` → **씨. 라이트 어라운드 더 코너.**
> 예: 일본어 `こんにちは` → **콩니치하**
> 예: 프랑스어 `bonjour` → **봉주흐**
> 예: 러시아어 `Москва` → **모스크바**

### 주요 기능

- 🎙️ **완전 오프라인 ASR** — Whisper 다국어 모델(tiny/base/small/medium/large) 로컬 실행, **언어 자동 감지**
- 🔤 **소리 기반 음차 (사전 의존 X)** — 스크립트 자동 감지 + 음운 규칙으로 실제 발음 추출. 단어 의미 사전 매칭이 아닌 문맥 기반 발음 규칙:
  - 단일 대문자 → 알파벳 이름 (A → 에이, B → 비)
  - `c` 문맥 감지 (corner=코너, city=시티)
  - 영어 `re-` 비강세, `-ed` 과거시제, `ei` 장모음, `we'` 축약, open syllable 장모음
  - 일본어 가나 직접 매핑, 프랑스어 비음/묵음, 스페인어 jota, 독일어 움라우트, 키릴문자 음절 조립
- 🌐 **한국어 번역** — MyMemory(무료, 키 불필요) 기본 / LibreTranslate 셀프호스트 지원
- 🎧 **음성·영상 모두 지원** — MP3, WAV, M4A, FLAC, OGG, MP4, MOV, MKV, WEBM, AVI
- 🖱️ **다중 파일 드래그앤드롭** — OS 레벨 네이티브 드롭
- ▶️ **미디어 플레이어 연동** — 재생 위치에 따른 세그먼트 실시간 하이라이트, 세그먼트 클릭 시 해당 구간으로 시크
- 💾 **로컬 SQLite 저장** — 프로젝트·파일·세그먼트·설정 영구 저장
- 🖥️ **멀티플랫폼** — macOS (Universal), Windows, Linux
- 🌍 **웹 배포 지원** — 동일 Rust 코어를 axum HTTP/SSE 서버로 재사용, Docker 한 줄로 셀프호스트 → [README-Web.md](README-Web.md)

### 시스템 요구사항

| 항목 | 최소 | 권장 |
| :-- | :-- | :-- |
| OS | macOS 11+, Windows 10+, Ubuntu 20.04+ | 최신 |
| RAM | 4 GB | 8 GB |
| 디스크 | 500 MB + Whisper 모델(75 MB~3 GB) | 5 GB |
| CPU | x86_64 / arm64 | Apple Silicon 또는 AVX2 |
| 네트워크 | 모델 최초 다운로드 및 번역 시에만 필요 | — |

### CLI 테스트 (음차 정확도 검증)

샘플 음원을 빠르게 검증하는 CLI 파이프라인:

```bash
./test.sh .samples/01_19_04.mp3              # base 모델 (기본)
./test.sh .samples/01_19_04.mp3 small        # small 모델 지정
```

출력 예시:
```
시작          종료          원문 (STT)                                  한글 음차
─────────────────────────────────────────────────────────────────────────────
00:00.000   00:04.540    Number 4. I sent the catering order to ...   넘버 4. 아이 센트 더 케이터잉 오더 ...
00:04.540   00:07.700    A. I don't think we've received it.          에이. 아이 돈트 씽크 위브 리시브드 잇.
00:07.700   00:10.440    B. They're out of order.                     비. 데어 아웃 오브 오더.
00:10.440   00:13.140    C. Right around the corner.                  씨. 라이트 어라운드 더 코너.
```

내부적으로 `cargo run --example ais_test`(release) 호출 → ffmpeg PCM 추출 → Whisper 추론 → translit 음차.

### 전체 테스트 스위트 (`test/`)

코어·웹·데스크탑 전반을 시나리오 기반으로 검증합니다. [`test/SCENARIOS.md`](test/SCENARIOS.md) 및 최신 결과 [`test/results/RESULTS.md`](test/results/RESULTS.md) 참조.

| ID | 범위 | 방법 | 상태 |
| :-- | :-- | :-- | :-- |
| D1.1 | Rust 코어 유닛 (translit / asr / media) | `cargo test --manifest-path src-tauri/Cargo.toml --release` | ✅ 27/27 |
| D1.2 | CLI 파이프라인 (STT → 한글) | `./test.sh .samples/01_19_04.mp3 base` | ✅ pass |
| D2.1–2.3 | 웹 빌드 + 서빙 | `pnpm typecheck` · `pnpm build` · `pnpm preview` | ✅ pass |
| D2.4 | 웹 풀스택 (브라우저) | `as-it-sounds-web` (axum `:8080`) + `VITE_AIS_API_BASE` preview, 헤드리스 렌더 | ✅ pass |
| D3.1 | 데스크탑 번들 | `pnpm tauri:build:mac` (universal) → `.app` + `.dmg` | ✅ pass |
| D3.2 | 데스크탑 실행 | 빌드된 `.app` 실행, 크래시 없음 | ✅ pass |

> 웹 모드는 axum `/api` 백엔드 필요 (운영 환경에서는 nginx-proxy 뒤에서 제공). 백엔드 없이 `vite preview`만 실행하면 의도적으로 "초기화에 실패했습니다"가 표시됩니다.

### 빠른 시작 (개발자)

#### 1. 사전 설치

**macOS**
```bash
# Node / pnpm / cmake / ffmpeg
brew install node pnpm cmake ffmpeg

# Rust — rustup 권장 (크로스 컴파일 필수). Homebrew rust 는 단일 target만 지원.
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
source "$HOME/.cargo/env"
```

> **이미 `brew install rust` 한 경우** — 크로스 컴파일 (Intel / Windows / Linux) 을 하려면 rustup 으로 교체:
> ```bash
> brew uninstall rust
> curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
> source "$HOME/.cargo/env"
> ```

**크로스 컴파일 target 추가 (rustup 필수)**
```bash
# macOS Universal (ARM + Intel)
rustup target add aarch64-apple-darwin x86_64-apple-darwin

# Windows (MSVC) — Windows 호스트 또는 cargo-xwin 필요
rustup target add x86_64-pc-windows-msvc

# Linux — Linux 호스트 권장 (webkit2gtk 링크 때문)
rustup target add x86_64-unknown-linux-gnu aarch64-unknown-linux-gnu
```

**Windows (PowerShell, 관리자 권한)**
```powershell
winget install OpenJS.NodeJS.LTS
winget install pnpm.pnpm
winget install Rustlang.Rustup
winget install Kitware.CMake
winget install Gyan.FFmpeg
# Microsoft C++ Build Tools (Visual Studio 2022)도 설치 필요
winget install Microsoft.VisualStudio.2022.BuildTools --override "--add Microsoft.VisualStudio.Workload.VCTools --includeRecommended"
# WebView2 Runtime (보통 Windows 11은 이미 포함)
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

#### 2. 의존성 설치 및 실행

```bash
pnpm install
pnpm tauri:dev
```

최초 실행 시 Whisper `base` 다국어 모델(≈142 MB) 을 Hugging Face에서 자동 다운로드합니다. 진행률이 상단에 표시됩니다.

### 사용법

1. **새 프로젝트 생성** — 좌측 사이드바 "새 프로젝트" 또는 홈 화면 "빈 프로젝트로 시작"
2. **파일 추가** — 드래그앤드롭 또는 "파일 추가" 버튼으로 음성·영상 파일 선택(다중 가능)
3. **변환 시작** — 상단 우측 "변환 시작" 버튼 클릭
4. **결과 확인** — 각 세그먼트마다 영어 원문 · 한글 발음 · 한국어 번역이 3단으로 표시됨. 세그먼트 클릭 시 해당 시각으로 재생 이동

### 설정 (⚙️ 사이드바 하단)

- **Whisper 모델**: tiny(75 MB) ~ large-v3(2.9 GB) 선택 — 다국어 모델 권장 (`.en` 접미사 없는 것), 크기가 클수록 정확도↑, 속도↓
- **번역 엔진**: MyMemory · LibreTranslate · 사용 안 함
- **번역 엔드포인트**: 셀프호스트 URL 지정 가능
- **LibreTranslate API Key**: (선택)
- **ffmpeg 경로**: 커스텀 경로 지정 (비우면 PATH 자동 탐색)

### 데이터 저장 위치

| OS | 경로 |
| :-- | :-- |
| macOS | `~/Library/Application Support/com.asitsounds.app/` |
| Windows | `%APPDATA%\com.asitsounds.app\` |
| Linux | `~/.local/share/com.asitsounds.app/` |

내부 구조:
- `as-it-sounds.sqlite` — 프로젝트·파일·세그먼트·설정
- `models/ggml-*.bin` — 다운로드된 Whisper 모델

### 배포 빌드

```bash
# 현재 플랫폼용 기본 빌드 (설치 파일 생성)
pnpm tauri:build

# 디버그 번들 (자체 서명, 빠른 확인용)
pnpm tauri:build:debug

# macOS Universal (Apple Silicon + Intel)
pnpm tauri:build:mac
# macOS Apple Silicon 전용
pnpm tauri:build:mac:arm
# macOS Intel 전용
pnpm tauri:build:mac:intel

# Windows
pnpm tauri:build:win

# Linux
pnpm tauri:build:linux
pnpm tauri:build:linux:arm

# 모든 플랫폼 일괄 (macOS Universal + Windows + Linux x64/arm64)
# 주의: 각 target 의 rustup 설치 필요. Win/Linux 는 보통 CI 또는 네이티브 호스트에서만 빌드 성공.
pnpm tauri:build:all
```

> **⚠️ 크로스컴파일 한계 (중요)**
>
> 본 프로젝트는 C/C++ 네이티브 의존성 (`whisper-rs`/whisper.cpp, `ring`, `rusqlite`) 을 사용. 다음은 사실상 불가:
> - macOS → Windows MSVC (`pnpm tauri:build:win`): C++ 헤더 부재로 `ring`/`whisper-rs-sys`/`libsqlite3-sys` 빌드 실패
> - macOS/Windows → Linux: webkit2gtk 링크 실패
>
> **권장 빌드 매트릭스:**
>
> | 호스트 | 빌드 가능 | 권장 명령 |
> | :-- | :-- | :-- |
> | macOS (Apple Silicon) | macOS ARM, macOS Intel, macOS Universal | `tauri:build:mac:arm` / `:intel` / `:mac` |
> | Windows | Windows x64 | `tauri:build:win` |
> | Linux x86_64 | Linux x64 | `tauri:build:linux` |
> | Linux ARM64 | Linux arm64 | `tauri:build:linux:arm` |
>
> **모든 플랫폼 동시 빌드는 [GitHub Actions CI](.github/workflows/release.yml) 사용을 권장.** 태그 푸시(`git tag v0.1.0 && git push --tags`)로 자동 빌드/릴리스.

### CI 없이 멀티플랫폼 빌드 (호스트별 가이드)

각 OS 빌드는 **해당 OS 의 네이티브 환경 (VM/실기/컨테이너)** 에서 수행. 호스트가 macOS 라면 macOS 만 직접 빌드 가능, 나머지는 아래 방식 중 선택.

#### A. Windows x64 빌드

| 방식 | 호스트 | 비고 |
| :-- | :-- | :-- |
| **Parallels Desktop + Windows 11** | macOS (ARM/Intel) | 가장 안정. Windows 11 ARM 또는 x64. 라이선스 필요 |
| **UTM + Windows 11 ARM** | macOS (Apple Silicon) | 무료. ISO 다운로드 [microsoft.com/software-download/windows11arm64](https://www.microsoft.com/software-download/windows11) |
| **VMware Fusion** | macOS | Personal Use 무료 |
| **Boot Camp** | macOS (Intel만) | Apple Silicon 불가 |
| **물리 Windows PC** | 별도 머신 | 가장 빠름 |

**VM 내부 셋업 (Windows 11, PowerShell 관리자):**
```powershell
# 1. 패키지 매니저 (winget 기본 포함, Win 11)
winget install OpenJS.NodeJS.LTS pnpm.pnpm Rustlang.Rustup Kitware.CMake Gyan.FFmpeg Git.Git
# 2. Visual Studio C++ Build Tools (필수, ring/whisper.cpp/rusqlite 빌드용)
winget install Microsoft.VisualStudio.2022.BuildTools --override "--add Microsoft.VisualStudio.Workload.VCTools --includeRecommended"
# 3. WebView2 (Win 11 기본 포함, Win 10 만 필요)
# winget install Microsoft.EdgeWebView2Runtime
# 4. 재부팅 후
git clone https://github.com/WisemanLim/As-It-Sounds.git
cd As-It-Sounds
pnpm install
pnpm tauri:build:win
```
**산출물:** `src-tauri\target\x86_64-pc-windows-msvc\release\bundle\msi\As It Sounds_0.1.0_x64_en-US.msi` 와 `nsis\*.exe`

#### B. Linux x64 빌드

| 방식 | 호스트 | 비고 |
| :-- | :-- | :-- |
| **Docker 컨테이너 (권장)** | 모든 호스트 | 격리, 재현 가능. 호스트에 Docker Desktop 또는 OrbStack 필요 |
| **UTM + Ubuntu 22.04 x64** | macOS | qemu 에뮬레이션 (Apple Silicon 에서 x86_64 게스트는 느림) |
| **Multipass** | 모든 호스트 | 경량 Ubuntu VM. `brew install multipass` |
| **Parallels + Ubuntu** | macOS | 빠름. 라이선스 필요 |
| **물리 Linux PC** | 별도 머신 | 가장 빠름 |

**Docker 빌드 (가장 빠른 경로):**

`Dockerfile.linux-build` (프로젝트 루트):
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

실행:
```bash
docker buildx build --platform linux/amd64 -t ais-linux-build -f Dockerfile.linux-build .
docker run --rm -v "$(pwd)":/work --platform linux/amd64 ais-linux-build
# 산출물: ./src-tauri/target/x86_64-unknown-linux-gnu/release/bundle/{deb,rpm,appimage}/
```

> Apple Silicon 호스트에서 amd64 컨테이너 = qemu 에뮬레이션 → 매우 느림 (10~30분). 가능하면 native Linux 또는 GitHub Codespaces 사용.

#### C. Linux ARM64 빌드 (aarch64)

| 방식 | 호스트 | 비고 |
| :-- | :-- | :-- |
| **Docker buildx (ARM64 컨테이너)** | Apple Silicon | **네이티브 속도** (Mac arm64 = Linux arm64 호환). 가장 빠름 |
| **UTM + Ubuntu 22.04 arm64** | macOS Apple Silicon | KVM 가속, 거의 네이티브 속도 |
| **Asahi Linux 듀얼부팅** | Apple Silicon | 최고 성능. 설치 부담 |
| **Raspberry Pi 4/5 (8GB)** | 별도 기기 | 실기. 컴파일 시간 길음 (whisper.cpp 빌드 무거움) |

**Docker 빌드 (Apple Silicon → Linux ARM64, 네이티브 속도):**
```bash
docker buildx build --platform linux/arm64 -t ais-linux-arm-build -f Dockerfile.linux-build .
docker run --rm -v "$(pwd)":/work --platform linux/arm64 ais-linux-arm-build \
  bash -c "pnpm install && pnpm tauri:build:linux:arm"
# 산출물: ./src-tauri/target/aarch64-unknown-linux-gnu/release/bundle/{deb,rpm,appimage}/
```

#### D. macOS Intel 빌드 (Apple Silicon 호스트)

```bash
rustup target add x86_64-apple-darwin
pnpm tauri:build:mac:intel
# 산출물: src-tauri/target/x86_64-apple-darwin/release/bundle/dmg/As It Sounds_0.1.0_x64.dmg
```

#### E. macOS Universal (ARM + Intel)

```bash
rustup target add aarch64-apple-darwin x86_64-apple-darwin
pnpm tauri:build:mac
# 산출물: src-tauri/target/universal-apple-darwin/release/bundle/dmg/As It Sounds_0.1.0_universal.dmg
```

#### F. VM/컨테이너 ↔ 호스트 파일 전송

| 방법 | 명령 |
| :-- | :-- |
| **Parallels 공유 폴더** | macOS Documents 자동 마운트 — `\\Mac\Home\Documents` (Win), `/media/psf/Home` (Linux) |
| **UTM SPICE 공유** | VM 설정 → Sharing → Directory Share |
| **scp** | `scp user@vm-ip:/path/to/installer.msi ~/Downloads/` |
| **Docker 볼륨 마운트** | `-v "$(pwd)":/work` 로 자동 (위 예제 참조) |

---

산출물은 `src-tauri/target/release/bundle/` 아래에 OS별로 생성됩니다:
- macOS: `.app`, `.dmg`
- Windows: `.msi`, `.exe` (NSIS)
- Linux: `.deb`, `.rpm`, `.AppImage`

### 코드 사이닝

**macOS (Apple Developer ID)**
```bash
export APPLE_SIGNING_IDENTITY="Developer ID Application: YOUR NAME (TEAMID)"
export APPLE_ID="you@example.com"
export APPLE_PASSWORD="@keychain:AC_PASSWORD"  # app-specific password
export APPLE_TEAM_ID="TEAMID"
pnpm tauri:build:mac
```
Hardened Runtime 과 Entitlements 는 [src-tauri/entitlements.plist](src-tauri/entitlements.plist) 에 미리 설정되어 있습니다. 공증(notarization)은 Tauri CLI가 자동 수행합니다.

**Windows (Authenticode)**
`src-tauri/tauri.conf.json` 의 `bundle.windows.certificateThumbprint` 에 인증서 지문을 넣거나, 환경변수로 전달:
```powershell
$env:TAURI_SIGNING_PRIVATE_KEY = "..."
$env:TAURI_SIGNING_PRIVATE_KEY_PASSWORD = "..."
pnpm tauri:build:win
```

**Linux**
보통 서명 불필요. AppImage는 GPG 서명만 선택적.

### GitHub Actions 자동 배포

- 태그 푸시(`v*`)로 [.github/workflows/release.yml](.github/workflows/release.yml) 이 트리거되어 macOS/Windows/Linux 3 플랫폼을 동시에 빌드하고 GitHub Release에 업로드합니다.
- 서명 secrets: `APPLE_CERTIFICATE`, `APPLE_CERTIFICATE_PASSWORD`, `APPLE_SIGNING_IDENTITY`, `APPLE_ID`, `APPLE_TEAM_ID`, `APPLE_PASSWORD` 를 리포지터리 Secrets 에 추가하세요.

### 기술 스택

**Backend (Rust / `src-tauri/`)**
- `tauri` 2.x + `tauri-plugin-dialog` + `tauri-plugin-fs`
- `whisper-rs` — whisper.cpp 바인딩 (Apple Silicon Metal 가속은 기본 비활성, 원하면 feature로 켜기)
- `rusqlite` (bundled) — 로컬 DB
- `reqwest` (rustls-tls) — 모델 다운로드, 번역 API
- `tokio` — 비동기 런타임

**Frontend (`src/`)**
- React 19 · TypeScript · Vite 7
- React Router · Zustand · Tailwind CSS (다크 테마 기본)

### 프로젝트 구조

```
.
├── .doc/                           # 리서치 & PRD
│   ├── PRD.md
│   └── Perplexity-As it sounds 구현을 위한 조사.md
├── .github/workflows/              # CI + release 워크플로우
├── Usage.md                        # 상세 사용 가이드
├── README-Web.md                   # 웹 배포 가이드
├── docker-compose.yml              # 웹: nginx-proxy + acme + backend + frontend
├── docker-compose.local.yml        # 웹: 로컬(HTTPS 없음) 배포
├── Dockerfile.web                  # 웹 백엔드 (axum) 이미지
├── Dockerfile.web-frontend         # 웹 프론트엔드 (nginx) 이미지
├── Dockerfile.linux-build          # 데스크탑 Linux 크로스빌드용
├── deploy/nginx.app.conf           # 웹 프론트 컨테이너 nginx 설정
├── .env.example                    # 웹: DOMAIN / LETSENCRYPT_EMAIL 템플릿
├── src/                            # React 프론트엔드 (데스크탑 + 웹 공용)
│   ├── components/                 # UI 컴포넌트
│   ├── hooks/                      # 네이티브 드롭 등 훅
│   ├── ipc/                        # IPC 스위처: client.ts(분기) / tauriClient.ts / webClient.ts
│   ├── lib/                        # 파일 선택, 포맷 유틸, env.ts(isTauri 감지)
│   ├── pages/                      # 홈/프로젝트/설정 라우트
│   ├── store/                      # Zustand 전역 상태
│   └── types.ts                    # 프론트 타입
├── web-server/                     # 웹 백엔드 axum 크레이트 (코어 #[path] 재사용)
│   └── src/
│       ├── main.rs                 # axum 라우터, REST/SSE 핸들러
│       ├── pipeline.rs             # broadcast 이벤트 기반 파이프라인
│       └── core.rs                 # src-tauri/src/core/* 재포함
└── src-tauri/                      # Rust 백엔드 (데스크탑)
    ├── src/
    │   ├── commands.rs             # Tauri 명령 정의
    │   ├── lib.rs                  # 앱 진입점, 상태 등록
    │   └── core/
    │       ├── asr.rs              # Whisper 래퍼 + 모델 다운로드
    │       ├── db.rs               # SQLite (rusqlite)
    │       ├── error.rs            # 공용 에러
    │       ├── media.rs            # ffmpeg PCM 추출
    │       ├── models.rs           # 공용 모델
    │       ├── pipeline.rs         # 파이프라인 오케스트레이션
    │       ├── settings.rs         # 설정 로드/저장
    │       ├── translate.rs        # 번역(MyMemory / LibreTranslate)
    │       └── translit.rs         # 다국어 → 한글 음차 (소리 기반 음운 규칙)
    ├── examples/
    │   └── ais_test.rs             # CLI 파이프라인 테스트 예제 (cargo run --example ais_test)
    ├── capabilities/default.json   # Tauri capability allowlist
    ├── entitlements.plist          # macOS 샌드박스 / notarization
    └── tauri.conf.json
└── test.sh                         # CLI 음차 검증 래퍼
```

### 파이프라인 흐름

```
MediaFile (영어·일본어·프랑스어·스페인어·독일어·러시아어 등)
  └─▶ ffmpeg  : 16 kHz mono f32 PCM 추출
        └─▶ Whisper ASR (언어 자동 감지) : segments[{start, end, text}]
              ├─▶ translit.rs : 스크립트 감지 → 언어별 발음 규칙 → 한글
              └─▶ translate.rs: → 한국어 (비동기)
                    └─▶ SQLite 저장 + `pipeline://event` 스트리밍
```

### 트러블슈팅

| 증상 | 원인 / 해결 |
| :-- | :-- |
| `ffmpeg not found: install it or set the path in settings` | ffmpeg 미설치 또는 PATH 미포함 → `brew install ffmpeg` 또는 설정에서 절대경로 지정 |
| `failed to execute command: No such file or directory ... is cmake not installed?` | whisper.cpp 빌드용 cmake 누락 → `brew install cmake` / `apt install cmake` / `choco install cmake` |
| Whisper 모델 다운로드 실패 | 방화벽/프록시 설정 확인, 또는 Hugging Face에서 수동 다운로드 후 앱 데이터 디렉터리 `models/` 에 `ggml-<name>.bin` 으로 배치 |
| 번역 결과가 빈 문자열 | MyMemory 일일 한도(1000단어) 초과 또는 네트워크 차단 → 설정에서 엔진 변경 |
| 영상 재생은 되는데 오디오 없음 | 해당 코덱 웹뷰 미지원 → ffmpeg가 전사에는 영향 없음(실제 ASR은 ffmpeg가 수행) |
| macOS "손상된 앱" 경고 | 서명 없이 배포된 경우 → `xattr -cr "/Applications/As It Sounds.app"` 또는 설정 → 개인정보 보호 및 보안에서 허용 |
| Windows "확인되지 않은 게시자" | Authenticode 서명 미적용 → SmartScreen "추가 정보 > 실행" 클릭 |

### 로드맵

- [x] Phase 1 UI 셸 — Sidebar / 드롭존 / 세그먼트 뷰
- [x] Phase 2 미디어 플레이어 — 시크 / 하이라이트 동기화
- [x] Phase 3 ASR 통합 — whisper-rs + ffmpeg 사이드카
- [x] Phase 4 음차 — 소리 기반 다국어 음운 규칙 (영어·일본어·프랑스어·스페인어·독일어·러시아어), 스크립트 자동 감지, CLI 검증 도구(`test.sh`)
- [x] Phase 5 번역 — MyMemory / LibreTranslate
- [x] Phase 6 영구화 — SQLite
- [x] Phase 7 패키징 — 번들 / 엔타이틀먼트 / CI / GitHub Release
- [x] **F1 실시간 진행률** — 단계별(decode·asr·translate) 진행 상태 표시 + translate 세그먼트 단위 퍼센트 프로그레스 바
- [x] **F2 파일 재시도** — 오류/중단 파일 재시도 버튼 (`file_reset` → 파이프라인 재실행)
- [x] **F3 고품질 모델** — medium·large-v3-turbo 추가, 영어 전용/다국어 variants 구분 (10개 옵션)
- [ ] Phase 8 편집·내보내기 — SRT/VTT 내보내기, 세그먼트 수동 수정, 재변환
- [ ] Phase 9 실시간 녹음 STT

### 라이선스

MIT — [LICENSE](LICENSE) 참조.

번들된 외부 구성요소:
- [whisper.cpp](https://github.com/ggerganov/whisper.cpp) — MIT
- [ffmpeg](https://ffmpeg.org) — LGPL (별도 설치)
- Whisper 모델 가중치 — MIT (Hugging Face `ggerganov/whisper.cpp`)
