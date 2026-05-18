# As It Sounds — 설치 및 사용 가이드 (macOS Apple Silicon)

**대상 빌드:** `As It Sounds_0.1.0_aarch64.dmg`
**플랫폼:** macOS 11+ (Apple Silicon / M 시리즈)

---

## 1. 사전 준비

### 1.1 시스템 요구사항

| 항목 | 최소 | 권장 |
| :-- | :-- | :-- |
| OS | macOS 11 Big Sur 이상 | macOS 14+ |
| CPU | Apple Silicon (M1/M2/M3/M4) | M2 Pro 이상 |
| RAM | 4 GB | 8 GB 이상 |
| 디스크 | 500 MB + Whisper 모델 (75 MB ~ 3 GB) | 5 GB 이상 |
| 네트워크 | 최초 모델 다운로드 + 번역 시에만 필요 | — |

> **참고:** Intel Mac은 이 DMG로 동작하지 않습니다. Intel용은 `As It Sounds_0.1.0_x64.dmg`(별도 빌드)를 사용하세요.

### 1.2 필수 의존성 — `ffmpeg`

음원/영상 디코딩에 `ffmpeg` 가 필요합니다. **앱 실행 전 반드시 설치하세요.**

```bash
# Homebrew 설치 (없으면 https://brew.sh)
brew install ffmpeg
```

설치 확인:
```bash
which ffmpeg
# 출력 예: /opt/homebrew/bin/ffmpeg
```

`ffmpeg` 가 표준 경로(`/opt/homebrew/bin`, `/usr/local/bin`)에 있으면 앱이 자동 탐색합니다. 다른 위치에 설치한 경우 [3.5 설정](#35-설정) 항목의 ffmpeg 경로를 직접 지정하세요.

---

## 2. 설치

### 2.1 DMG 마운트 및 복사

1. `As It Sounds_0.1.0_aarch64.dmg` 더블클릭
2. 마운트된 창에서 **As It Sounds.app** 아이콘을 **Applications** 폴더로 드래그
3. DMG 창 닫기 → Finder 사이드바의 디스크 추출 아이콘 클릭

### 2.2 첫 실행 시 보안 경고 해제

미서명 빌드는 macOS Gatekeeper가 차단합니다. 다음 중 하나 선택:

**방법 A — 권한 허용 (권장)**
1. Applications 폴더에서 `As It Sounds.app` 우클릭 → **열기**
2. 경고창에서 다시 **열기** 클릭
3. 이후부터는 정상 실행

**방법 B — 격리 속성 제거 (터미널)**
```bash
xattr -cr "/Applications/As It Sounds.app"
```
실행 후 일반적인 방법으로 앱 열기 가능.

> **메시지 "손상되었거나 신뢰할 수 없습니다"** 도 위 방법으로 해결됩니다. 코드는 동일하며 단지 Apple Developer ID 서명이 없을 뿐입니다.

---

## 3. 사용

### 3.1 첫 실행 — Whisper 모델 자동 다운로드

처음 변환 시도 시 Whisper `base` **다국어 모델 (≈142 MB)** 을 Hugging Face에서 자동 다운로드합니다. 진행률이 화면 상단에 표시됩니다. 모델은 한 번만 다운로드되며 이후 캐시됩니다.

저장 위치: `~/Library/Application Support/com.asitsounds.app/models/ggml-base.bin`

### 3.2 프로젝트 생성

좌측 사이드바 **"새 프로젝트"** 또는 홈 화면 **"빈 프로젝트로 시작"** 클릭.

### 3.3 파일 추가

두 가지 방법:
- **드래그 앤 드롭** — Finder에서 파일 여러 개를 앱 창으로 드래그
- **파일 추가 버튼** — 우상단 버튼 → 파일 선택 다이얼로그

지원 포맷:
- 오디오: MP3, WAV, M4A, FLAC, OGG
- 비디오: MP4, MOV, MKV, WEBM, AVI

### 3.4 변환 실행

1. 파일 추가 후 상단 우측 **"변환 시작"** 클릭
2. 단계별 진행 상태 표시:
   - `decode` — ffmpeg로 16 kHz mono PCM 추출
   - `asr` — Whisper 추론 (언어 자동 감지)
   - `translate` — 한국어 번역 + 한글 음차 생성

3. 완료 시 세그먼트 단위로 결과 출력:

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
| **ffmpeg 경로** | 비표준 위치에 설치 시 절대경로 입력 | 자동 탐색 |

**모델 선택 가이드:**

| 모델 | 크기 | 속도 (M2 기준) | 정확도 |
| :-- | :-- | :-- | :-- |
| `tiny` | 75 MB | 매우 빠름 | 낮음 |
| `base` | 142 MB | 빠름 | 보통 (기본 권장) |
| `small` | 466 MB | 보통 | 좋음 |
| `medium` | 1.5 GB | 느림 | 매우 좋음 |
| `large-v3` | 2.9 GB | 매우 느림 | 최고 |

영어 전용 콘텐츠라도 다국어 모델 (`.en` 접미사 없는 것)을 권장합니다. 본 앱은 **언어 자동 감지** 후 해당 언어의 발음을 한글로 변환합니다.

### 3.6 미디어 플레이어 연동

변환 완료 후 우측 상단에 플레이어가 표시됩니다.
- 재생 위치에 해당하는 세그먼트가 자동 하이라이트
- 세그먼트 클릭 → 해당 시각으로 즉시 이동 (시크)
- ±15초 점프 버튼, 볼륨/속도 조절 지원

---

## 4. 음차 (한글 발음 변환) 특징

본 앱은 **소리 기반** 음차를 수행합니다. 단어 의미 기반 사전 매칭이 아닌, **실제 발음에 맞는 한글**로 변환:

| 입력 | 출력 | 이유 |
| :-- | :-- | :-- |
| `A.` | 에이. | 단일 대문자 → 알파벳 이름 |
| `B.` | 비. | 〃 |
| `C.` | 씨. | 〃 |
| `corner` | 코너 | `c` 문맥 감지 (c+o = ㅋ) |
| `city` | 시티 | `c` 문맥 감지 (c+i = ㅅ) |
| `we've` | 위브 | 영어 `we'` 축약 처리 |
| `received` | 리시브드 | `re-` 비강세 + `ei` 장모음 + `-ed` 과거시제 |
| `catering` | 케이터잉 | open syllable 장모음 (a+C+V → /eɪ/) |
| `こんにちは` (일본어) | 콩니치하 | 가나 직접 매핑 |
| `bonjour` (프랑스어) | 봉주흐 | 비음 + 묵음 어말자음 |
| `Москва` (러시아어) | 모스크바 | 키릴 음절 조립 |

**지원 언어 (음차 규칙):** 영어 · 일본어 · 프랑스어 · 스페인어 · 독일어 · 러시아어
**그 외 언어:** Whisper STT 결과는 그대로 표시 (원문)

---

## 5. 데이터 저장 위치

```
~/Library/Application Support/com.asitsounds.app/
├── as-it-sounds.sqlite      # 프로젝트·파일·세그먼트·설정 영구 저장
└── models/
    └── ggml-base.bin        # 다운로드된 Whisper 모델
```

### 5.1 백업

전체 데이터 백업:
```bash
cp -R ~/Library/Application\ Support/com.asitsounds.app/ ~/Backups/as-it-sounds/
```

### 5.2 완전 초기화 (앱 재설치급)

```bash
rm -rf ~/Library/Application\ Support/com.asitsounds.app/
```

> **주의:** 모든 프로젝트와 변환 결과가 삭제됩니다. SQLite 파일만 지우면 모델은 보존:
> ```bash
> rm ~/Library/Application\ Support/com.asitsounds.app/as-it-sounds.sqlite
> ```

---

## 6. 트러블슈팅

| 증상 | 원인 / 해결 |
| :-- | :-- |
| **"손상되었거나 신뢰할 수 없는 개발자" 경고** | 미서명 빌드 → 2.2의 방법 A 또는 B 적용 |
| **`ffmpeg not found: install it or set the path in settings`** | `brew install ffmpeg` 또는 설정에서 ffmpeg 절대경로 지정 |
| **Whisper 모델 다운로드 실패** | 방화벽/프록시 확인. 수동 다운로드: `https://huggingface.co/ggerganov/whisper.cpp/resolve/main/ggml-base.bin` → `~/Library/Application Support/com.asitsounds.app/models/ggml-base.bin` 으로 저장 |
| **번역 결과가 비어 있음** | MyMemory 일일 한도 (1000 단어) 초과 또는 네트워크 차단 → 설정에서 LibreTranslate 또는 "사용 안 함"으로 변경 |
| **변환이 매우 느림** | 큰 모델(medium/large) 사용 중 → `base` 또는 `small` 로 변경 |
| **영상은 재생되는데 오디오 무음** | 시스템 WebView 코덱 미지원 (재생 한정) → ASR은 ffmpeg가 처리하므로 변환 결과는 정상 |
| **앱이 즉시 종료 / 실행 안 됨** | 콘솔 로그 확인: `Console.app` → "As It Sounds" 검색. ffmpeg 미설치가 가장 흔한 원인 |
| **세그먼트 클릭 시 시크 안 됨** | 비디오 코덱 미지원 → 오디오 파일로 분리 후 재변환 권장 |
| **모델이 자꾸 다시 다운로드** | 데이터 디렉터리 쓰기 권한 문제 → `ls -la ~/Library/Application\ Support/com.asitsounds.app/models/` 확인 |

### 6.1 로그 확인

```bash
# 실시간 로그 스트림 (앱 실행 중)
log stream --predicate 'subsystem == "com.asitsounds.app"' --info

# 또는 Console.app 사용
open /System/Applications/Utilities/Console.app
```

---

## 7. 단축키

| 기능 | 단축키 |
| :-- | :-- |
| 재생/일시정지 | `Space` (HTML 플레이어 포커스 시) |
| 개발자 도구 | `⌥ ⌘ I` |

> 신규 프로젝트/파일 추가/변환 시작 등 앱 단축키는 v0.1.0 에 없음. 버튼 클릭 사용.

---

## 8. 자주 묻는 질문

**Q. 인터넷이 없어도 동작하나요?**
A. ASR/음차는 완전 오프라인입니다. 단 **최초 모델 다운로드** 와 **한국어 번역** 만 네트워크 필요. 번역을 "사용 안 함"으로 설정하면 변환은 100% 오프라인.

**Q. 모델 파일을 미리 받아 둘 수 있나요?**
A. 가능. Hugging Face에서 직접 받아 `~/Library/Application Support/com.asitsounds.app/models/` 에 `ggml-<모델명>.bin` 형식으로 저장:
```bash
mkdir -p ~/Library/Application\ Support/com.asitsounds.app/models/
curl -L -o ~/Library/Application\ Support/com.asitsounds.app/models/ggml-base.bin \
  https://huggingface.co/ggerganov/whisper.cpp/resolve/main/ggml-base.bin
```

**Q. 영어 외 다른 언어도 한글로 변환되나요?**
A. 됩니다. Whisper가 언어를 자동 감지하고, 음차 모듈이 스크립트 (라틴/가나/키릴/한글 등) 를 감지해 해당 언어의 발음 규칙으로 한글화합니다.

**Q. 결과를 SRT/VTT로 내보낼 수 있나요?**
A. v0.1.0 에서는 미지원. **Phase 8 로드맵** 에 포함되어 있습니다.

**Q. 실시간 마이크 입력 STT는 되나요?**
A. v0.1.0 에서는 미지원. **Phase 9 로드맵** 에 포함되어 있습니다.

**Q. 변환 결과를 검증해 보고 싶습니다.**
A. 개발자 모드에서 CLI 검증 도구 제공:
```bash
./test.sh .samples/01_19_04.mp3
```
GitHub 저장소 클론 후 `pnpm install` → `./test.sh <오디오 파일>`.

---

## 9. 라이선스 및 크레딧

- **As It Sounds** — MIT
- **whisper.cpp** — MIT
- **ffmpeg** — LGPL (별도 설치)
- **Whisper 모델 가중치** — MIT (Hugging Face `ggerganov/whisper.cpp`)

문의/이슈: GitHub Issues 또는 프로젝트 README 참조.
