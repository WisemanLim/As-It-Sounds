# src-tauri/src/core/ — 디렉토리 개요 / Directory Abstract

> [← INDEX.md](../../../INDEX.md)

## 파일 목록 / File List
| 파일 / File | LOC | 역할 / Role | 리뷰 링크 / Review |
|---|---|---|---|
| mod.rs | 11 | 모듈 선언 + `pub use error::*` | [mod.rs.md](mod.rs.md) |
| error.rs | 58 | `AppError` enum, `AppResult` | [error.rs.md](error.rs.md) |
| models.rs | 154 | Project/MediaFile/Segment/Settings/PipelineEvent | [models.rs.md](models.rs.md) |
| db.rs | 341 | SQLite (rusqlite) CRUD | [db.rs.md](db.rs.md) |
| asr.rs | 187 | Whisper engine (`whisper-rs`) | [asr.rs.md](asr.rs.md) |
| media.rs | 131 | ffmpeg PCM 추출 | [media.rs.md](media.rs.md) |
| translate.rs | 110 | MyMemory / LibreTranslate 어댑터 | [translate.rs.md](translate.rs.md) |
| translit.rs | 1713 | 다국어 phonetic → Hangul (사전 + 룰) | [translit.rs.md](translit.rs.md) |
| pipeline.rs | 179 | ETL 오케스트레이터 (Tauri 전용) | [pipeline.rs.md](pipeline.rs.md) |
| settings.rs | 21 | settings JSON kv wrapper | [settings.rs.md](settings.rs.md) |

## 디렉토리 구조 / Structure Summary
도메인의 단일 진리원천. 파이프라인 한 사이클은 `models::Settings` 로 시작해 `asr::ensure_model` → `media::extract_pcm_f32_mono_16k` → `asr::transcribe` → `translate::translate_many` + `translit::transliterate` → `db::insert_segment` 순. `pipeline.rs` 만 Tauri 의 `AppHandle` 의존 → web-server 는 이 파일을 제외하고 나머지를 `#[path]` 로 import.

## 주요 의존 관계 / Key Dependencies
- 외부: `whisper-rs` (FFI to whisper.cpp), `rusqlite`, `reqwest`, `tokio`, `parking_lot::Mutex`, `once_cell::Lazy`, `uuid`, `serde`/`serde_json`, `thiserror`.
- 내부: `pipeline → {asr, media, translate, translit, db}`; `db ↔ models ↔ error`; `settings → db`.
- 시스템: ffmpeg 외부 바이너리 (macOS/Linux/Windows 필요), HTTPS 외부 (huggingface 모델 다운로드, 번역 API).

## 위험 신호 / Risk Signals
- (Concurrency) `asr.rs` 의 `Arc<Mutex<Option<CachedCtx>>>` 가 추론 전 구간 보유 → 동시 호출 직렬화 + fairness 부재.
- (Portability) `media.rs` 의 ffmpeg spawn 은 iOS 미지원.
- (Performance) `db.rs` 단일 connection mutex + segment 별 lock churn — 대용량 segment 처리 시 병목.
- (Code dup) `pipeline.rs` 는 `web-server/src/pipeline.rs` 와 80%+ 동일 — drift 위험.
