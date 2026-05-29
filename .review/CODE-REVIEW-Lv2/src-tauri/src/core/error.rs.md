# src-tauri/src/core/error.rs — 코드 리뷰 / Code Review

> 리뷰 레벨 / Review Level: 2
> [← INDEX.md](../../../INDEX.md)

**파일 요약 / File Summary**
- 역할 / Role: 앱 공용 에러 enum `AppError` 와 `AppResult<T>` 타입 별칭, 프론트로 직렬화하기 위한 `Serialize` 구현.
- 진입점에서의 거리 / Distance from entrypoint: 2
- 라인 수 / LOC: 58
- 주요 export / Exports: `AppError`, `AppResult`
- 의존 모듈 / Depends on (internal): 없음
- 사용된 외부 패키지 / External packages used: `serde`, `thiserror`, `rusqlite`, `reqwest`, `serde_json`, `anyhow`
- 부수효과 / Side effects: 없음
- 동시성 모델 / Concurrency model: N/A (Send + Sync 자동 도출)
- 테스트 커버리지 흔적 / Test coverage signal: 없음

```rust path=src-tauri/src/core/error.rs lines=1-32
use serde::Serialize;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum AppError {
    #[error("database error: {0}")]
    Db(#[from] rusqlite::Error),

    #[error("io error: {0}")]
    Io(#[from] std::io::Error),

    #[error("http error: {0}")]
    Http(#[from] reqwest::Error),

    #[error("serialization error: {0}")]
    Json(#[from] serde_json::Error),

    #[error("whisper error: {0}")]
    Whisper(String),

    #[error("ffmpeg not found: install it or set the path in settings")]
    FfmpegNotFound,

    #[error("ffmpeg failed: {0}")]
    Ffmpeg(String),

    #[error("asr model not ready")]
    ModelNotReady,

    #[error("{0}")]
    Generic(String),
}
```
**분석 / Analysis (Level 2)**
- `thiserror` 매크로로 `Display + std::error::Error` 자동 구현. `#[from]` 덕분에 `?` 연산자로 자동 변환 가능. / `?` works for Db/Io/Http/Json via From impls.
- `Whisper(String)` 는 `whisper-rs` 에러를 문자열화해서 보관. From 구현이 없는 이유는 외부 crate 의 에러 타입이 stable 하지 않거나 다양해서. / Whisper errors stringified.

**짚어볼 점 / Watch-outs**
- (CWE-209) `error: {0}` 포맷은 내부 메시지를 그대로 노출. SQLite/HTTP/IO 의 path 정보가 그대로 사용자에게 보일 수 있어 정보 누출 가능성. 데스크톱 single-user 환경에선 OK, web-server 에서는 주의.

```rust path=src-tauri/src/core/error.rs lines=34-40
pub type AppResult<T> = Result<T, AppError>;

impl Serialize for AppError {
    fn serialize<S: serde::Serializer>(&self, s: S) -> Result<S::Ok, S::Error> {
        s.serialize_str(&self.to_string())
    }
}
```
**분석 / Analysis (Level 2)**
- 에러를 항상 단일 문자열로 직렬화. JSON 출력 시 `"database error: no such table"` 같은 평문 string. Tauri command 가 Err 반환 시 그대로 프론트에 전달. / Stringly-typed JSON error.

**짚어볼 점 / Watch-outs**
- 프론트 입장에서 에러 종류를 구별하려면 텍스트 prefix(`database error:`)에 의존해야 함 → 다국어/i18n 어려움. tag+message 구조가 더 견고.

```rust path=src-tauri/src/core/error.rs lines=42-58
impl From<anyhow::Error> for AppError {
    fn from(e: anyhow::Error) -> Self {
        AppError::Generic(e.to_string())
    }
}

impl From<String> for AppError {
    fn from(s: String) -> Self {
        AppError::Generic(s)
    }
}

impl From<&str> for AppError {
    fn from(s: &str) -> Self {
        AppError::Generic(s.to_string())
    }
}
```
**분석 / Analysis (Level 2)**
- `From<&str>`/`From<String>` 은 편리하지만 위험. 일반 문자열 메시지가 `?` 로 silently 에러로 승격되며, 호출자가 의도치 않게 String 이 에러로 바뀔 수 있음. / `From<String>` makes any String coercible via `?`.
- `From<anyhow::Error>` 는 anyhow 사용 코드와 호환 위해 필요.

**짚어볼 점 / Watch-outs**
- `From<&str>` 은 거의 안 쓰일 가능성. 사용처 검색해보면 좋음.

**파일 수준 평가 / File-Level Verdict**
- 강점 / Strengths:
  1. `thiserror` + `#[from]` 으로 보일러플레이트 최소화.
  2. 직렬화가 추가되어 Tauri/Axum 양쪽에서 같은 타입 사용 가능.
  3. enum 분류가 도메인을 잘 반영 (Db, Http, Whisper, Ffmpeg…).
- 위험 / Risks:
  1. (CWE-209) 에러 메시지 그대로 노출 — web-server 외부 노출 시 정보 누출.
  2. `From<String>`/`From<&str>` 가 ergonomic 하지만 의도치 않은 변환 유발 가능.
  3. 에러 변형 정보가 없어 i18n/구조화된 에러 처리에 한계.
- 개선 권고 / Recommendations:
  1. `Serialize` 를 `{ "kind": "Db", "message": "..." }` 형태로 확장.
  2. `From<String>` 제거하고 `AppError::Generic(...)` 명시 호출 권장.
  3. 외부 노출용 메시지와 내부 로깅용을 분리.
- 재읽기 권고 레벨 / Suggested re-read level: 2.
