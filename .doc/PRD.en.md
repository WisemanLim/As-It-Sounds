# As It Sounds — Product Requirements Document (PRD)

> 🌐 [한국어](PRD.md) · **English**

## 1. Overview

**As It Sounds** is a multiplatform desktop application that transcribes English audio/video files into **Hangul phonetic notation (transliteration) exactly as heard**, and provides a **Korean translation** of the same segments side by side.

- **Product name (code)**: `as-it-sounds`
- **Platforms**: macOS, Windows, Linux
- **Tech stack**: Tauri 2 (Rust) + React + TypeScript
- **Runtime model**: Offline-first desktop app. External API integration available as an opt-in.

### 1.1 Core Value

1. English listening/shadowing learners can **visualize native speaker pronunciation** in Hangul.
2. **Translation results** are displayed alongside the transliteration for comprehension.
3. Users can process files **entirely locally without sending data to external services**.

### 1.2 Glossary

- **Project**: A single chat session. A container grouping multiple audio/video files and their conversion results.
- **Transcript**: The original English text output from ASR.
- **Hangul Transliteration**: English pronunciation written in Hangul. e.g. `As it sounds` → `애즈 잇 사운즈`.
- **Translation**: Korean translation of the Transcript.
- **Segment**: A timestamped unit of text (sentence/phrase).

---

## 2. Target Users

- **English learners**: Users who need native pronunciation visualized for pronunciation correction and shadowing practice.
- **Content creators / translators**: Users who need to transcribe interview or lecture audio and produce a Korean translation draft.
- **Privacy-conscious users**: Users who prefer local processing over uploading to commercial APIs.

---

## 3. Technology Stack Rationale

Based on the research document (`.doc/Perplexity-As it sounds 구현을 위한 조사.md`).

| Layer | Choice | Alternatives | Rationale |
| :-- | :-- | :-- | :-- |
| Desktop shell | **Tauri 2** | Electron, Flutter Desktop | Small binary size and memory footprint; runs local model pipelines at high performance via Rust; excellent security model. |
| Frontend | **React + TypeScript + Vite** | Svelte, Vue | Rich ecosystem, abundant chat-style UI libraries, team productivity. |
| State management | **Zustand** | Redux, Jotai | Lightweight, minimal boilerplate. |
| Styling | **Tailwind CSS + shadcn/ui** | MUI | Rapid development of a Gemini-like dark UI. |
| Routing | **React Router** | TanStack Router | Stability. |
| Speech recognition (ASR) | **whisper.cpp** (Rust binding `whisper-rs`) | OpenAI Whisper API, faster-whisper | Fully offline, multilingual support, local GGUF model execution. |
| English→Hangul transliteration | **Hangulize (Rust/Go port) + engtokor-transliterator (optional remote inference)** | konglish(TS) | Rule-based default engine + quality mode toggle. |
| Translation | **LibreTranslate (local Docker) or Argos Translate CLI** (local) / DeepL·Google API (optional) | Direct model serving | Offline by default; high-quality mode selects API. |
| Media playback | **HTML5 `<video>`/`<audio>`** + Tauri asset protocol | Embedded libVLC | Browser engine handles most MP3/WAV/MP4/MOV, minimizing complexity. Falls back to **ffmpeg transcode** for MOV/HEVC codec issues. |
| Audio extraction/decoding | **ffmpeg (sidecar binary)** | symphonia | Wide format support. Video→audio extraction, sample-rate conversion. |
| DB / storage | **SQLite (`tauri-plugin-sql`)** | JSON files | Project/segment indexing and search. |

---

## 4. Functional Requirements

### 4.1 MVP (Phase 1)

| ID | Feature | Description |
| :-- | :-- | :-- |
| F-1 | Project (chat) create / select | Project list in left navigation with a "New Project" button. |
| F-2 | Drag-and-drop upload within a project | Multi-file drop zone in the center area. Supports `.mp3/.wav/.m4a/.mp4/.mov/.mkv`. |
| F-3 | File list management | Display uploaded files as a list inside the project; files can be removed. |
| F-4 | Processing execution | "Start conversion" button → runs ASR → transliteration → translation pipeline. |
| F-5 | Result display | Three-row display per segment: ① original English ② Hangul transliteration ③ Korean translation. |
| F-6 | Media player | File play/pause; highlight the segment corresponding to the current playback position. |
| F-7 | Segment click navigation | Clicking a segment seeks to its timestamp. |
| F-8 | Local persistence | Permanently store projects, segments, and settings in SQLite. |
| F-9 | Settings screen | Whisper model selection (tiny/base/small/medium/large), translation engine selection, API key storage. |

### 4.2 Phase 2 (Extensions)

- Export results (SRT, VTT, TXT, Markdown).
- Segment editing (correct source text → regenerate transliteration and translation).
- Global search (project/segment full-text search).
- Word-level highlight synchronization (karaoke-style).
- Pronunciation difficulty statistics, learning mode (loop playback, shadowing).
- Streaming STT (microphone input).

### 4.3 Non-Functional Requirements

- **Offline-first**: The default path operates without external network access.
- **Privacy**: User files are not sent outside the local directory (except in API mode, which requires explicit opt-in).
- **Performance target**: Processing at real-time factor ≤ 1.0 with Whisper small (Apple Silicon baseline).
- **Accessibility**: Keyboard navigation, default Korean/English UI toggle.
- **Security**: Minimize Tauri allowlist; validate ffmpeg/whisper sidecar paths.

---

## 5. UI/UX Requirements

### 5.1 Overall Layout (Gemini-inspired)

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

- **Dark theme by default**, with a light mode toggle.
- Left **Sidebar (fixed width 260px, collapsible)**: logo, new chat button, project list, settings.
- Right **Main**: shows a start screen when no project is open; shows drop zone, files, and results when a project is active.

### 5.2 Key Screens

1. **Welcome / New Project**: Large centered drop zone reading "Drop your audio/video files here (MP3, WAV, MP4, MOV...)". Clicking opens a file picker dialog.
2. **Project Detail**:
   - Top: file list (thumbnail / filename / duration / status badge: pending/processing/done/error).
   - Currently selected file: media player + three-row segment results.
   - Right side toggle panel: options (model, translation engine, language pair).
3. **Settings**: model download/path, ffmpeg path, translation engine and API keys, data folder.

### 5.3 Segment Rendering

```
[00:03.120 — 00:05.480]
EN:   As it sounds, it works.
HAN:  애즈 잇 사운즈, 잇 웍스.
KO:   들리는 대로, 그것은 동작합니다.
```

- The currently playing segment is highlighted (background).
- Each row has an individual copy button.

---

## 6. System Architecture

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

### 6.1 Processing Pipeline

```
Media File
  └─▶ ffmpeg: 16kHz mono PCM 추출
        └─▶ Whisper ASR: segments[{start,end,text}]
              └─▶ Transliterator: text → 한글 음차
              └─▶ Translator:     text → 한국어 번역
                    └─▶ SQLite 저장 + Renderer 스트리밍 이벤트
```

- **Event streaming** per segment (`segment://{projectId}/{fileId}`); the UI renders progressively.
- Failed segments are kept in a retry queue.

### 6.2 Module Structure (Rust)

- `core::pipeline`: pipeline orchestration.
- `core::asr`: whisper-rs wrapper, model loader.
- `core::translit`: English→Hangul transliteration engine (rule-based default, plugin-style).
- `core::translate`: translation engine abstraction (LibreTranslate HTTP, Argos CLI, external API).
- `core::media`: ffmpeg sidecar control.
- `core::db`: SQLite migrations and repository.

### 6.3 Module Structure (Frontend)

- `src/app`: routing, layout.
- `src/features/projects`: project CRUD.
- `src/features/files`: upload / drop zone.
- `src/features/transcription`: segment view, player integration.
- `src/features/settings`: settings.
- `src/shared/ipc`: Tauri command wrappers, shared types.
- `src/shared/ui`: shadcn/ui-based components.

---

## 7. Data Model (SQLite)

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

## 8. IPC Interface (Tauri Commands)

| Command | Input | Output | Description |
| :-- | :-- | :-- | :-- |
| `project_list` | — | `Project[]` | List all projects. |
| `project_create` | `{ name }` | `Project` | Create a new project. |
| `project_delete` | `{ id }` | `void` | Delete a project. |
| `file_add_many` | `{ projectId, paths[] }` | `MediaFile[]` | Register multiple files. |
| `file_remove` | `{ id }` | `void` | Remove a file. |
| `pipeline_run` | `{ fileIds[], options }` | `void` | Run the pipeline (event streaming). |
| `segments_list` | `{ fileId }` | `Segment[]` | Retrieve stored segments. |
| `settings_get` / `settings_set` | — | `Settings` | Get / save settings. |

**Events**: `pipeline://progress`, `pipeline://segment`, `pipeline://done`, `pipeline://error`.

---

## 9. Development Phases (Roadmap)

| Phase | Goal | Key Deliverables |
| :-- | :-- | :-- |
| **0. Scaffolding** | Project structure based on Tauri + React + Tailwind | Empty app that builds; sidebar and main layout |
| **1. UI Shell** | Project CRUD, drop zone, file list, mock segment render | End-to-end flow verifiable through UI alone |
| **2. Media Playback** | HTML5 player, seek/highlight synchronization | Real file playback |
| **3. ASR Integration** | whisper-rs connection, model downloader, ffmpeg sidecar | Real English transcript generation |
| **4. Transliteration Integration** | Rule-based transliterator connection | "As it sounds" → "애즈 잇 사운즈" |
| **5. Translation Integration** | LibreTranslate (local) first, API fallback | Korean translation displayed |
| **6. Persistence / Search** | SQLite persistence, search | Projects retained after restart |
| **7. Packaging** | macOS/Windows/Linux icons, code signing, updater | Distributable binary |

---

## 10. Risks and Mitigations

| Risk | Impact | Mitigation |
| :-- | :-- | :-- |
| Whisper model size / download time | Poor initial UX | Default to `base` model; allow selective download in settings. |
| MOV/HEVC codec playback failure in WebView | Unplayable files | Transparent ffmpeg transcode fallback. |
| English→Hangul transliteration quality | Reduced learning effectiveness | Rule-based default + optional deep learning model (remote) toggle. |
| Limited Tauri plugin ecosystem | Implementation delays | Implement required parts directly in Rust. |
| Sidecar distribution (ffmpeg license) | Distribution issues | Choose LGPL build; also offer user-installed option. |

---

## 11. Success Metrics

- **Segment accuracy (manual sampling) ≥ 85%** on a 10-minute English video.
- Transliteration readability user satisfaction (beta survey) ≥ 4 / 5.
- Cold start → first segment displayed in ≤ **10 seconds** (Whisper small, M1 baseline).
- Full pipeline operates 100% in offline mode.
