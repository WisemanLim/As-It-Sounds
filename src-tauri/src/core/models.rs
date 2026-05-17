use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Project {
    pub id: String,
    pub name: String,
    pub created_at: i64,
    pub updated_at: i64,
}

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
