//! Re-exports the desktop core via `#[path]` so the same Whisper/ffmpeg/translit
//! logic powers both the Tauri app and this HTTP server. Pipeline is NOT
//! re-exported here — the web server defines its own pipeline with a broadcast
//! event sink (see `crate::pipeline`) instead of `tauri::AppHandle`.

#[path = "../../src-tauri/src/core/error.rs"]
pub mod error;

#[path = "../../src-tauri/src/core/models.rs"]
pub mod models;

#[path = "../../src-tauri/src/core/translit.rs"]
pub mod translit;

#[path = "../../src-tauri/src/core/translate.rs"]
pub mod translate;

#[path = "../../src-tauri/src/core/db.rs"]
pub mod db;

#[path = "../../src-tauri/src/core/asr.rs"]
pub mod asr;

#[path = "../../src-tauri/src/core/media.rs"]
pub mod media;

#[path = "../../src-tauri/src/core/settings.rs"]
pub mod settings;

pub use error::{AppError, AppResult};
