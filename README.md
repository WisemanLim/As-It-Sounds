<div align="center">

# As It Sounds

**어떤 언어의 음성/영상이든 실제 발음을 한글로 옮기고, 한국어 번역을 함께 보여주는 오프라인 우선 데스크탑 앱**

**An offline-first desktop app that phonetically transcribes audio/video in any language into Korean Hangul — as it actually sounds — with a Korean translation alongside.**

Tauri 2 · Rust · React 19 · TypeScript · Whisper.cpp

</div>

---

## 🇰🇷 한국어

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

내부적으로 `cargo run --bin ais_test`(release) 호출 → ffmpeg PCM 추출 → Whisper 추론 → translit 음차.

### 빠른 시작 (개발자)

#### 1. 사전 설치

**macOS**
```bash
brew install node pnpm rust cmake ffmpeg
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
```

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
├── src/                            # React 프론트엔드
│   ├── components/                 # UI 컴포넌트
│   ├── hooks/                      # 네이티브 드롭 등 훅
│   ├── ipc/                        # Tauri invoke 래퍼
│   ├── lib/                        # 파일 선택, 포맷 유틸
│   ├── pages/                      # 홈/프로젝트/설정 라우트
│   ├── store/                      # Zustand 전역 상태
│   └── types.ts                    # 프론트 타입
└── src-tauri/                      # Rust 백엔드
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
    ├── src/bin/
    │   └── ais_test.rs             # CLI 파이프라인 테스트 바이너리
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
- [ ] Phase 8 편집·내보내기 — SRT/VTT 내보내기, 세그먼트 수동 수정, 재변환
- [ ] Phase 9 실시간 녹음 STT

### 라이선스

MIT — [LICENSE](LICENSE) 참조.

번들된 외부 구성요소:
- [whisper.cpp](https://github.com/ggerganov/whisper.cpp) — MIT
- [ffmpeg](https://ffmpeg.org) — LGPL (별도 설치)
- Whisper 모델 가중치 — MIT (Hugging Face `ggerganov/whisper.cpp`)

---

## 🇬🇧 English

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

Internally invokes `cargo run --bin ais_test` (release) → ffmpeg PCM extract → Whisper inference → translit.

### Quick start (developers)

#### 1. Prerequisites

**macOS**
```bash
brew install node pnpm rust cmake ffmpeg
```

**Windows (PowerShell as Administrator)**
```powershell
winget install OpenJS.NodeJS.LTS
winget install pnpm.pnpm
winget install Rustlang.Rustup
winget install Kitware.CMake
winget install Gyan.FFmpeg
winget install Microsoft.VisualStudio.2022.BuildTools --override "--add Microsoft.VisualStudio.Workload.VCTools --includeRecommended"
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
4. **Review** — each segment shows the English original, the Korean phonetic transliteration, and the Korean translation. Click a segment to seek the player.

### Settings

- **Whisper model**: tiny (75 MB) … large-v3 (2.9 GB) — prefer multilingual models (no `.en` suffix); bigger = more accurate, slower.
- **Translator**: MyMemory · LibreTranslate · *Disabled*.
- **Endpoint** and **API key** (LibreTranslate): configurable.
- **ffmpeg path**: custom path if not in `PATH`.

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
```

Artifacts are written to `src-tauri/target/release/bundle/`:
- macOS: `.app`, `.dmg`
- Windows: `.msi`, `.exe` (NSIS)
- Linux: `.deb`, `.rpm`, `.AppImage`

### Code signing

**macOS (Apple Developer ID)**
```bash
export APPLE_SIGNING_IDENTITY="Developer ID Application: YOUR NAME (TEAMID)"
export APPLE_ID="you@example.com"
export APPLE_PASSWORD="@keychain:AC_PASSWORD"
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
- `whisper-rs` — whisper.cpp binding
- `rusqlite` (bundled) — local database
- `reqwest` (rustls-tls) — model download & translation
- `tokio` — async runtime

**Frontend (`src/`)**
- React 19 · TypeScript · Vite 7
- React Router · Zustand · Tailwind CSS (dark theme)

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
- [ ] Phase 8 Editing & export — SRT/VTT export, segment editing, re-run
- [ ] Phase 9 Live recording STT

### License

MIT — see [LICENSE](LICENSE).

Third-party components:
- [whisper.cpp](https://github.com/ggerganov/whisper.cpp) — MIT
- [ffmpeg](https://ffmpeg.org) — LGPL (installed separately)
- Whisper model weights — MIT (Hugging Face `ggerganov/whisper.cpp`)
