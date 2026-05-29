# web-server/src/pipeline.rs — 코드 리뷰 / Code Review

> 리뷰 레벨 / Review Level: 2
> [← INDEX.md](../../INDEX.md)

**파일 요약 / File Summary**
- 역할 / Role: 데스크톱 Pipeline 과 동일한 ETL 흐름. 이벤트 sink 만 `tokio::sync::broadcast` 채널로 바뀜.
- 진입점에서의 거리 / Distance from entrypoint: 2 (main → pipeline_run handler → Pipeline::run)
- 라인 수 / LOC: 155
- 주요 export / Exports: `Pipeline` (Clone)
- 의존 모듈 / Depends on (internal): `crate::core::*`
- 사용된 외부 패키지 / External packages used: `reqwest::Client`, `tokio::sync::broadcast`, `uuid::Uuid`
- 부수효과 / Side effects: HTTP, 파일 IO, Whisper, DB, broadcast send.
- 동시성 모델 / Concurrency model: async/await, broadcast 채널로 fan-out.
- 테스트 커버리지 흔적 / Test coverage signal: 없음.

```rust path=web-server/src/pipeline.rs lines=1-26
//! Web pipeline — same flow as the Tauri pipeline but emits events through a
//! `tokio::sync::broadcast` channel instead of `AppHandle::emit`.

use std::path::PathBuf;
use std::sync::Arc;

use reqwest::Client;
use tokio::sync::broadcast;
use uuid::Uuid;

use crate::core::asr::AsrEngine;
use crate::core::db::Db;
use crate::core::error::{AppError, AppResult};
use crate::core::media;
use crate::core::models::{FileStatus, PipelineEvent, Segment, Settings};
use crate::core::translate;
use crate::core::translit;

#[derive(Clone)]
pub struct Pipeline {
    pub db: Db,
    pub asr: Arc<AsrEngine>,
    pub http: Client,
    pub events: broadcast::Sender<PipelineEvent>,
}
```
**분석 / Analysis (Level 2)**
- 핵심 차이: `events: broadcast::Sender<PipelineEvent>` 가 desktop 의 `AppHandle::emit` 자리에 들어간다. / Broadcast replaces Tauri emit.
- `Pipeline` 자체가 `Clone` (desktop 은 Arc<Pipeline> 로 감싸짐). web-server 도 `Arc<Pipeline>` 로 사용 → 굳이 Clone 일 필요는 없지만 Sender/Client/Db 가 모두 cheap clone 이라 비용 적음.

```rust path=web-server/src/pipeline.rs lines=27-42
impl Pipeline {
    pub fn new(
        db: Db,
        asr: Arc<AsrEngine>,
        events: broadcast::Sender<PipelineEvent>,
    ) -> AppResult<Self> {
        let http = Client::builder()
            .user_agent("as-it-sounds-web/0.1")
            .build()
            .map_err(AppError::from)?;
        Ok(Self { db, asr, http, events })
    }

    fn emit(&self, e: PipelineEvent) {
        let _ = self.events.send(e);
    }
```
**분석 / Analysis (Level 2)**
- HTTP client timeout 미설정 (desktop 과 동일 결함). / Same timeout gap.
- `events.send(e)` 결과 무시: subscriber 0명이면 `SendError`. 정상 — 단지 “듣는 사람 없음”. / Ignored on purpose.

**짚어볼 점 / Watch-outs**
- subscriber 가 너무 느릴 때 broadcast lag → 일부 이벤트 손실. main.rs 의 SSE 핸들러도 lag 시 None 처리.

```rust path=web-server/src/pipeline.rs lines=44-67
    pub async fn run(&self, file_ids: Vec<String>, settings: Settings) -> AppResult<()> {
        let model_name = settings.whisper_model.clone();
        let tx = self.events.clone();
        let model_path = self
            .asr
            .ensure_model(&model_name, move |pct, msg| {
                let _ = tx.send(PipelineEvent::ModelDownloading {
                    progress: pct,
                    message: msg,
                });
            })
            .await?;

        let ffmpeg = media::resolve_ffmpeg(settings.ffmpeg_path.as_deref())?;

        for file_id in file_ids {
            let file = match self.db.get_file(&file_id)? {
                Some(f) => f,
                None => continue,
            };
            self.emit(PipelineEvent::FileStarted { file_id: file_id.clone() });
            self.db.set_file_status(&file_id, FileStatus::Processing, None)?;
            self.db.clear_segments(&file_id)?;
```
**분석 / Analysis (Level 2)**
- desktop `pipeline.rs` 와 동일한 흐름. 차이는 `Self::emit(&app, ...)` 대신 `self.emit(...)`. / Same flow.
- progress closure 가 `tx.clone()` 캡처 → `move` 클로저에 Sender 이동. broadcast Sender 는 `Clone`. / Sender cloned into closure.

```rust path=web-server/src/pipeline.rs lines=68-104
            let result: AppResult<()> = async {
                self.emit(PipelineEvent::Progress {
                    file_id: file_id.clone(),
                    stage: "decode".into(),
                    message: "오디오 추출 중...".into(),
                });
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

                let samples = media::extract_pcm_f32_mono_16k(&ffmpeg, &input_path).await?;

                self.emit(PipelineEvent::Progress {
                    file_id: file_id.clone(),
                    stage: "asr".into(),
                    message: "Whisper 추론 중...".into(),
                });
                let asr_segments = self
                    .asr
                    .transcribe(&model_name, model_path.clone(), samples)
                    .await?;

                if asr_segments.is_empty() {
                    return Err(AppError::Generic("인식된 세그먼트가 없습니다.".into()));
                }
```
**분석 / Analysis (Level 2)**
- decode → asr 단계가 desktop 과 1:1 동일. 변경 시 두 파일을 함께 유지해야 하는 **drift 위험**. / Duplicate code drift risk.
- `input_path.exists()` 검사. web-server 의 경우 path 가 `uploads_dir/<UUID>.<ext>` 이므로 항상 존재해야 함. 만약 사용자가 디스크에서 직접 지웠으면 여기서 잡힘.

**짚어볼 점 / Watch-outs**
- 데스크톱과 동일한 로직을 별도 파일로 유지 → trait 또는 generic event sink 로 통합 권장.

```rust path=web-server/src/pipeline.rs lines=106-131
                self.emit(PipelineEvent::Progress {
                    file_id: file_id.clone(),
                    stage: "translate".into(),
                    message: "번역·음차 변환 중...".into(),
                });

                let texts: Vec<String> = asr_segments.iter().map(|s| s.text.clone()).collect();
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
                    self.emit(PipelineEvent::Segment { segment });
                }
```
**분석 / Analysis (Level 2)**
- segment 별 DB insert + emit. desktop 과 동일 패턴. / Same per-segment pattern.
- broadcast send 가 subscriber 부재일 때 무시. SSE 가 lag 시 segment 일부 손실 가능 (UX 측면).

```rust path=web-server/src/pipeline.rs lines=133-155
                self.db.set_file_status(&file_id, FileStatus::Done, None)?;
                self.emit(PipelineEvent::FileDone {
                    file_id: file_id.clone(),
                    duration_ms: duration,
                });
                Ok(())
            }
            .await;

            if let Err(e) = result {
                let msg = e.to_string();
                self.db.set_file_status(&file_id, FileStatus::Error, Some(&msg))?;
                self.emit(PipelineEvent::FileError {
                    file_id: file_id.clone(),
                    error: msg,
                });
            }
        }

        self.emit(PipelineEvent::AllDone);
        Ok(())
    }
}
```
**분석 / Analysis (Level 2)**
- per-file try/catch + AllDone emit. desktop 과 동일. / Identical to desktop.

**짚어볼 점 / Watch-outs**
- 모델/ffmpeg pre-flight 실패 시 `?` 로 즉시 return → AllDone 안 보냄 → 프론트가 loading state 에 갇힐 위험 (desktop 과 동일 결함).

**파일 수준 평가 / File-Level Verdict**
- 강점 / Strengths:
  1. 데스크톱 pipeline 과 사실상 동일 → 한쪽 검증되면 다른쪽도 동작 보장.
  2. broadcast 채널로 다중 client SSE 지원.
  3. progress callback 까지 broadcast 로 통합.
- 위험 / Risks:
  1. (Code duplication) desktop `pipeline.rs` 와 거의 동일 → drift 위험.
  2. AllDone 미보장 (pre-flight 단계 실패 시).
  3. broadcast lag 시 이벤트 손실.
- 개선 권고 / Recommendations:
  1. `trait EventSink` 정의 후 desktop/web 가 다른 구현체 주입.
  2. `PipelineEvent::RunStarted/RunFailed` 추가.
  3. broadcast capacity 모니터링 메트릭.
- 재읽기 권고 레벨 / Suggested re-read level: 2.
