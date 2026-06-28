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

            let result: AppResult<()> = async {
                Self::emit(
                    &app,
                    &PipelineEvent::Progress {
                        file_id: file_id.clone(),
                        stage: "decode".into(),
                        message: "오디오 추출 중...".into(),
                        percent: None,
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
                        percent: None,
                    },
                );
                let asr_segments = self
                    .asr
                    .transcribe(&model_name, model_path.clone(), samples)
                    .await?;

                if asr_segments.is_empty() {
                    return Err(AppError::Generic("인식된 세그먼트가 없습니다.".into()));
                }

                let texts: Vec<String> =
                    asr_segments.iter().map(|s| s.text.clone()).collect();
                let ko_translations =
                    translate::translate_many(&settings, &self.http, &texts).await;

                let total = asr_segments.len();
                Self::emit(
                    &app,
                    &PipelineEvent::Progress {
                        file_id: file_id.clone(),
                        stage: "translate".into(),
                        message: format!("번역·음차 변환 중... (0/{})", total),
                        percent: Some(0.0),
                    },
                );

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
                    let percent = (i + 1) as f32 / total as f32;
                    Self::emit(
                        &app,
                        &PipelineEvent::Progress {
                            file_id: file_id.clone(),
                            stage: "translate".into(),
                            message: format!("번역·음차 변환 중... ({}/{})", i + 1, total),
                            percent: Some(percent),
                        },
                    );
                }

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
