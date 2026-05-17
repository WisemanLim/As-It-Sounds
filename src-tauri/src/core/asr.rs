use std::path::{Path, PathBuf};
use std::sync::Arc;

use futures_util::StreamExt;
use parking_lot::Mutex;
use tokio::io::AsyncWriteExt;
use whisper_rs::{FullParams, SamplingStrategy, WhisperContext, WhisperContextParameters};

use super::error::{AppError, AppResult};

pub struct AsrSegment {
    pub start_ms: i64,
    pub end_ms: i64,
    pub text: String,
}

pub struct AsrEngine {
    models_dir: PathBuf,
    ctx: Arc<Mutex<Option<CachedCtx>>>,
}

struct CachedCtx {
    name: String,
    ctx: WhisperContext,
}

impl AsrEngine {
    pub fn new(models_dir: PathBuf) -> AppResult<Self> {
        std::fs::create_dir_all(&models_dir)?;
        Ok(Self {
            models_dir,
            ctx: Arc::new(Mutex::new(None)),
        })
    }

    pub fn model_path(&self, model: &str) -> PathBuf {
        self.models_dir.join(format!("ggml-{}.bin", model))
    }

    pub fn model_url(model: &str) -> String {
        format!(
            "https://huggingface.co/ggerganov/whisper.cpp/resolve/main/ggml-{}.bin",
            model
        )
    }

    /// Ensure the model file exists; download it if missing. Reports download
    /// progress via the supplied callback (0.0..1.0).
    pub async fn ensure_model<F>(
        &self,
        model: &str,
        mut progress: F,
    ) -> AppResult<PathBuf>
    where
        F: FnMut(f32, String) + Send,
    {
        let path = self.model_path(model);
        if path.exists() && std::fs::metadata(&path).map(|m| m.len() > 0).unwrap_or(false) {
            return Ok(path);
        }

        progress(0.0, format!("Whisper 모델 다운로드 중: {model}"));
        let url = Self::model_url(model);
        let client = reqwest::Client::builder()
            .build()
            .map_err(AppError::from)?;
        let resp = client.get(&url).send().await?.error_for_status()?;
        let total = resp.content_length().unwrap_or(0);
        let tmp_path = path.with_extension("bin.download");
        let mut file = tokio::fs::File::create(&tmp_path).await?;
        let mut stream = resp.bytes_stream();
        let mut downloaded: u64 = 0;
        let mut last_report = 0.0_f32;
        while let Some(chunk) = stream.next().await {
            let chunk = chunk?;
            file.write_all(&chunk).await?;
            downloaded += chunk.len() as u64;
            if total > 0 {
                let pct = downloaded as f32 / total as f32;
                if pct - last_report > 0.02 || (pct - 1.0).abs() < f32::EPSILON {
                    last_report = pct;
                    progress(
                        pct,
                        format!(
                            "Whisper 모델 다운로드 중: {model} ({:.1}/{:.1} MB)",
                            downloaded as f32 / 1_048_576.0,
                            total as f32 / 1_048_576.0,
                        ),
                    );
                }
            }
        }
        file.flush().await?;
        drop(file);
        tokio::fs::rename(&tmp_path, &path).await?;
        progress(1.0, "모델 준비 완료".into());
        Ok(path)
    }

    fn load_or_reuse(&self, model_name: &str, path: &Path) -> AppResult<()> {
        let mut guard = self.ctx.lock();
        if let Some(c) = guard.as_ref() {
            if c.name == model_name {
                return Ok(());
            }
        }
        let ctx = WhisperContext::new_with_params(
            path.to_string_lossy().as_ref(),
            WhisperContextParameters::default(),
        )
        .map_err(|e| AppError::Whisper(e.to_string()))?;
        *guard = Some(CachedCtx {
            name: model_name.to_string(),
            ctx,
        });
        Ok(())
    }

    /// Run ASR over the given PCM samples (16kHz mono f32).
    pub async fn transcribe(
        &self,
        model_name: &str,
        model_path: PathBuf,
        samples: Vec<f32>,
    ) -> AppResult<Vec<AsrSegment>> {
        self.load_or_reuse(model_name, &model_path)?;

        let ctx_arc = self.ctx.clone();
        let segments = tokio::task::spawn_blocking(move || -> AppResult<Vec<AsrSegment>> {
            let guard = ctx_arc.lock();
            let cached = guard
                .as_ref()
                .ok_or_else(|| AppError::ModelNotReady)?;
            let mut state = cached
                .ctx
                .create_state()
                .map_err(|e| AppError::Whisper(e.to_string()))?;
            let mut params = FullParams::new(SamplingStrategy::Greedy { best_of: 1 });
            params.set_n_threads(num_cpus());
            params.set_language(None); // auto-detect: any language
            params.set_translate(false);
            params.set_print_progress(false);
            params.set_print_special(false);
            params.set_print_realtime(false);
            params.set_print_timestamps(false);
            params.set_suppress_blank(true);

            state
                .full(params, &samples)
                .map_err(|e| AppError::Whisper(e.to_string()))?;

            let num = state
                .full_n_segments()
                .map_err(|e| AppError::Whisper(e.to_string()))?;
            let mut out = Vec::with_capacity(num as usize);
            for i in 0..num {
                let text = state
                    .full_get_segment_text(i)
                    .map_err(|e| AppError::Whisper(e.to_string()))?;
                let t0 = state
                    .full_get_segment_t0(i)
                    .map_err(|e| AppError::Whisper(e.to_string()))?;
                let t1 = state
                    .full_get_segment_t1(i)
                    .map_err(|e| AppError::Whisper(e.to_string()))?;
                // whisper timestamps are in centiseconds (10ms).
                out.push(AsrSegment {
                    start_ms: (t0 as i64) * 10,
                    end_ms: (t1 as i64) * 10,
                    text: text.trim().to_string(),
                });
            }
            Ok(out)
        })
        .await
        .map_err(|e| AppError::Generic(format!("join error: {e}")))??;

        Ok(segments)
    }
}

fn num_cpus() -> std::os::raw::c_int {
    let n = std::thread::available_parallelism()
        .map(|n| n.get())
        .unwrap_or(4);
    n.min(8) as std::os::raw::c_int
}
