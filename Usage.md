# As It Sounds — 설치 및 사용 가이드 (멀티플랫폼)

**플랫폼:** macOS 11+ (ARM / Intel) · Windows 10/11 · Linux (Ubuntu 20.04+, Fedora, Arch)

| 플랫폼 | 빌드 파일 (v0.1.0) |
| :-- | :-- |
| macOS Apple Silicon | `As It Sounds_0.1.0_aarch64.dmg` |
| macOS Intel | `As It Sounds_0.1.0_x64.dmg` |
| Windows (x64) | `As It Sounds_0.1.0_x64_en-US.msi` (또는 `.exe` NSIS 설치 파일) |
| Linux (Debian/Ubuntu) | `as-it-sounds_0.1.0_amd64.deb` / `as-it-sounds_0.1.0_arm64.deb` |
| Linux (Fedora/RHEL) | `as-it-sounds-0.1.0-1.x86_64.rpm` / `as-it-sounds-0.1.0-1.aarch64.rpm` |
| Linux (배포판 무관) | `as-it-sounds_0.1.0_amd64.AppImage` |

---

## 1. 사전 준비

### 1.1 시스템 요구사항

| 항목 | 최소 | 권장 |
| :-- | :-- | :-- |
| OS | macOS 11 / Windows 10 / Ubuntu 20.04 | 최신 |
| CPU | x86_64 또는 arm64 | Apple Silicon · AVX2 지원 Intel · ARM64 |
| RAM | 4 GB | 8 GB 이상 |
| 디스크 | 500 MB + Whisper 모델 (75 MB ~ 3 GB) | 5 GB 이상 |
| 네트워크 | 최초 모델 다운로드 + 번역 시에만 필요 | — |

### 1.2 필수 의존성 — `ffmpeg`

오디오/영상 디코딩에 `ffmpeg` 필요. **앱 실행 전 반드시 설치하세요.**

**macOS** (Homebrew)
```bash
brew install ffmpeg
```

**Windows** (winget 또는 Chocolatey)
```powershell
winget install Gyan.FFmpeg
# 또는
choco install ffmpeg
```
설치 후 새 PowerShell/터미널 열기 (PATH 갱신).

**Linux** (Ubuntu/Debian)
```bash
sudo apt update && sudo apt install -y ffmpeg
```

**Linux** (Fedora/RHEL)
```bash
sudo dnf install -y ffmpeg
```

**Linux** (Arch)
```bash
sudo pacman -S ffmpeg
```

설치 확인:
```bash
ffmpeg -version
```

> 표준 경로(`/opt/homebrew/bin`, `/usr/local/bin`, `/usr/bin`, `C:\ffmpeg\bin` 등)에 있으면 앱이 자동 탐색. 비표준 위치 시 [3.5 설정](#35-설정) → ffmpeg 경로 직접 지정.

---

## 2. 설치

### 2.1 macOS (.dmg)

**Apple Silicon (M1/M2/M3/M4):** `As It Sounds_0.1.0_aarch64.dmg`
**Intel:** `As It Sounds_0.1.0_x64.dmg`

1. DMG 더블클릭 → 마운트
2. **As It Sounds.app** 아이콘을 **Applications** 폴더로 드래그
3. DMG 창 닫기 → 디스크 추출

**첫 실행 — Gatekeeper 우회 (미서명 빌드):**

방법 A — 권한 허용 (권장)
1. Applications 폴더에서 `As It Sounds.app` **우클릭** → **열기**
2. 경고창 다시 **열기** 클릭

방법 B — 격리 속성 제거 (터미널)
```bash
xattr -cr "/Applications/As It Sounds.app"
```

> **"손상되었거나 신뢰할 수 없는 개발자"** 메시지 동일 방법으로 해결. 코드는 동일, 단지 Apple Developer ID 서명 부재.

### 2.2 Windows (.msi / .exe)

**MSI 설치 (권장):**
1. `As It Sounds_0.1.0_x64_en-US.msi` 더블클릭
2. **"Windows에서 PC를 보호했습니다"** SmartScreen 경고 → **추가 정보** → **실행** 클릭
3. 설치 마법사 진행 (기본 경로: `C:\Program Files\As It Sounds\`)
4. 시작 메뉴 → "As It Sounds" 실행

**NSIS 설치 (.exe):**
- `.exe` 파일 더블클릭 → 설치 → 시작 메뉴 등록 (동일)

**SmartScreen 경고 무시:** Authenticode 서명 부재 시 발생. **추가 정보 > 실행** 클릭.

### 2.3 Linux

**Debian / Ubuntu (.deb)**
```bash
sudo dpkg -i as-it-sounds_0.1.0_amd64.deb
# 의존성 누락 시:
sudo apt --fix-broken install
```
시작 메뉴 또는 터미널:
```bash
as-it-sounds
```

**Fedora / RHEL (.rpm)**
```bash
sudo rpm -i as-it-sounds-0.1.0-1.x86_64.rpm
# 또는
sudo dnf install ./as-it-sounds-0.1.0-1.x86_64.rpm
```

**AppImage (배포판 무관)**
```bash
chmod +x As\ It\ Sounds_0.1.0_amd64.AppImage
./As\ It\ Sounds_0.1.0_amd64.AppImage
```
> **FUSE 필요:** Ubuntu 22.04+ 는 `sudo apt install libfuse2`.

**런타임 의존성 (Linux):**
- `libwebkit2gtk-4.1-0` (또는 `4.0-37`) — WebView
- `libappindicator3-1` — 시스템 트레이
- `librsvg2-2` — SVG 렌더링
- `ffmpeg` — 오디오 디코딩

Ubuntu 누락 시 한 번에:
```bash
sudo apt install -y libwebkit2gtk-4.1-0 libappindicator3-1 librsvg2-2 ffmpeg
```

---

## 3. 사용

### 3.1 첫 실행 — Whisper 모델 자동 다운로드

처음 변환 시도 시 Whisper `base` **다국어 모델 (≈142 MB)** 을 Hugging Face에서 자동 다운로드. 진행률은 화면 상단에 표시. 한 번만 다운로드, 이후 캐시.

저장 위치 (OS별):

| OS | 경로 |
| :-- | :-- |
| macOS | `~/Library/Application Support/com.asitsounds.app/models/ggml-base.bin` |
| Windows | `%APPDATA%\com.asitsounds.app\models\ggml-base.bin` |
| Linux | `~/.local/share/com.asitsounds.app/models/ggml-base.bin` |

### 3.2 프로젝트 생성

좌측 사이드바 **"새 프로젝트"** 또는 홈 화면 **"빈 프로젝트로 시작"** 클릭.

### 3.3 파일 추가

- **드래그 앤 드롭** — 파일 매니저 (Finder/Explorer/Files) 에서 파일 여러 개를 앱 창으로 드래그
- **파일 추가 버튼** — 우상단 버튼 → OS 네이티브 파일 선택 다이얼로그

지원 포맷:
- 오디오: MP3, WAV, M4A, FLAC, OGG
- 비디오: MP4, MOV, MKV, WEBM, AVI

### 3.4 변환 실행

1. 파일 추가 후 상단 우측 **"변환 시작"** 클릭
2. 단계별 진행:
   - `decode` — ffmpeg 로 16 kHz mono PCM 추출
   - `asr` — Whisper 추론 (언어 자동 감지)
   - `translate` — 한국어 번역 + 한글 음차

3. 세그먼트 단위 결과:
```
원문 (STT):    A. I don't think we've received it.
한글 음차:     에이. 아이 돈트 씽크 위브 리시브드 잇.
한국어 번역:   A. 우리는 그것을 받지 못한 것 같습니다.
```

### 3.5 설정 (⚙️ 사이드바 하단)

| 항목 | 설명 | 기본값 |
| :-- | :-- | :-- |
| **Whisper 모델** | tiny (75 MB) ~ large-v3 (2.9 GB). 다국어 권장 (`.en` 접미사 없음) | `base` |
| **번역 엔진** | MyMemory / LibreTranslate / 사용 안 함 | MyMemory |
| **번역 엔드포인트** | 셀프호스트 URL 지정 가능 | MyMemory 기본 |
| **LibreTranslate API Key** | (선택) | 없음 |
| **ffmpeg 경로** | 비표준 위치 설치 시 절대경로 입력 | 자동 탐색 |

**모델 선택 가이드 (M2 기준 속도):**

| 모델 | 크기 | 속도 | 정확도 |
| :-- | :-- | :-- | :-- |
| `tiny` | 75 MB | 매우 빠름 | 낮음 |
| `base` | 142 MB | 빠름 | 보통 (기본) |
| `small` | 466 MB | 보통 | 좋음 |
| `medium` | 1.5 GB | 느림 | 매우 좋음 |
| `large-v3` | 2.9 GB | 매우 느림 | 최고 |

영어 전용 콘텐츠라도 다국어 모델 권장 (앱이 언어 자동 감지 후 발음 변환).

### 3.6 미디어 플레이어 연동

- 재생 위치에 따른 세그먼트 자동 하이라이트
- 세그먼트 클릭 → 해당 시각으로 시크
- ±15초 점프, 볼륨/속도 조절

---

## 4. 음차 (한글 발음 변환) 특징

**소리 기반** 음차 — 단어 의미 사전 매칭 아닌, 실제 발음에 맞는 한글:

| 입력 | 출력 | 이유 |
| :-- | :-- | :-- |
| `A.` / `B.` / `C.` | 에이. / 비. / 씨. | 단일 대문자 → 알파벳 이름 |
| `corner` | 코너 | `c` 문맥 감지 (c+o = ㅋ) |
| `city` | 시티 | `c` 문맥 감지 (c+i = ㅅ) |
| `we've` | 위브 | 영어 `we'` 축약 처리 |
| `received` | 리시브드 | `re-` 비강세 + `ei` 장모음 + `-ed` 과거시제 |
| `catering` | 케이터잉 | open syllable 장모음 |
| `こんにちは` (일본어) | 콩니치하 | 가나 직접 매핑 |
| `bonjour` (프랑스어) | 봉주흐 | 비음 + 묵음 어말자음 |
| `Москва` (러시아어) | 모스크바 | 키릴 음절 조립 |

**지원 언어 (음차 규칙):** 영어 · 일본어 · 프랑스어 · 스페인어 · 독일어 · 러시아어
**기타 언어:** Whisper STT 결과 그대로 표시.

---

## 5. 데이터 저장 위치

| OS | 데이터 디렉터리 |
| :-- | :-- |
| macOS | `~/Library/Application Support/com.asitsounds.app/` |
| Windows | `%APPDATA%\com.asitsounds.app\` (보통 `C:\Users\<user>\AppData\Roaming\com.asitsounds.app\`) |
| Linux | `~/.local/share/com.asitsounds.app/` |

구조 (공통):
```
com.asitsounds.app/
├── as-it-sounds.sqlite      # 프로젝트·파일·세그먼트·설정
└── models/
    └── ggml-base.bin        # 다운로드된 Whisper 모델
```

### 5.1 백업

**macOS / Linux**
```bash
cp -R ~/Library/Application\ Support/com.asitsounds.app/ ~/Backups/as-it-sounds/   # macOS
cp -R ~/.local/share/com.asitsounds.app/ ~/Backups/as-it-sounds/                    # Linux
```

**Windows (PowerShell)**
```powershell
Copy-Item -Recurse "$env:APPDATA\com.asitsounds.app" "$env:USERPROFILE\Backups\as-it-sounds"
```

### 5.2 완전 초기화

**macOS**
```bash
rm -rf ~/Library/Application\ Support/com.asitsounds.app/
```

**Linux**
```bash
rm -rf ~/.local/share/com.asitsounds.app/
```

**Windows (PowerShell)**
```powershell
Remove-Item -Recurse -Force "$env:APPDATA\com.asitsounds.app"
```

> SQLite만 지우고 모델 보존하려면 `as-it-sounds.sqlite` 만 삭제.

---

## 6. 트러블슈팅

### 6.1 공통

| 증상 | 원인 / 해결 |
| :-- | :-- |
| **`ffmpeg not found`** | ffmpeg 미설치 또는 PATH 누락 → 1.2 참조하여 설치, 또는 설정에서 절대경로 지정 |
| **Whisper 모델 다운로드 실패** | 방화벽/프록시 확인. 수동: `https://huggingface.co/ggerganov/whisper.cpp/resolve/main/ggml-base.bin` → 5절 데이터 디렉터리 `models/` 에 `ggml-base.bin` 으로 저장 |
| **번역 결과 비어 있음** | MyMemory 일일 한도(1000 단어) 초과 또는 네트워크 차단 → 설정에서 LibreTranslate 또는 "사용 안 함" |
| **변환이 매우 느림** | 큰 모델(medium/large) → `base`/`small` 로 변경. CPU 코어 부족하면 `tiny` |
| **영상은 재생되는데 오디오 무음** | 시스템 WebView 코덱 미지원. ASR 결과는 정상 |

### 6.2 macOS 전용

| 증상 | 해결 |
| :-- | :-- |
| **"손상되었거나 신뢰할 수 없는 개발자"** | `xattr -cr "/Applications/As It Sounds.app"` |
| **앱이 즉시 종료** | `Console.app` 열고 "As It Sounds" 검색. ffmpeg 미설치 우선 확인 |
| **로그 실시간 확인** | `log stream --predicate 'subsystem == "com.asitsounds.app"' --info` |
| **Apple Silicon 에 Intel DMG 설치** | Rosetta 자동 실행 가능하나 권장 X. ARM DMG 사용 |

### 6.3 Windows 전용

| 증상 | 해결 |
| :-- | :-- |
| **SmartScreen "Windows에서 PC를 보호했습니다"** | **추가 정보 > 실행** |
| **WebView2 누락 오류** | Windows 11 기본 포함. Win 10 은 별도 설치: `winget install Microsoft.EdgeWebView2Runtime` |
| **앱이 흰 화면 / 즉시 닫힘** | WebView2 미설치 또는 손상. 위 명령으로 재설치 |
| **이벤트 뷰어 로그** | `Windows Logs > Application` → 소스 `As It Sounds` 또는 `as-it-sounds` 검색 |
| **ffmpeg PATH 인식 안 됨** | 새 터미널/탐색기 열기. 그래도 안 되면 설정에서 `C:\path\to\ffmpeg.exe` 절대경로 입력 |

### 6.4 Linux 전용

| 증상 | 해결 |
| :-- | :-- |
| **`error while loading shared libraries: libwebkit2gtk-4.1.so.0`** | `sudo apt install libwebkit2gtk-4.1-0` (또는 `-4.0-37` 구버전) |
| **`libappindicator3.so.1: cannot open shared object`** | `sudo apt install libappindicator3-1` |
| **AppImage 실행 시 `dlopen(): libfuse.so.2`** | `sudo apt install libfuse2` |
| **Wayland 환경에서 렌더링 깨짐** | `GDK_BACKEND=x11 as-it-sounds` 로 X11 모드 실행 |
| **시스템 트레이/메뉴 표시 안 됨** | GNOME 에서 `gnome-shell-extension-appindicator` 설치 후 재로그인 |
| **로그 확인** | `journalctl --user -f` (앱 실행 중) 또는 터미널에서 직접 `as-it-sounds` 실행 |

---

## 7. 단축키

| 기능 | 단축키 |
| :-- | :-- |
| 재생/일시정지 | `Space` (HTML 플레이어 포커스 시) |
| 개발자 도구 (macOS) | `⌥ ⌘ I` |
| 개발자 도구 (Win/Linux) | `Ctrl+Shift+I` |

> 신규 프로젝트/파일 추가/변환 시작 등 앱 단축키는 v0.1.0 에 없음. 버튼 클릭 사용.

---

## 8. 자주 묻는 질문

**Q. 인터넷 없이도 동작?**
A. ASR/음차는 완전 오프라인. 최초 모델 다운로드와 한국어 번역만 네트워크 필요. 번역 "사용 안 함" 설정 시 100% 오프라인.

**Q. 모델 미리 다운로드?**

macOS / Linux:
```bash
# macOS
MODELS_DIR="$HOME/Library/Application Support/com.asitsounds.app/models"
# Linux
MODELS_DIR="$HOME/.local/share/com.asitsounds.app/models"

mkdir -p "$MODELS_DIR"
curl -L -o "$MODELS_DIR/ggml-base.bin" \
  https://huggingface.co/ggerganov/whisper.cpp/resolve/main/ggml-base.bin
```

Windows (PowerShell):
```powershell
$dir = "$env:APPDATA\com.asitsounds.app\models"
New-Item -ItemType Directory -Force -Path $dir | Out-Null
Invoke-WebRequest -OutFile "$dir\ggml-base.bin" `
  https://huggingface.co/ggerganov/whisper.cpp/resolve/main/ggml-base.bin
```

**Q. 다른 언어도 한글 변환?**
A. 됩니다. Whisper 언어 자동 감지 → 음차 모듈이 스크립트 (라틴/가나/키릴/한글 등) 감지 → 해당 언어 발음 규칙으로 한글화.

**Q. SRT/VTT 내보내기?**
A. v0.1.0 미지원. **Phase 8** 로드맵.

**Q. 실시간 마이크 STT?**
A. v0.1.0 미지원. **Phase 9** 로드맵.

**Q. 변환 결과 검증 (CLI)?**
A. 개발자 모드 CLI 도구:
```bash
./test.sh .samples/01_19_04.mp3      # macOS / Linux
.\test.sh .samples\01_19_04.mp3       # Windows (Git Bash 또는 WSL)
```
GitHub 저장소 클론 → `pnpm install` → `./test.sh <오디오>`.

**Q. 모든 플랫폼 설치 파일을 한 번에 빌드?**
A. **각 플랫폼은 네이티브 호스트에서만 안정 빌드 가능.** 본 프로젝트는 C/C++ 네이티브 의존성 (whisper.cpp, ring, rusqlite) 때문에 크로스컴파일 사실상 불가:

| 호스트 | 빌드 가능 |
| :-- | :-- |
| macOS | macOS ARM/Intel/Universal |
| Windows | Windows x64 |
| Linux | Linux x64/arm64 |

`pnpm tauri:build:all` 은 모든 target 시도하나, 호스트 OS와 다른 target 은 실패. **GitHub Actions CI 권장** — 태그 푸시(`git tag v0.1.0 && git push --tags`) 시 macOS/Win/Linux 네이티브 러너에서 동시 빌드 후 자동 릴리스.

대안: Windows VM (Parallels/UTM/VMware) 또는 Linux VM 에서 직접 빌드.

---

## 9. 라이선스 및 크레딧

- **As It Sounds** — MIT
- **whisper.cpp** — MIT
- **ffmpeg** — LGPL (별도 설치)
- **Whisper 모델 가중치** — MIT (Hugging Face `ggerganov/whisper.cpp`)

문의/이슈: GitHub Issues 또는 프로젝트 README 참조.
