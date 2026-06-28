# As It Sounds — 제품 요구사항 문서 (PRD)

> 🌐 **한국어** · [English](PRD.en.md)

## 1. 개요

**As It Sounds**는 영어 음성/영상 파일을 **들리는 발음 그대로 한글로 표기(음차)** 하고, 동일 구간의 **한국어 번역**을 함께 제공하는 멀티플랫폼 데스크탑 애플리케이션입니다.

- **제품명(코드)**: `as-it-sounds`
- **플랫폼**: macOS, Windows, Linux
- **기술 스택**: Tauri 2 (Rust) + React + TypeScript
- **실행 형태**: 로컬 우선(offline-first) 데스크탑 앱. 선택적으로 외부 API 연동 가능.

### 1.1 핵심 가치

1. 영어 청취/섀도잉 학습자가 **실제 원어민 발음**을 한글로 시각화하여 확인.
2. 의미 파악을 위해 **번역 결과**를 나란히 제공.
3. 사용자의 **파일 데이터를 외부로 전송하지 않고** 로컬에서 처리 가능한 옵션 제공.

### 1.2 용어

- **Project**: 하나의 채팅 세션. 여러 음성/영상 파일과 변환 결과를 묶는 단위.
- **Transcript**: 원문 영어 텍스트(ASR 결과).
- **Hangul Transliteration(음차)**: 영어 발음을 한글로 옮긴 표기. 예) `As it sounds` → `애즈 잇 사운즈`.
- **Translation**: Transcript의 한국어 번역.
- **Segment**: 타임스탬프가 붙은 텍스트 단위(문장/구절).

---

## 2. 타겟 사용자

- **영어 학습자**: 발음 교정/섀도잉을 위해 원어민 발음 시각화가 필요한 사용자.
- **콘텐츠 제작자/번역가**: 인터뷰·강연 음성을 받아 적고 한국어 번역 초안이 필요한 사용자.
- **프라이버시 민감 사용자**: 상용 API에 업로드하지 않고 로컬 처리 선호.

---

## 3. 기술 스택 선정 근거

조사 문서(`.doc/Perplexity-As it sounds 구현을 위한 조사.md`) 기반.

| 계층 | 선택 | 대안 | 선정 근거 |
| :-- | :-- | :-- | :-- |
| 데스크탑 셸 | **Tauri 2** | Electron, Flutter Desktop | 바이너리 크기·메모리 사용량이 작고, Rust로 로컬 모델 파이프라인을 고성능으로 실행. 보안 모델 우수. |
| 프론트엔드 | **React + TypeScript + Vite** | Svelte, Vue | 생태계, 채팅형 UI 라이브러리 풍부, 팀 생산성. |
| 상태관리 | **Zustand** | Redux, Jotai | 경량, 보일러플레이트 최소. |
| 스타일 | **Tailwind CSS + shadcn/ui** | MUI | Gemini 유사 다크 UI를 빠르게 구축. |
| 라우팅 | **React Router** | TanStack Router | 안정성. |
| 음성 인식(ASR) | **whisper.cpp**(Rust 바인딩 `whisper-rs`) | OpenAI Whisper API, faster-whisper | 완전 오프라인, 다국어 지원, GGUF 모델 로컬 구동. |
| 영어→한글 음차 | **Hangulize (Rust/Go 포트) + engtokor-transliterator (선택적 원격 추론)** | konglish(TS) | 규칙 기반 기본 엔진 + 품질 모드 토글. |
| 번역 | **LibreTranslate(로컬 도커) 또는 Argos Translate CLI** (로컬) / DeepL·Google API(선택) | 모델 직접 서빙 | 오프라인 기본, 고품질 모드는 API 선택. |
| 미디어 재생 | **HTML5 `<video>`/`<audio>`** + Tauri asset protocol | libVLC 내장 | 대부분 MP3/WAV/MP4/MOV를 브라우저 엔진이 처리, 복잡성 최소화. MOV/HEVC 등 코덱 이슈 시 **ffmpeg 트랜스코드**로 폴백. |
| 오디오 추출/디코딩 | **ffmpeg (사이드카 바이너리)** | symphonia | 광범위한 포맷 지원. 비디오→오디오 추출, 샘플링 레이트 변환. |
| DB/저장 | **SQLite (`tauri-plugin-sql`)** | JSON 파일 | 프로젝트/세그먼트 인덱싱, 검색. |

---

## 4. 기능 요구사항

### 4.1 MVP (Phase 1)

| ID | 기능 | 설명 |
| :-- | :-- | :-- |
| F-1 | 프로젝트(채팅) 생성/선택 | 좌측 내비게이션에 프로젝트 목록, "새 프로젝트" 버튼. |
| F-2 | 프로젝트 내 드래그앤드롭 업로드 | 중앙 영역에 음성/영상 파일 다중 드롭 존. `.mp3/.wav/.m4a/.mp4/.mov/.mkv` 지원. |
| F-3 | 파일 목록 관리 | 업로드된 파일을 프로젝트 안에 리스트로 표시, 제거 가능. |
| F-4 | 처리 실행 | "변환 시작" 버튼 → ASR → 음차 → 번역 파이프라인 실행. |
| F-5 | 결과 표시 | 세그먼트 단위로 3단 표시: ① 원문 영어 ② 한글 음차 ③ 한국어 번역. |
| F-6 | 미디어 플레이어 | 파일 재생/일시정지, 현재 재생 시각에 해당하는 세그먼트 하이라이트. |
| F-7 | 세그먼트 클릭 탐색 | 세그먼트를 클릭하면 해당 타임스탬프로 시크. |
| F-8 | 로컬 저장 | 프로젝트·세그먼트·설정을 SQLite에 영구 저장. |
| F-9 | 설정 화면 | Whisper 모델 선택(tiny/base/small/medium/large-v3-turbo/large-v3 × 영어전용·다국어), 번역 엔진 선택, API 키 저장. |
| F-22 | 실시간 변환 진행률 | 파이프라인 단계(decode·asr·translate)별 진행 상태 표시. translate 단계는 세그먼트 단위 퍼센트 프로그레스 바 표시. |
| F-23 | 오류/중단 파일 재시도 | 오류(error) 또는 처리 중(processing) 상태 파일에 재시도 버튼 표시. `file_reset` 후 파이프라인 재실행. |

### 4.2 Phase 2 (확장)

- 결과 내보내기 (SRT, VTT, TXT, Markdown).
- 세그먼트 편집(원문 교정 → 음차·번역 재생성).
- 전체 검색(프로젝트·세그먼트 full-text search).
- 단어 단위 하이라이트 동기화(karaoke-style).
- 발음 난이도 통계, 학습 모드(구간 반복, 따라 말하기).
- 스트리밍 STT (녹음 입력).

### 4.3 비기능 요구사항

- **오프라인 우선**: 기본 경로는 외부 네트워크 없이 동작.
- **프라이버시**: 사용자 파일은 로컬 디렉터리 외부로 전송되지 않음(API 모드 제외, 명시적 옵트인).
- **성능 목표**: Whisper small 기준 1배속 이하 처리(RTF ≤ 1.0, Apple Silicon 기준).
- **접근성**: 키보드 내비게이션, 기본 한/영 UI 토글.
- **보안**: Tauri allowlist 최소화, ffmpeg/whisper 사이드카 경로 검증.

---

## 5. UI/UX 요구사항

### 5.1 전체 레이아웃 (Gemini 유사)

```
┌──────────────────────────────────────────────────────────────┐
│ ┌───────────┐ ┌────────────────────────────────────────────┐ │
│ │ Sidebar   │ │  Main                                      │ │
│ │           │ │  ┌────────────────────────────────────────┐│ │
│ │ ▸ 새 채팅 │ │  │ Header (프로젝트 이름, 모델 선택)      ││ │
│ │ ▸ 내 항목 │ │  ├────────────────────────────────────────┤│ │
│ │           │ │  │                                        ││ │
│ │ Projects  │ │  │   Empty: Drag & Drop Zone              ││ │
│ │  • Proj A │ │  │   Active: File list + Player + Result  ││ │
│ │  • Proj B │ │  │                                        ││ │
│ │           │ │  ├────────────────────────────────────────┤│ │
│ │ ⚙ 설정    │ │  │ Action Bar (파일 추가 / 변환 시작)     ││ │
│ └───────────┘ │  └────────────────────────────────────────┘│ │
│                └────────────────────────────────────────────┘ │
└──────────────────────────────────────────────────────────────┘
```

- **다크 테마 기본**, 라이트 토글 제공.
- 좌측 **Sidebar (고정 폭 260px, 접기 가능)**: 로고, 새 채팅 버튼, 프로젝트 리스트, 설정.
- 우측 **Main**: 프로젝트가 없으면 시작 화면, 프로젝트가 있으면 드롭존·파일·결과.

### 5.2 핵심 화면

1. **Welcome / New Project**: 중앙 대형 드롭존 "여기에 음성·영상 파일을 놓으세요 (MP3, WAV, MP4, MOV...)". 클릭 시 파일 선택 다이얼로그.
2. **Project Detail**:
   - 상단: 파일 리스트(썸네일/파일명/길이/상태 배지: pending/processing/done/error).
   - 현재 선택된 파일: 미디어 플레이어 + 3단 세그먼트 결과.
   - 우측 사이드 토글: 옵션(모델, 번역 엔진, 언어쌍).
3. **Settings**: 모델 다운로드/경로, ffmpeg 경로, 번역 엔진·API 키, 데이터 폴더.

### 5.3 세그먼트 렌더링

```
[00:03.120 — 00:05.480]
EN:   As it sounds, it works.
HAN:  애즈 잇 사운즈, 잇 웍스.
KO:   들리는 대로, 그것은 동작합니다.
```

- 현재 재생 중 세그먼트 강조(배경).
- 각 줄은 개별 복사 버튼 제공.

---

## 6. 시스템 아키텍처

```
┌───────────────────────── Tauri App ─────────────────────────┐
│                                                             │
│  [Renderer: React]  ◀───IPC(invoke/event)───▶  [Rust Core]  │
│                                                             │
│   • UI, 상태관리                      • Command: process()  │
│   • 미디어 재생                        • Pipeline Executor  │
│   • 드래그앤드롭                       • File Registry      │
│                                        • SQLite (sqlx)      │
│                                        • whisper-rs          │
│                                        • hangulize-rs        │
│                                        • translate client    │
│                                                             │
│                                     ▼ sidecar / subprocess  │
│                                   ffmpeg, (선택) LibreTranslate│
└─────────────────────────────────────────────────────────────┘
```

### 6.1 처리 파이프라인

```
Media File
  └─▶ ffmpeg: 16kHz mono PCM 추출
        └─▶ Whisper ASR: segments[{start,end,text}]
              └─▶ Transliterator: text → 한글 음차
              └─▶ Translator:     text → 한국어 번역
                    └─▶ SQLite 저장 + Renderer 스트리밍 이벤트
```

- 세그먼트 단위로 **이벤트 스트리밍**(`segment://{projectId}/{fileId}`), UI는 점진적으로 렌더링.
- 실패 세그먼트는 재시도 큐에 보관.

### 6.2 모듈 구성 (Rust)

- `core::pipeline`: 파이프라인 오케스트레이션.
- `core::asr`: whisper-rs 래퍼, 모델 로더.
- `core::translit`: 영어→한글 음차 엔진 (규칙 기반 기본, 플러그인형).
- `core::translate`: 번역 엔진 추상화(LibreTranslate HTTP, Argos CLI, 외부 API).
- `core::media`: ffmpeg 사이드카 제어.
- `core::db`: SQLite 마이그레이션·리포지터리.

### 6.3 모듈 구성 (Frontend)

- `src/app`: 라우팅, 레이아웃.
- `src/features/projects`: 프로젝트 CRUD.
- `src/features/files`: 업로드/드롭존.
- `src/features/transcription`: 세그먼트 뷰, 플레이어 연동.
- `src/features/settings`: 설정.
- `src/shared/ipc`: Tauri command 래퍼, 타입 공유.
- `src/shared/ui`: shadcn/ui 기반 컴포넌트.

---

## 7. 데이터 모델 (SQLite)

```sql
CREATE TABLE projects (
  id TEXT PRIMARY KEY,
  name TEXT NOT NULL,
  created_at INTEGER NOT NULL,
  updated_at INTEGER NOT NULL
);

CREATE TABLE media_files (
  id TEXT PRIMARY KEY,
  project_id TEXT NOT NULL REFERENCES projects(id) ON DELETE CASCADE,
  path TEXT NOT NULL,
  display_name TEXT NOT NULL,
  mime TEXT,
  duration_ms INTEGER,
  status TEXT NOT NULL CHECK (status IN ('pending','processing','done','error')),
  error TEXT,
  created_at INTEGER NOT NULL
);

CREATE TABLE segments (
  id TEXT PRIMARY KEY,
  file_id TEXT NOT NULL REFERENCES media_files(id) ON DELETE CASCADE,
  idx INTEGER NOT NULL,
  start_ms INTEGER NOT NULL,
  end_ms INTEGER NOT NULL,
  text_en TEXT NOT NULL,
  text_hangul TEXT,
  text_ko TEXT
);

CREATE INDEX idx_segments_file ON segments(file_id, idx);
```

---

## 8. IPC 인터페이스 (Tauri Commands)

| Command | 입력 | 출력 | 설명 |
| :-- | :-- | :-- | :-- |
| `project_list` | — | `Project[]` | 프로젝트 목록. |
| `project_create` | `{ name }` | `Project` | 새 프로젝트 생성. |
| `project_delete` | `{ id }` | `void` | 삭제. |
| `file_add_many` | `{ projectId, paths[] }` | `MediaFile[]` | 다중 파일 등록. |
| `file_remove` | `{ id }` | `void` | 파일 제거. |
| `pipeline_run` | `{ fileIds[], options }` | `void` | 파이프라인 실행(이벤트 스트리밍). |
| `segments_list` | `{ fileId }` | `Segment[]` | 저장된 세그먼트 조회. |
| `settings_get` / `settings_set` | — | `Settings` | 설정 조회/저장. |

**이벤트**: `pipeline://progress`, `pipeline://segment`, `pipeline://done`, `pipeline://error`.

---

## 9. 개발 단계 (로드맵)

| Phase | 목표 | 주요 산출물 |
| :-- | :-- | :-- |
| **0. 스캐폴딩** | Tauri + React + Tailwind 기반 프로젝트 구조 | 빌드되는 빈 앱, 사이드바·메인 레이아웃 |
| **1. UI 셸** | 프로젝트 CRUD, 드롭존, 파일 리스트, Mock 세그먼트 렌더 | UI만으로 end-to-end 흐름 확인 |
| **2. 미디어 재생** | HTML5 플레이어, 시크/하이라이트 동기화 | 실제 파일 재생 |
| **3. ASR 통합** | whisper-rs 연결, 모델 다운로더, ffmpeg 사이드카 | 실제 영어 transcript 생성 |
| **4. 음차 통합** | 규칙 기반 transliterator 연결 | "As it sounds" → "애즈 잇 사운즈" |
| **5. 번역 통합** | LibreTranslate(local) 우선, API 폴백 | 한국어 번역 표시 |
| **6. 저장/검색** | SQLite 영구화, 검색 | 재시작 후 프로젝트 유지 |
| **7. 패키징** | macOS/Windows/Linux 아이콘·코드사이닝·업데이터 | 배포 가능한 바이너리 |

---

## 10. 리스크 및 대응

| 리스크 | 영향 | 대응 |
| :-- | :-- | :-- |
| Whisper 모델 크기/다운로드 시간 | 초기 UX 저하 | 기본 `base` 모델, 설정에서 선택 다운로드. |
| MOV/HEVC 코덱 웹뷰 재생 실패 | 재생 불가 | ffmpeg로 투명 트랜스코드 폴백. |
| 영어→한글 음차 품질 | 학습 효과 저하 | 규칙 기반 기본 + 옵션으로 딥러닝 모델(원격) 토글. |
| Tauri 플러그인 생태계 제한 | 구현 지연 | 필요한 부분은 Rust에서 직접 구현. |
| 사이드카 배포(ffmpeg 라이선스) | 배포 이슈 | LGPL 빌드 선택, 사용자 직접 설치 옵션 병행. |

---

## 11. 성공 지표

- 10분짜리 영어 영상에서 **세그먼트 정확도(수동 표본 기준) ≥ 85%**.
- 음차 가독성 사용자 만족도(베타 설문) ≥ 4 / 5.
- 콜드 스타트 → 첫 세그먼트 표시까지 ≤ **10초** (Whisper small, M1 기준).
- 오프라인 모드에서 전체 파이프라인 100% 동작.

---

## 변경 이력

| 차수 | 날짜 | 내용 |
| :-- | :-- | :-- |
| 1차 | 초기 | MVP 기능 정의 (F-1~F-9), 시스템 아키텍처, 데이터 모델 |
| 2차 | 2026-06-28 | F-22 실시간 진행률(progress bar per stage), F-23 오류 파일 재시도, F-9 모델 확장(medium/large-v3-turbo/다국어 variants) |
