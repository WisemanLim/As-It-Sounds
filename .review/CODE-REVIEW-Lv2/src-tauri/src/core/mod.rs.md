# src-tauri/src/core/mod.rs — 코드 리뷰 / Code Review

> 리뷰 레벨 / Review Level: 2
> [← INDEX.md](../../../INDEX.md)

**파일 요약 / File Summary**
- 역할 / Role: `core` 하위 9개 모듈 선언 및 `AppError/AppResult` 재출.
- 진입점에서의 거리 / Distance from entrypoint: 1 (lib.rs → core)
- 라인 수 / LOC: 11
- 주요 export / Exports: 모듈 9개 + `pub use error::{AppError, AppResult}`
- 의존 모듈 / Depends on (internal): asr, db, error, media, models, pipeline, settings, translate, translit
- 사용된 외부 패키지 / External packages used: 없음
- 부수효과 / Side effects: 없음 (선언만).
- 동시성 모델 / Concurrency model: N/A
- 테스트 커버리지 흔적 / Test coverage signal: 없음.

```rust path=src-tauri/src/core/mod.rs lines=1-11
pub mod asr;
pub mod db;
pub mod error;
pub mod media;
pub mod models;
pub mod pipeline;
pub mod settings;
pub mod translate;
pub mod translit;

pub use error::{AppError, AppResult};
```
**분석 / Analysis (Level 2)**
- 모두 `pub` → web-server 가 `#[path = "../../src-tauri/src/core/..."]` 으로 직접 모듈 파일 재이용. 만약 `core::mod` 자체가 web-server 에서 import 되면 `pipeline` 까지 노출되지만 web-server 는 모듈을 개별 `#[path]` 로 골라 가져옴. / pub everywhere because web-server reuses these files via `#[path]`.
- `pub use` 로 `AppError/AppResult` 단축 경로 제공. 자주 쓰이는 두 타입을 `core::AppError` 로 접근 가능. / Convenience re-export.

**짚어볼 점 / Watch-outs**
- `pipeline` 은 Tauri 전용 (AppHandle 사용). web-server 가 잘못 `#[path]` 로 가져오면 컴파일 실패. 현 web-server 의 `core.rs` 는 pipeline 을 제외시켜 안전.

**파일 수준 평가 / File-Level Verdict**
- 강점 / Strengths:
  1. 모듈 트리가 한눈에 보여 디렉토리 이해 빠름.
  2. 자주 쓰는 에러 타입 재출(re-export)로 호출 깊이 감소.
- 위험 / Risks:
  1. 모두 `pub` 라 외부에서 접근 가능. 데스크톱 단독이면 `pub(crate)` 가 더 보수적이지만, web-server reuse 때문에 이대로 두는 게 맞음.
- 개선 권고 / Recommendations:
  1. 문서 주석으로 “web-server 가 `#[path]` 로 이 파일들을 직접 import 함을 명시”.
- 재읽기 권고 레벨 / Suggested re-read level: 1.
