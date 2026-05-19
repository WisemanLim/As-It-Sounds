//! As It Sounds — HTTP/SSE server.
//!
//! Routes mirror the desktop Tauri commands so the React frontend can reuse the
//! same `ipc` shape (just over HTTP). Pipeline events flow through a single
//! broadcast channel served as Server-Sent Events at `/api/events`.

mod core;
mod pipeline;

use std::convert::Infallible;
use std::net::SocketAddr;
use std::path::PathBuf;
use std::sync::Arc;
use std::time::Duration;

use axum::body::Body;
use axum::extract::{DefaultBodyLimit, Multipart, Path as AxPath, Request, State};
use axum::http::{header, StatusCode};
use axum::response::sse::{Event, KeepAlive, Sse};
use axum::response::{IntoResponse, Response};
use axum::routing::{delete, get, patch, post};
use axum::{Json, Router};
use futures_util::stream::{Stream, StreamExt as _};
use serde::{Deserialize, Serialize};
use tokio::io::AsyncWriteExt;
use tokio::sync::broadcast;
use tokio_stream::wrappers::BroadcastStream;
use tower::ServiceExt;
use tower_http::cors::{Any, CorsLayer};
use tower_http::services::ServeFile;
use tower_http::trace::TraceLayer;
use uuid::Uuid;

use crate::core::asr::AsrEngine;
use crate::core::db::Db;
use crate::core::error::AppError;
use crate::core::models::{
    FileStatus, MediaFile, MediaKind, PipelineEvent, Project, Segment, Settings,
};
use crate::core::settings as settings_store;
use crate::core::translit;
use crate::pipeline::Pipeline;

// ─── App state ───────────────────────────────────────────────────────────────

#[derive(Clone)]
struct AppState {
    db: Db,
    pipeline: Arc<Pipeline>,
    events: broadcast::Sender<PipelineEvent>,
    uploads_dir: Arc<PathBuf>,
}

// ─── Error → response ────────────────────────────────────────────────────────

struct ApiError(AppError);

impl From<AppError> for ApiError {
    fn from(e: AppError) -> Self {
        Self(e)
    }
}

impl IntoResponse for ApiError {
    fn into_response(self) -> Response {
        let body = serde_json::json!({ "error": self.0.to_string() });
        (StatusCode::INTERNAL_SERVER_ERROR, Json(body)).into_response()
    }
}

type ApiResult<T> = Result<T, ApiError>;

// ─── Entry ───────────────────────────────────────────────────────────────────

#[tokio::main]
async fn main() {
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "info,axum=info,tower_http=info".into()),
        )
        .init();

    let data_dir = std::env::var("AIS_DATA_DIR")
        .map(PathBuf::from)
        .unwrap_or_else(|_| PathBuf::from("/data"));
    std::fs::create_dir_all(&data_dir).expect("create data dir");
    let uploads = data_dir.join("uploads");
    std::fs::create_dir_all(&uploads).expect("create uploads dir");
    let models = data_dir.join("models");
    std::fs::create_dir_all(&models).expect("create models dir");
    let db_path = data_dir.join("as-it-sounds.sqlite");
    let db = Db::open(&db_path).expect("open db");
    let asr = Arc::new(AsrEngine::new(models).expect("init asr"));
    let (tx, _rx) = broadcast::channel::<PipelineEvent>(1024);
    let pipeline =
        Arc::new(Pipeline::new(db.clone(), asr, tx.clone()).expect("init pipeline"));

    let state = AppState {
        db,
        pipeline,
        events: tx,
        uploads_dir: Arc::new(uploads),
    };

    let max_mb: usize = std::env::var("AIS_MAX_UPLOAD_MB")
        .ok()
        .and_then(|s| s.parse().ok())
        .unwrap_or(2048);

    let api = Router::new()
        .route("/healthz", get(healthz))
        .route("/projects", get(projects_list).post(projects_create))
        .route(
            "/projects/:id",
            patch(projects_rename).delete(projects_delete),
        )
        .route(
            "/projects/:id/files",
            get(files_list).post(files_upload),
        )
        .route("/files/:id", delete(files_remove))
        .route("/files/:id/reset", post(files_reset))
        .route("/files/:id/segments", get(segments_list))
        .route("/files/:id/media", get(files_media))
        .route("/settings", get(settings_get).put(settings_set))
        .route("/pipeline/run", post(pipeline_run))
        .route("/translit/preview", post(translit_preview))
        .route("/events", get(events_sse));

    let app = Router::new()
        .nest("/api", api)
        .layer(DefaultBodyLimit::max(max_mb * 1024 * 1024))
        .layer(
            CorsLayer::new()
                .allow_origin(Any)
                .allow_methods(Any)
                .allow_headers(Any),
        )
        .layer(TraceLayer::new_for_http())
        .with_state(state);

    let bind = std::env::var("AIS_BIND").unwrap_or_else(|_| "0.0.0.0:8080".into());
    let addr: SocketAddr = bind.parse().expect("parse bind");
    tracing::info!("listening on {addr}");
    let listener = tokio::net::TcpListener::bind(addr).await.expect("bind");
    axum::serve(listener, app).await.expect("serve");
}

// ─── Handlers ────────────────────────────────────────────────────────────────

async fn healthz() -> &'static str {
    "ok"
}

async fn projects_list(State(s): State<AppState>) -> ApiResult<Json<Vec<Project>>> {
    Ok(Json(s.db.list_projects()?))
}

#[derive(Deserialize)]
struct CreateProjectBody {
    name: Option<String>,
}
async fn projects_create(
    State(s): State<AppState>,
    Json(body): Json<CreateProjectBody>,
) -> ApiResult<Json<Project>> {
    Ok(Json(s.db.create_project(body.name)?))
}

#[derive(Deserialize)]
struct RenameProjectBody {
    name: String,
}
async fn projects_rename(
    State(s): State<AppState>,
    AxPath(id): AxPath<String>,
    Json(body): Json<RenameProjectBody>,
) -> ApiResult<StatusCode> {
    s.db.rename_project(&id, &body.name)?;
    Ok(StatusCode::NO_CONTENT)
}

async fn projects_delete(
    State(s): State<AppState>,
    AxPath(id): AxPath<String>,
) -> ApiResult<StatusCode> {
    s.db.delete_project(&id)?;
    Ok(StatusCode::NO_CONTENT)
}

async fn files_list(
    State(s): State<AppState>,
    AxPath(id): AxPath<String>,
) -> ApiResult<Json<Vec<MediaFile>>> {
    Ok(Json(s.db.list_files(&id)?))
}

/// Multipart upload: any number of file parts. Each is stored under
/// `<uploads_dir>/<file_id><ext>` and an associated `media_files` row is
/// inserted. Returns the array of created MediaFile rows (matches the desktop
/// `file_add_many` return shape).
async fn files_upload(
    State(s): State<AppState>,
    AxPath(project_id): AxPath<String>,
    mut multipart: Multipart,
) -> ApiResult<Json<Vec<MediaFile>>> {
    let mut prepared: Vec<(String, String, MediaKind, Option<i64>)> = Vec::new();

    while let Some(field) = multipart
        .next_field()
        .await
        .map_err(|e| AppError::Generic(format!("multipart: {e}")))?
    {
        let original_name = field
            .file_name()
            .map(|s| s.to_string())
            .unwrap_or_else(|| format!("upload-{}", Uuid::new_v4()));
        let ext = std::path::Path::new(&original_name)
            .extension()
            .and_then(|x| x.to_str())
            .unwrap_or("bin")
            .to_string();
        let file_id_basename = Uuid::new_v4().to_string();
        let stored = s.uploads_dir.join(format!("{file_id_basename}.{ext}"));
        let mut f = tokio::fs::File::create(&stored)
            .await
            .map_err(AppError::from)?;
        let mut written: u64 = 0;
        let mut stream = field;
        while let Some(chunk) = stream
            .chunk()
            .await
            .map_err(|e| AppError::Generic(format!("multipart chunk: {e}")))?
        {
            f.write_all(&chunk).await.map_err(AppError::from)?;
            written += chunk.len() as u64;
        }
        f.flush().await.map_err(AppError::from)?;
        let kind = MediaKind::from_name(&original_name);
        prepared.push((
            original_name,
            stored.to_string_lossy().into_owned(),
            kind,
            Some(written as i64),
        ));
    }

    if prepared.is_empty() {
        return Ok(Json(vec![]));
    }
    let created = s.db.add_files(&project_id, &prepared)?;
    Ok(Json(created))
}

async fn files_remove(
    State(s): State<AppState>,
    AxPath(id): AxPath<String>,
) -> ApiResult<StatusCode> {
    // Best-effort cleanup of the on-disk upload before removing the row.
    if let Ok(Some(f)) = s.db.get_file(&id) {
        let _ = tokio::fs::remove_file(&f.path).await;
    }
    s.db.remove_file(&id)?;
    Ok(StatusCode::NO_CONTENT)
}

async fn files_reset(
    State(s): State<AppState>,
    AxPath(id): AxPath<String>,
) -> ApiResult<StatusCode> {
    s.db.clear_segments(&id)?;
    s.db.set_file_status(&id, FileStatus::Pending, None)?;
    Ok(StatusCode::NO_CONTENT)
}

async fn segments_list(
    State(s): State<AppState>,
    AxPath(id): AxPath<String>,
) -> ApiResult<Json<Vec<Segment>>> {
    Ok(Json(s.db.list_segments(&id)?))
}

/// Serve the original uploaded media file. Forwards the request to
/// `tower_http::services::ServeFile` so HTTP Range / If-Modified-Since are
/// handled automatically (needed for `<audio>` / `<video>` seeking).
async fn files_media(
    State(s): State<AppState>,
    AxPath(id): AxPath<String>,
    req: Request<Body>,
) -> Response {
    let file = match s.db.get_file(&id) {
        Ok(Some(f)) => f,
        Ok(None) => return (StatusCode::NOT_FOUND, "file not found").into_response(),
        Err(e) => return ApiError(e).into_response(),
    };
    let mime = mime_guess::from_path(&file.name)
        .first_or_octet_stream()
        .to_string();
    let svc = ServeFile::new(&file.path).with_buf_chunk_size(64 * 1024);
    match svc.oneshot(req).await {
        Ok(mut resp) => {
            // ServeFile sets content-type from the on-disk filename, which uses
            // our UUID-based stored name. Override using the original filename.
            resp.headers_mut()
                .insert(header::CONTENT_TYPE, mime.parse().unwrap());
            resp.into_response()
        }
        Err(e) => match e {},
    }
}

async fn settings_get(State(s): State<AppState>) -> ApiResult<Json<Settings>> {
    Ok(Json(settings_store::load(&s.db)?))
}

async fn settings_set(
    State(s): State<AppState>,
    Json(settings): Json<Settings>,
) -> ApiResult<Json<Settings>> {
    settings_store::save(&s.db, &settings)?;
    Ok(Json(settings))
}

#[derive(Deserialize)]
struct PipelineRunBody {
    #[serde(rename = "fileIds")]
    file_ids: Vec<String>,
}
async fn pipeline_run(
    State(s): State<AppState>,
    Json(body): Json<PipelineRunBody>,
) -> ApiResult<StatusCode> {
    let settings = settings_store::load(&s.db)?;
    let pipeline = s.pipeline.clone();
    tokio::spawn(async move {
        if let Err(e) = pipeline.run(body.file_ids, settings).await {
            tracing::error!("pipeline run error: {e}");
        }
    });
    Ok(StatusCode::ACCEPTED)
}

#[derive(Deserialize)]
struct TranslitBody {
    text: String,
}
#[derive(Serialize)]
struct TranslitOut {
    result: String,
}
async fn translit_preview(Json(body): Json<TranslitBody>) -> Json<TranslitOut> {
    Json(TranslitOut {
        result: translit::transliterate(&body.text),
    })
}

async fn events_sse(
    State(s): State<AppState>,
) -> Sse<impl Stream<Item = Result<Event, Infallible>>> {
    let rx = s.events.subscribe();
    let stream = BroadcastStream::new(rx).filter_map(|msg| async move {
        match msg {
            Ok(ev) => match serde_json::to_string(&ev) {
                Ok(json) => Some(Ok(Event::default().data(json))),
                Err(_) => None,
            },
            Err(_) => None,
        }
    });
    Sse::new(stream).keep_alive(
        KeepAlive::new()
            .interval(Duration::from_secs(15))
            .text("keep-alive"),
    )
}
