# src-tauri/src/core/models.rs — 코드 리뷰 / Code Review

> 리뷰 레벨 / Review Level: 2
> [← INDEX.md](../../../INDEX.md)

**파일 요약 / File Summary**
- 역할 / Role: 도메인 데이터 모델 정의 (Project, MediaFile, Segment, Settings, PipelineEvent…). serde 직렬화/역직렬화.
- 진입점에서의 거리 / Distance from entrypoint: 2
- 라인 수 / LOC: 154
- 주요 export / Exports: `Project`, `MediaKind`, `FileStatus`, `MediaFile`, `Segment`, `Settings`, `NewFileInput`, `PipelineEvent`
- 의존 모듈 / Depends on (internal): 없음
- 사용된 외부 패키지 / External packages used: `serde`
- 부수효과 / Side effects: 없음
- 동시성 모델 / Concurrency model: N/A (모두 `Clone`).
- 테스트 커버리지 흔적 / Test coverage signal: 없음.

```rust path=src-tauri/src/core/models.rs lines=1-10
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Project {
    pub id: String,
    pub name: String,
    pub created_at: i64,
    pub updated_at: i64,
}
```
**분석 / Analysis (Level 2)**
- `#[serde(rename_all = "camelCase")]` → JSON 에서 `createdAt` 으로 직렬화. TS 프론트와 호환. / camelCase wire format.
- `i64` 는 ms epoch 가정. UTC. / Epoch ms.

```rust path=src-tauri/src/core/models.rs lines=12-41
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum MediaKind {
    Audio,
    Video,
}

impl MediaKind {
    pub fn from_name(name: &str) -> Self {
        let lower = name.to_lowercase();
        let video_exts = [".mp4", ".mov", ".mkv", ".webm", ".avi", ".m4v"];
        if video_exts.iter().any(|e| lower.ends_with(e)) {
            MediaKind::Video
        } else {
            MediaKind::Audio
        }
    }
    pub fn as_str(&self) -> &'static str {
        match self {
            MediaKind::Audio => "audio",
            MediaKind::Video => "video",
        }
    }
    pub fn parse(s: &str) -> Self {
        match s {
            "video" => MediaKind::Video,
            _ => MediaKind::Audio,
        }
    }
}
```
**분석 / Analysis (Level 2)**
- `from_name` 은 확장자 휴리스틱. mp3/wav 등 명시 오디오 확장자가 아닌 알 수 없는 파일도 Audio 로 간주. / Unknown extensions default to Audio.
- `parse` 는 lowercase 만 처리. DB 의 텍스트가 항상 lowercase 라는 invariant 가짐.

**짚어볼 점 / Watch-outs**
- `parse` 의 fallback 이 Audio. 잘못된 enum value 가 DB 에 있어도 Audio 로 silently. 디버깅 어려울 수 있음.

```rust path=src-tauri/src/core/models.rs lines=43-69
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum FileStatus {
    Pending,
    Processing,
    Done,
    Error,
}

impl FileStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            FileStatus::Pending => "pending",
            FileStatus::Processing => "processing",
            FileStatus::Done => "done",
            FileStatus::Error => "error",
        }
    }
    pub fn parse(s: &str) -> Self {
        match s {
            "processing" => FileStatus::Processing,
            "done" => FileStatus::Done,
            "error" => FileStatus::Error,
            _ => FileStatus::Pending,
        }
    }
}
```
**분석 / Analysis (Level 2)**
- 상태 머신 4개 값. `parse` 의 default 가 Pending → 의미상 “초기 상태” 라 합리적.

```rust path=src-tauri/src/core/models.rs lines=71-97
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct MediaFile {
    pub id: String,
    pub project_id: String,
    pub name: String,
    pub path: String,
    pub kind: MediaKind,
    pub size_bytes: Option<i64>,
    pub duration_ms: Option<i64>,
    pub status: FileStatus,
    pub error: Option<String>,
    pub created_at: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Segment {
    pub id: String,
    pub file_id: String,
    pub index: i64,
    pub start_ms: i64,
    pub end_ms: i64,
    pub text_en: String,
    pub text_hangul: Option<String>,
    pub text_ko: Option<String>,
}
```
**분석 / Analysis (Level 2)**
- `MediaFile.path` 는 OS 절대경로. Tauri/web 양쪽 다른 의미를 가지지만 string 표현 통일. / OS path string.
- `Segment.text_en` 은 원문 (Whisper 인식 결과; 영어 가정이나 실제로는 인식 언어 그대로). 필드명 `text_en` 이 약간 미스리딩. / `text_en` actually holds source language transcription.

**짚어볼 점 / Watch-outs**
- `text_en` 명칭은 다국어 인식이 가능한 시스템에는 부정확. `text_source` 가 더 정직.

```rust path=src-tauri/src/core/models.rs lines=99-128
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Settings {
    pub whisper_model: String,
    pub translate_endpoint: String,
    pub translate_api_key: Option<String>,
    pub translator: String,
    pub ffmpeg_path: Option<String>,
}

impl Default for Settings {
    fn default() -> Self {
        Self {
            whisper_model: "base".to_string(),
            translate_endpoint: "https://api.mymemory.translated.net/get".to_string(),
            translate_api_key: None,
            translator: "mymemory".to_string(),
            ffmpeg_path: None,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct NewFileInput {
    pub name: String,
    pub path: String,
    pub kind: Option<String>,
    pub size_bytes: Option<i64>,
}
```
**분석 / Analysis (Level 2)**
- (CWE-312) `translate_api_key: Option<String>` 평문. SQLite kv_settings 에 JSON 으로 저장된다. 모바일/멀티유저 환경에서 위험. / Plaintext API key.
- 기본 번역기는 `mymemory` (anonymous, public API). 무료지만 일일 한도 있음.

**짚어볼 점 / Watch-outs**
- API key encryption 또는 OS keychain 활용 권장 (`keyring` crate).
- `whisper_model: String` 인데 enum 또는 검증 함수 없음 → 사용자가 잘못 입력하면 다운로드 단계에서 404.

```rust path=src-tauri/src/core/models.rs lines=130-154
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", tag = "type")]
pub enum PipelineEvent {
    #[serde(rename_all = "camelCase")]
    FileStarted { file_id: String },
    #[serde(rename_all = "camelCase")]
    ModelDownloading { progress: f32, message: String },
    #[serde(rename_all = "camelCase")]
    Progress {
        file_id: String,
        stage: String,
        message: String,
    },
    #[serde(rename_all = "camelCase")]
    Segment { segment: Segment },
    #[serde(rename_all = "camelCase")]
    FileDone {
        file_id: String,
        duration_ms: Option<i64>,
    },
    #[serde(rename_all = "camelCase")]
    FileError { file_id: String, error: String },
    #[serde(rename_all = "camelCase")]
    AllDone,
}
```
**분석 / Analysis (Level 2)**
- `#[serde(tag = "type")]` → 직렬화 결과는 `{ "type":"fileStarted", "fileId":"…" }` 같은 internally-tagged JSON. TS discriminated union 과 잘 맞음.
- 모든 variant 가 별도 rename_all → camelCase 일관성. / Consistent camelCase per variant.
- `Progress` 의 `stage` 는 string-typed; pipeline 코드에서 `"decode"/"asr"/"translate"` 하드코드 → enum 화 권장.

**짚어볼 점 / Watch-outs**
- 새 stage 추가 시 string 변경 시 프론트와 sync 깨지기 쉬움.

**파일 수준 평가 / File-Level Verdict**
- 강점 / Strengths:
  1. 모델 정의가 한 파일에 모여 도메인 일람 가능.
  2. camelCase JSON 호환으로 프론트 통합 매끄러움.
  3. `PipelineEvent` 가 internally-tagged enum 으로 TS 친화.
- 위험 / Risks:
  1. (CWE-312) `translate_api_key` 평문 저장.
  2. `parse` 의 silent fallback (Audio/Pending) 가 디버깅 신호 손실.
  3. `stage`, `whisper_model` 등 stringly-typed 필드.
- 개선 권고 / Recommendations:
  1. API key 는 keychain 사용; 디스크 저장 시 마스킹.
  2. enum + `try_from<&str>` 으로 변환 실패를 명시.
  3. `text_en` → `text_source` 로 리네임 검토.
- 재읽기 권고 레벨 / Suggested re-read level: 2.
