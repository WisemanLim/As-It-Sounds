# src-tauri/src/core/asr.rs — 코드 리뷰 / Code Review

> 리뷰 레벨 / Review Level: 2
> [← INDEX.md](../../../INDEX.md)

**파일 요약 / File Summary**
- 역할 / Role: `AsrEngine` — Whisper 모델 다운로드/캐시/추론 wrapper. PCM f32 16kHz 입력 → segment 출력.
- 진입점에서의 거리 / Distance from entrypoint: 3 (lib → setup → pipeline → asr)
- 라인 수 / LOC: 187
- 주요 export / Exports: `AsrEngine`, `AsrSegment`
- 의존 모듈 / Depends on (internal): `super::error`
- 사용된 외부 패키지 / External packages used: `whisper_rs`, `reqwest`, `tokio`, `futures_util`, `parking_lot::Mutex`
- 부수효과 / Side effects: 디스크 IO(모델 다운로드/캐시), 네트워크 IO(huggingface), 큰 메모리 사용(모델 로드), CPU/GPU 가속.
- 동시성 모델 / Concurrency model: `Arc<Mutex<Option<CachedCtx>>>` + `tokio::task::spawn_blocking`. mutex 가 추론 전 구간을 보유 → 동시 추론 직렬화.
- 테스트 커버리지 흔적 / Test coverage signal: 없음.

```rust path=src-tauri/src/core/asr.rs lines=1-15
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
```
**분석 / Analysis (Level 2)**
- `parking_lot::Mutex` 는 `std::sync::Mutex` 보다 빠르고 **poison 개념이 없음** (panic 발생 시 mutex 가 영구 잠기지 않음). / No poisoning concept.
- `AsrSegment` 는 self-contained POD. 외부로 노출돼 pipeline 이 사용.

```rust path=src-tauri/src/core/asr.rs lines=17-34
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
```
**분석 / Analysis (Level 2)**
- `ctx` 는 옵션. 첫 호출 시 lazy 로드. 다른 모델로 전환 시 교체. / Lazy load; one model cached at a time.
- `Arc` 가 굳이 필요? `AsrEngine` 자체가 `Arc<AsrEngine>` 로 lib.rs 에서 감싸이므로 내부 Mutex 는 굳이 `Arc` 로 감쌀 필요 없음. 단, `transcribe` 가 `self.ctx.clone()` 으로 spawn_blocking 에 넘기기 위해 `Arc` 필요. / `Arc` is needed for the `spawn_blocking` move.

```rust path=src-tauri/src/core/asr.rs lines=36-45
    pub fn model_path(&self, model: &str) -> PathBuf {
        self.models_dir.join(format!("ggml-{}.bin", model))
    }

    pub fn model_url(model: &str) -> String {
        format!(
            "https://huggingface.co/ggerganov/whisper.cpp/resolve/main/ggml-{}.bin",
            model
        )
    }
```
**분석 / Analysis (Level 2)**
- (CWE-22 path traversal) `model` 문자열이 사용자 입력에서 옴(Settings.whisper_model). `"../etc/passwd"` 같은 값이 들어오면 `models_dir.join(...)` 로 상위 디렉토리 접근 가능. 단, 실제로는 file write 전에 hf URL 호출하므로 거기서 404. 그래도 검증이 더 안전. / Path traversal possible via Settings input.
- URL 도 `model` 그대로 삽입 → HTTP injection 가능성 거의 없음(`reqwest::get` 이 URL 파싱).

**짚어볼 점 / Watch-outs**
- `model` 입력은 영숫자 + `-` + `.` 만 허용하도록 사전 검증 권장.

```rust path=src-tauri/src/core/asr.rs lines=47-98
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
```
**분석 / Analysis (Level 2)**
- 캐시 검사 → 다운로드 → 임시 파일 → rename. 원자적 교체. / Atomic replace via rename — good pattern.
- 진행률은 2% 마다 callback. UI 스팸 방지. / 2% throttle on progress.
- `client = reqwest::Client::builder().build()` 매번 새로 빌드. 매 다운로드 마다 새 client 인데, 모델은 한번 받으면 끝이라 큰 부담 아님. 다만 timeout 미설정 → 끊긴 네트워크면 영원히 대기 (CWE-1088).
- 에러 발생 시 `tmp_path` 가 남아 디스크에 쓰레기 누적. cleanup 없음 (CWE-459 resource leak — minor).

**짚어볼 점 / Watch-outs**
- `Client::builder().timeout(...)` 또는 `connect_timeout` 설정 필요. 무한 대기는 UX 망침.
- 다운로드 도중 cancel 메커니즘 없음. UX 측면 보강 권장.
- TLS verification 은 기본 설정으로 활성화. OK.

```rust path=src-tauri/src/core/asr.rs lines=100-117
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
```
**분석 / Analysis (Level 2)**
- mutex 보유 중 `WhisperContext::new_with_params` 호출. 모델 로드는 수백 MB 메모리 매핑 + 수백 ms 소요 → 모든 다른 transcribe 호출이 그동안 대기.
- 동기 함수, 내부에 blocking IO 있음. async 코드에서 호출되면 tokio thread block (CWE-?). 다만 호출자 `transcribe` 가 직후 `spawn_blocking` 에서 추론을 돌리므로 첫 로드만 main runtime 에 영향. / Loads model on Tokio runtime thread — could block briefly.

**짚어볼 점 / Watch-outs**
- `load_or_reuse` 자체를 `spawn_blocking` 로 감싸는 게 더 안전.

```rust path=src-tauri/src/core/asr.rs lines=119-180
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
```
**분석 / Analysis (Level 2)**
- 핵심 동시성 패턴: `spawn_blocking` 안에서 `parking_lot::Mutex` 보유 → 다른 transcribe 호출은 직렬화. / Serialized transcription.
- **중요**: 작업 스레드가 mutex 를 들고 `state.full(...)` (수초~수분 가능) 호출 → 동시에 다른 task 가 `load_or_reuse` 도 진입할 수 없음. **Fairness**: tokio mutex 가 아니라 OS 락이므로 FIFO 보장 안 됨. 가벼운 요청이 무거운 추론 뒤에 무한 대기 가능.
- `params.set_language(None)` → 자동 감지. translate=false → 원문 언어 그대로.
- `num_cpus()` 가 8 로 clamp → CPU 8코어 이상 머신에선 보수적. ARM Mac M1 8코어 적합.

**짚어볼 점 / Watch-outs**
- (Concurrency) 큰 파일 추론 도중 사용자가 `translit_preview` 등 다른 빠른 작업을 부르면 OK (mutex 미공유). 다만 `pipeline_run` 두 번 누르면 모두 직렬. UX 메시지 필요.
- (Cancel) `state.full` 은 abort 불가. 큰 파일 처리 중 사용자 취소 못함.
- (iOS) `whisper-rs` 는 metal 가속을 켜면 iOS 시뮬레이터에서 동작하나, FFI 빌드는 ios target 으로 cross-compile 필요. 별도 확인 필요.

```rust path=src-tauri/src/core/asr.rs lines=182-187
fn num_cpus() -> std::os::raw::c_int {
    let n = std::thread::available_parallelism()
        .map(|n| n.get())
        .unwrap_or(4);
    n.min(8) as std::os::raw::c_int
}
```
**분석 / Analysis (Level 2)**
- 8 clamp 의미: 더 많은 스레드는 whisper.cpp 에서 효율 떨어짐 (cache miss). / Empirical clamp.
- `c_int` 캐스트 → whisper-rs FFI 인자 타입.

**파일 수준 평가 / File-Level Verdict**
- 강점 / Strengths:
  1. 모델 다운로드의 원자적 rename, 진행률 throttle 잘 됨.
  2. `spawn_blocking` 로 Whisper 동기 추론을 tokio 와 안전하게 분리.
  3. 모델 캐시 (이름 일치 시 재사용) 로직.
- 위험 / Risks:
  1. Mutex 가 추론 전 구간 보유 → 동시 호출 시 직렬화 + fairness 부재.
  2. 다운로드 timeout 없음(무한 대기 가능, CWE-1088).
  3. `model` 사용자 입력의 path traversal 가능성 (낮은 실위험).
- 개선 권고 / Recommendations:
  1. `Client::builder().connect_timeout(...).timeout(...)` 추가.
  2. 추론 직전 ctx 를 클론(가능하면) 또는 RwLock + 작업별 state 분리로 fairness 개선.
  3. `model` 입력 화이트리스트(`tiny|base|small|medium|large`) 검증.
- 재읽기 권고 레벨 / Suggested re-read level: 3 — 동시성, whisper-rs 빌드 옵션(GPU/Metal/iOS) 까지 보려면.
