# src-tauri/src/core/pipeline.rs — 코드 리뷰 / Code Review

> 리뷰 레벨 / Review Level: 2
> [← INDEX.md](../../../INDEX.md)

**파일 요약 / File Summary**
- 역할 / Role: 오케스트레이터 — 모델 다운로드 → ffmpeg 디코드 → Whisper 추론 → 번역 → 음차 → DB persist → Tauri event emit.
- 진입점에서의 거리 / Distance from entrypoint: 2 (commands::pipeline_run → Pipeline::run)
- 라인 수 / LOC: 179
- 주요 export / Exports: `Pipeline`, `EVENT` (이벤트 채널 이름)
- 의존 모듈 / Depends on (internal): `asr`, `db`, `error`, `media`, `models`, `translate`, `translit`
- 사용된 외부 패키지 / External packages used: `reqwest::Client`, `tauri::{AppHandle, Emitter}`, `uuid::Uuid`
- 부수효과 / Side effects: HTTP, 파일 IO, Whisper 추론, DB, Tauri IPC emit.
- 동시성 모델 / Concurrency model: async/await. 파일 단위 순차 처리. 내부 `try` 블록으로 per-file error isolation.
- 테스트 커버리지 흔적 / Test coverage signal: 없음.

```rust path=src-tauri/src/core/pipeline.rs lines=1-22
use std::path::PathBuf;
use std::sync::Arc;

use reqwest::Client;
use tauri::{AppHandle, Emitter};
use uuid::Uuid;

use super::asr::AsrEngine;
use super::db::Db;
use super::error::{AppError, AppResult};
use super::media;
use super::models::{FileStatus, PipelineEvent, Segment, Settings};
use super::translate;
use super::translit;

pub const EVENT: &str = "pipeline://event";

pub struct Pipeline {
    pub db: Db,
    pub asr: Arc<AsrEngine>,
    pub http: Client,
}
```
**분석 / Analysis (Level 2)**
- `EVENT` 채널명 고정. 프론트는 `listen("pipeline://event", ...)` 으로 구독. / Single event channel.
- `db`/`asr`/`http` 모두 cheap clone (Arc/Connection-arc/Client 내부 arc). / All Arc-backed.

```rust path=src-tauri/src/core/pipeline.rs lines=24-35
impl Pipeline {
    pub fn new(db: Db, asr: Arc<AsrEngine>) -> AppResult<Self> {
        let http = Client::builder()
            .user_agent("as-it-sounds/0.1")
            .build()
            .map_err(AppError::from)?;
        Ok(Self { db, asr, http })
    }

    fn emit(app: &AppHandle, event: &PipelineEvent) {
        let _ = app.emit(EVENT, event);
    }
```
**분석 / Analysis (Level 2)**
- HTTP client 에 timeout 미설정 — translate.rs 와 동일한 문제 상속. / Same timeout gap as translate.
- `emit` 실패는 `_` 로 무시. Tauri 가 윈도우 닫힌 후 emit 하면 실패하는 정도. (CWE-252 minor).

```rust path=src-tauri/src/core/pipeline.rs lines=37-58
    pub async fn run(
        &self,
        app: AppHandle,
        file_ids: Vec<String>,
        settings: Settings,
    ) -> AppResult<()> {
        // Ensure model ready (with download progress) first.
        let model_name = settings.whisper_model.clone();
        let app_for_progress = app.clone();
        let model_path = self
            .asr
            .ensure_model(&model_name, move |pct, msg| {
                Self::emit(
                    &app_for_progress,
                    &PipelineEvent::ModelDownloading {
                        progress: pct,
                        message: msg,
                    },
                );
            })
            .await?;
```
**분석 / Analysis (Level 2)**
- 모델 다운로드 단계 에러는 여기서 `?` 로 전체 run 종료 → 모든 파일 처리 안 됨. 적절. / Pre-flight failure aborts whole run.
- progress callback closure 가 `app.clone()` 캡처 → AppHandle 은 cheap clone. / Cheap clone.

```rust path=src-tauri/src/core/pipeline.rs lines=59-74
        let ffmpeg = media::resolve_ffmpeg(settings.ffmpeg_path.as_deref())?;

        for file_id in file_ids {
            let file = match self.db.get_file(&file_id)? {
                Some(f) => f,
                None => continue,
            };
            Self::emit(
                &app,
                &PipelineEvent::FileStarted {
                    file_id: file_id.clone(),
                },
            );
            self.db.set_file_status(&file_id, FileStatus::Processing, None)?;
            self.db.clear_segments(&file_id)?;
```
**분석 / Analysis (Level 2)**
- `resolve_ffmpeg` 실패는 전체 run 종료 → 사용자는 ffmpeg 부재 메시지 받음.
- 알 수 없는 file_id 는 silently skip. 사용자에게 알림 없음. UI 가 stale list 가졌을 때 발생 가능.
- file_status 변경과 segments 정리는 DB 두 호출 — 트랜잭션 미사용. 중간 패닉이면 status 만 Processing, segments 안 지워짐.

**짚어볼 점 / Watch-outs**
- 알 수 없는 file_id 에 대해 `PipelineEvent::FileError` emit 권장.

```rust path=src-tauri/src/core/pipeline.rs lines=75-114
            let result: AppResult<()> = async {
                Self::emit(
                    &app,
                    &PipelineEvent::Progress {
                        file_id: file_id.clone(),
                        stage: "decode".into(),
                        message: "오디오 추출 중...".into(),
                    },
                );
                let input_path = PathBuf::from(&file.path);
                if !input_path.exists() {
                    return Err(AppError::Generic(format!(
                        "파일을 찾을 수 없습니다: {}",
                        file.path
                    )));
                }

                let duration = media::probe_duration_ms(&ffmpeg, &input_path)
                    .await
                    .ok()
                    .flatten();
                if let Some(d) = duration {
                    let _ = self.db.set_file_duration(&file_id, d);
                }

                let samples =
                    media::extract_pcm_f32_mono_16k(&ffmpeg, &input_path).await?;

                Self::emit(
                    &app,
                    &PipelineEvent::Progress {
                        file_id: file_id.clone(),
                        stage: "asr".into(),
                        message: "Whisper 추론 중...".into(),
                    },
                );
                let asr_segments = self
                    .asr
                    .transcribe(&model_name, model_path.clone(), samples)
                    .await?;
```
**분석 / Analysis (Level 2)**
- 익명 async 블록 패턴: try-catch 모방. 결과를 `result` 에 받아 마지막에 일괄 처리. / Inner async block = catch.
- `probe_duration_ms` 실패는 `.ok().flatten()` 으로 None — duration 못 알면 그냥 진행. OK.
- `set_file_duration` 결과는 `let _ =` 으로 swallow. duration update 실패는 비치명. / Minor swallow.
- 파일 존재 검증 후 ffmpeg 호출. 그러나 race 가능: 파일이 직후 삭제되어도 ffmpeg 가 적절 에러 반환.

**짚어볼 점 / Watch-outs**
- (CWE-22) `file.path` 가 DB 에 신뢰 입력으로 들어가 있음. desktop 가정에서는 단일 사용자라 자기책임. web-server 는 multipart 가 uploads_dir 로 강제.
- 큰 PCM `samples` 가 transcribe 에 move → 메모리 점유 큼. drop 시점은 transcribe 종료 후.

```rust path=src-tauri/src/core/pipeline.rs lines=116-149
                if asr_segments.is_empty() {
                    return Err(AppError::Generic("인식된 세그먼트가 없습니다.".into()));
                }

                Self::emit(
                    &app,
                    &PipelineEvent::Progress {
                        file_id: file_id.clone(),
                        stage: "translate".into(),
                        message: "번역·음차 변환 중...".into(),
                    },
                );

                let texts: Vec<String> =
                    asr_segments.iter().map(|s| s.text.clone()).collect();
                let ko_translations =
                    translate::translate_many(&settings, &self.http, &texts).await;

                for (i, seg) in asr_segments.into_iter().enumerate() {
                    let hangul = translit::transliterate(&seg.text);
                    let ko = ko_translations.get(i).and_then(|o| o.clone());
                    let segment = Segment {
                        id: Uuid::new_v4().to_string(),
                        file_id: file_id.clone(),
                        index: i as i64,
                        start_ms: seg.start_ms,
                        end_ms: seg.end_ms,
                        text_en: seg.text,
                        text_hangul: Some(hangul),
                        text_ko: ko,
                    };
                    self.db.insert_segment(&segment)?;
                    Self::emit(&app, &PipelineEvent::Segment { segment });
                }
```
**분석 / Analysis (Level 2)**
- 번역 결과 길이가 `asr_segments.len()` 과 일치한다는 보장은 `translate_many` 의 구현에 달림 (Vec::with_capacity + push). `.get(i)` 로 안전 인덱싱.
- 각 segment 마다 DB insert + emit. 100 개 segment = 100 회 lock — db.rs 의 bulk insert 권장 항목과 연결.
- emit 직전에 segment move → `Self::emit(&app, &Event { segment })` 직렬화 후 broadcast.

**짚어볼 점 / Watch-outs**
- segment 처리 도중 `?` 로 중도 종료되면 일부 segment 만 저장됨 + status 는 Processing → outer catch 가 Error 로 마무리하지만 부분 데이터가 남음. 일반적 ETL 패턴(append 모드) 으로 봐도 OK.

```rust path=src-tauri/src/core/pipeline.rs lines=151-179
                self.db.set_file_status(&file_id, FileStatus::Done, None)?;
                Self::emit(
                    &app,
                    &PipelineEvent::FileDone {
                        file_id: file_id.clone(),
                        duration_ms: duration,
                    },
                );
                Ok(())
            }
            .await;

            if let Err(e) = result {
                let msg = e.to_string();
                self.db.set_file_status(&file_id, FileStatus::Error, Some(&msg))?;
                Self::emit(
                    &app,
                    &PipelineEvent::FileError {
                        file_id: file_id.clone(),
                        error: msg,
                    },
                );
            }
        }

        Self::emit(&app, &PipelineEvent::AllDone);
        Ok(())
    }
}
```
**분석 / Analysis (Level 2)**
- `if let Err(e) = result { ... }` 외부 루프 계속 → 다음 파일 처리. 좋은 isolation. / Per-file isolation, continue on failure.
- `set_file_status(... Error, msg)` 가 다시 `?` → 이 update 자체가 실패하면 `run` 전체 종료. 드물지만 가능.
- 마지막 `AllDone` 은 항상 emit (앞단 종료가 아니라면). UX 적 closure.

**짚어볼 점 / Watch-outs**
- run 시작 전 모델/ffmpeg 단계에서 에러로 즉시 return 하면 `AllDone` emit 안 됨 → 프론트 loading 상태가 영구일 가능. 별도 “run started/run failed” 이벤트 권장.

**파일 수준 평가 / File-Level Verdict**
- 강점 / Strengths:
  1. async-block try/catch 패턴으로 파일별 격리.
  2. Progress 이벤트 stage 별로 emit → UX 진행도 표시.
  3. 모델/ffmpeg 사전 검증으로 early-fail.
- 위험 / Risks:
  1. `AllDone` 보장 안 됨 (pre-flight 실패 시 frontend stuck).
  2. segment 마다 DB lock → 다량 segment 성능 저하.
  3. status update 가 transactional 이 아니라 부분 상태 잔존 가능.
- 개선 권고 / Recommendations:
  1. `PipelineEvent::RunStarted/RunFailed` 추가하여 phase 명시.
  2. segment 묶음 transactional insert.
  3. 알 수 없는 file_id 에 대해 명시적 FileError emit.
- 재읽기 권고 레벨 / Suggested re-read level: 2.
