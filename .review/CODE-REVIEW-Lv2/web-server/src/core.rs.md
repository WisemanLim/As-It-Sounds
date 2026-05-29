# web-server/src/core.rs — 코드 리뷰 / Code Review

> 리뷰 레벨 / Review Level: 2
> [← INDEX.md](../../INDEX.md)

**파일 요약 / File Summary**
- 역할 / Role: `#[path]` 어트리뷰트로 `src-tauri/src/core/*.rs` 파일들을 web-server 크레이트 안에서 재사용. pipeline 만 제외.
- 진입점에서의 거리 / Distance from entrypoint: 1 (main → core)
- 라인 수 / LOC: 30
- 주요 export / Exports: `error`, `models`, `translit`, `translate`, `db`, `asr`, `media`, `settings` 모듈 + `AppError/AppResult` re-export.
- 의존 모듈 / Depends on (internal): src-tauri/src/core 의 8 개 파일.
- 사용된 외부 패키지 / External packages used: 없음 (재경로만)
- 부수효과 / Side effects: 없음.
- 동시성 모델 / Concurrency model: N/A.
- 테스트 커버리지 흔적 / Test coverage signal: 없음.

```rust path=web-server/src/core.rs lines=1-12
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
```
**분석 / Analysis (Level 2)**
- `#[path]` 트릭으로 두 크레이트 간 코드 단일 소스. workspace 가 아니라도 동작. / Same file path-included in two crates.
- docstring 이 “pipeline 은 분리됨” 을 명시 → 다음 reader 가 의도 즉시 이해.

**짚어볼 점 / Watch-outs**
- 같은 .rs 파일이 두 곳에서 컴파일됨 → 양쪽 모두 빌드 비용. Workspace + features 로 분리하는 게 더 깔끔하지만 현 구조도 작동.
- `pipeline.rs` 만 다른 이유: `tauri::AppHandle` 의존 → 빌드 못함. 정당한 분리.

```rust path=web-server/src/core.rs lines=15-30
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
```
**분석 / Analysis (Level 2)**
- 8개의 `#[path]` 매핑. 일관성 좋고 변경 영향이 명확.
- desktop core 모듈이 `super::*` 로 형제 모듈을 참조 → web-server core 에서도 동일한 형제 관계로 보여야 함. 모든 모듈이 같은 `web-server/src/core` 부모 아래 있으므로 OK.

**짚어볼 점 / Watch-outs**
- 만약 src-tauri/src/core 안에 새 모듈을 추가하고 web-server 가 그것을 쓰려면 여기에 `#[path]` 라인 추가 필요. 빠뜨리면 compile error 발생.

**파일 수준 평가 / File-Level Verdict**
- 강점 / Strengths:
  1. 한 파일에 모든 reuse mapping 집중 → 디스커버리 용이.
  2. docstring 으로 의도(왜 pipeline 분리) 명시.
  3. 빌드 시 desktop 변경이 web-server 에도 자동 반영.
- 위험 / Risks:
  1. `#[path]` 는 IDE/cargo doc 도구의 cross-crate 추적이 불완전할 수 있음.
  2. 두 빌드 사이 feature gate 차이 시 헷갈림 (현재는 동일 빌드).
- 개선 권고 / Recommendations:
  1. cargo workspace + 별도 `as-it-sounds-core` 크레이트로 옮기면 표준 의존성으로 명확.
- 재읽기 권고 레벨 / Suggested re-read level: 1.
