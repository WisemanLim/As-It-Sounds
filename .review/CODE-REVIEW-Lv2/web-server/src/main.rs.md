# web-server/src/main.rs — 코드 리뷰 / Code Review

> 리뷰 레벨 / Review Level: 2
> [← INDEX.md](../../INDEX.md)

**파일 요약 / File Summary**
- 역할 / Role: Axum HTTP/SSE 서버. Tauri command 와 동일 형태를 HTTP 라우트로 노출. SSE 로 PipelineEvent 브로드캐스트.
- 진입점에서의 거리 / Distance from entrypoint: 0 (이 파일이 `main`)
- 라인 수 / LOC: 376
- 주요 export / Exports: `fn main`, `AppState`, `ApiError`, 라우트 핸들러 다수.
- 의존 모듈 / Depends on (internal): `core::*`, `pipeline::Pipeline`
- 사용된 외부 패키지 / External packages used: `axum`, `tower`, `tower_http`, `tokio`, `tokio_stream`, `futures_util`, `mime_guess`, `serde`, `serde_json`, `tracing`, `tracing_subscriber`, `uuid`
- 부수효과 / Side effects: TCP listen, 디스크 write (uploads), broadcast 채널, DB, ASR.
- 동시성 모델 / Concurrency model: tokio 멀티 스레드. broadcast 채널 1024 capacity. handler 마다 spawn.
- 테스트 커버리지 흔적 / Test coverage signal: 없음.

```rust path=web-server/src/main.rs lines=1-32
//! As It Sounds — HTTP/SSE server.
...
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
...
use uuid::Uuid;
```
**분석 / Analysis (Level 2)**
- `mod core` 가 핵심 reuse 포인트 (core.rs 가 src-tauri 모듈을 `#[path]` 로 import).
- Axum 0.7+ 의 새 라우팅 시그니처 (State, AxPath, Multipart) 사용.

```rust path=web-server/src/main.rs lines=44-71
#[derive(Clone)]
struct AppState {
    db: Db,
    pipeline: Arc<Pipeline>,
    events: broadcast::Sender<PipelineEvent>,
    uploads_dir: Arc<PathBuf>,
}

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
```
**분석 / Analysis (Level 2)**
- (CWE-209) 모든 에러를 500 + 그대로 메시지 노출. SQL/IO 에러 메시지가 외부에 흘러나갈 수 있음. / Information leak via error text.
- HTTP 상태코드가 일률적 500 → 클라이언트가 4xx (예: 404 file not found, 400 bad request) 구분 어려움.

**짚어볼 점 / Watch-outs**
- `IntoResponse` 안에서 `AppError` 변형별로 적절한 코드 매핑 권장 (Db→500, NotFound→404, Multipart→400…).

```rust path=web-server/src/main.rs lines=75-104
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
```
**분석 / Analysis (Level 2)**
- 5+ 곳 `expect` — 부팅 실패는 즉시 패닉. 컨테이너 환경에선 OK (재시작/로그). / Boot-time fail-fast is acceptable here.
- `_rx` 는 broadcast 채널 유지용. 첫 receiver 가 drop 되면 다음 send 가 채널을 닫는다 — `tx.clone()` 으로 sender 보존이므로 OK. `_rx` 가 없으면 처음 send 가 SendError. / `_rx` keeps channel alive.
- broadcast capacity 1024. 빠른 producer + 느린 consumer 시 lag → 오래된 이벤트 손실 (broadcast 의 정상 동작).

```rust path=web-server/src/main.rs lines=106-148
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
```
**분석 / Analysis (Level 2)**
- 기본 2GB 업로드 한도. 환경변수 조정 가능. / 2 GiB default.
- **CORS: `Any` * 3** → 완전 개방. 모든 origin/method/header 허용. (CWE-942) 만약 인증이 추가될 경우 CSRF/CORS 정책 재검토 필수. **현재는 인증 미존재 + 사설망 가정**. / Wide-open CORS, OK if private-network.
- (CWE-306) **인증/인가 없음**. 누구나 프로젝트 생성/삭제/파일 업로드/파이프라인 실행 가능. README 에 “localhost/사설망 전용” 명시 필요. / No authn/authz.
- bind default `0.0.0.0:8080` → 모든 인터페이스 노출. 컨테이너 외부에서도 접근. 인증이 없으므로 공개 인터넷 노출 시 즉시 RCE-수준 위험은 아니지만 데이터 노출. 보안 prod 운용시 reverse proxy + auth 필수.

**짚어볼 점 / Watch-outs**
- 인증 부재 + CORS `Any` 조합은 사설 네트워크/dev 환경 전제. README/Deployment 문서에 강조.
- `TraceLayer::new_for_http()` 가 path/method 만 로깅. body 는 안 됨 → 비밀번호/API key 누출 없음.

```rust path=web-server/src/main.rs lines=152-198
async fn healthz() -> &'static str { "ok" }

async fn projects_list(State(s): State<AppState>) -> ApiResult<Json<Vec<Project>>> {
    Ok(Json(s.db.list_projects()?))
}

#[derive(Deserialize)]
struct CreateProjectBody { name: Option<String> }
async fn projects_create(
    State(s): State<AppState>,
    Json(body): Json<CreateProjectBody>,
) -> ApiResult<Json<Project>> {
    Ok(Json(s.db.create_project(body.name)?))
}

#[derive(Deserialize)]
struct RenameProjectBody { name: String }
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

async fn files_list(...) -> ApiResult<Json<Vec<MediaFile>>> {
    Ok(Json(s.db.list_files(&id)?))
}
```
**분석 / Analysis (Level 2)**
- 각 핸들러는 thin wrapper. DB 호출 직접 → 동기 함수가 tokio runtime 위에 깔린다. SQLite 가 mutex 직렬화하므로 blocking 가능. axum 의 멀티 워커가 다른 요청 처리 가능. / Sync DB call in async handler.

**짚어볼 점 / Watch-outs**
- 무거운 list 쿼리에서 worker thread 점유. `spawn_blocking` 권장.

```rust path=web-server/src/main.rs lines=200-254
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
    ...
}
```
**분석 / Analysis (Level 2)**
- (CWE-22 path traversal 분석) 저장 파일명은 **UUID + 사용자 확장자**. UUID 부분이 안전하지만 확장자에 `../` 또는 NUL byte 가 들어가도 일단 `Path::new(&original_name).extension()` 이 마지막 `.` 이후만 추출 → 보안 OK. 단, 확장자가 매우 긴 임의 문자열 가능. / UUID basename guards path traversal; extension is user-controlled but contained.
- (CWE-400) 업로드 도중 에러시 부분 파일이 디스크에 남음 (cleanup 없음). / Partial file leaks on error.
- `DefaultBodyLimit` 으로 전체 요청 크기 제한.
- `original_name` 은 DB 의 `name` 컬럼에 저장 → XSS 위험 검토는 프론트엔드 책임 (Rust 백엔드는 그대로 저장만).

**짚어볼 점 / Watch-outs**
- 확장자 화이트리스트 (mp3/wav/mp4/...) 적용 권장.
- 업로드 도중 클라이언트 disconnect → tmp 파일 잔존. cleanup task 또는 try/finally.

```rust path=web-server/src/main.rs lines=256-282
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

async fn files_reset(...) -> ApiResult<StatusCode> { ... }

async fn segments_list(...) -> ApiResult<Json<Vec<Segment>>> {
    Ok(Json(s.db.list_segments(&id)?))
}
```
**분석 / Analysis (Level 2)**
- (CWE-22 here) `f.path` 가 DB 에 저장된 절대경로 — 업로드 시 `uploads_dir/UUID.ext` 로 저장됐으므로 외부 경로 가능성 없음. 안전. / Path is constrained by upload pipeline.
- 디스크 remove 가 silently 실패해도 DB row 는 지워짐 → 디스크 파일이 orphan. 약간 leak.

```rust path=web-server/src/main.rs lines=284-311
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
            resp.headers_mut()
                .insert(header::CONTENT_TYPE, mime.parse().unwrap());
            resp.into_response()
        }
        Err(e) => match e {},
    }
}
```
**분석 / Analysis (Level 2)**
- `ServeFile` 가 Range/If-Modified-Since 자동 처리 → 비디오 seek 지원.
- (CWE-22 again) `file.path` 가 사용자 제어 가능? upload 단계에서만 row 가 생성되며 path 는 서버가 결정한 uploads_dir UUID. 단, DB 가 외부에서 수정될 수 있다면 위험.
- `mime.parse().unwrap()` 는 mime_guess 가 valid mime 만 반환하므로 사실상 panic 없음. 그래도 `.unwrap()` 은 review 시 red flag.
- `match e {}` 는 `Infallible` 매칭 — type level 로 unreachable.

**짚어볼 점 / Watch-outs**
- `mime.parse()` 가 hypothetically 실패하면 panic → 5xx 처리로 바꾸기.
- DB path 가 uploads_dir 밖이면 임의 파일 노출 — 현 흐름에선 발생 안 함이지만 defense-in-depth 로 canonicalize 검사 권장.

```rust path=web-server/src/main.rs lines=313-356
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
struct TranslitBody { text: String }
#[derive(Serialize)]
struct TranslitOut { result: String }
async fn translit_preview(Json(body): Json<TranslitBody>) -> Json<TranslitOut> {
    Json(TranslitOut { result: translit::transliterate(&body.text) })
}
```
**분석 / Analysis (Level 2)**
- pipeline_run 의 `tokio::spawn` 결과 에러는 `tracing::error!` 로 로그 → desktop 보다 좀 더 좋은 처리.
- 202 Accepted 응답으로 비동기 시작 표현. / Proper 202 semantics.
- `translit_preview` 가 매우 큰 text 입력 받으면 sync CPU 작업이 axum worker thread 점유. spawn_blocking 권장.

```rust path=web-server/src/main.rs lines=358-376
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
```
**분석 / Analysis (Level 2)**
- SSE keep-alive 15초. 프록시/방화벽 idle 종료 회피. / Standard SSE keepalive.
- broadcast lag 발생 시 `Err(Lagged)` → `None` 으로 drop. 클라이언트가 일부 이벤트 누락 가능 (broadcast 의 정상 동작). 채널 capacity 1024 가 충분한지는 트래픽 의존.
- serde_json fail 시 silent drop → 정상 PipelineEvent 는 항상 serialize OK.

**짚어볼 점 / Watch-outs**
- 다중 동시 client 가 subscribe 시 broadcast 가 fan-out. 1024 buffer per subscriber. 메모리 가벼움.
- 인증 없음 → SSE 도 누구나 listen.

**파일 수준 평가 / File-Level Verdict**
- 강점 / Strengths:
  1. Tauri 와 동일한 모델/DB/translit 재사용 (`#[path]` 트릭).
  2. Multipart 업로드 + UUID 파일명 + ServeFile 의 Range 지원으로 비디오 seek 완벽.
  3. SSE keep-alive 와 broadcast 채널 capacity 적절.
- 위험 / Risks:
  1. (CWE-306) **인증/인가 부재** — 공개 인터넷 노출 금지.
  2. (CWE-942) CORS `Any` 전면 허용.
  3. (CWE-209) 모든 에러 → 500 + 원본 메시지 노출.
- 개선 권고 / Recommendations:
  1. API key middleware (Authorization header) 또는 OAuth.
  2. CORS 를 `allow_origin(["http://localhost:5173".parse().unwrap()])` 형태로 좁히기.
  3. `IntoResponse` 에 에러 분류 (404/400/500) + 외부용 메시지 분리.
  4. 업로드 확장자/MIME 화이트리스트.
  5. `spawn_blocking` 으로 동기 DB 호출 분리.
- 재읽기 권고 레벨 / Suggested re-read level: 3 — production 배포 전 보안 검토 필수.
