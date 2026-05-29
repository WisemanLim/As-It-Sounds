# src-tauri/src/core/settings.rs — 코드 리뷰 / Code Review

> 리뷰 레벨 / Review Level: 2
> [← INDEX.md](../../../INDEX.md)

**파일 요약 / File Summary**
- 역할 / Role: kv_settings 테이블의 `"settings"` 키에 JSON 직렬화/역직렬화 wrapper.
- 진입점에서의 거리 / Distance from entrypoint: 3 (commands → settings)
- 라인 수 / LOC: 21
- 주요 export / Exports: `load(&Db) -> Settings`, `save(&Db, &Settings)`
- 의존 모듈 / Depends on (internal): `super::db`, `super::error`, `super::models`
- 사용된 외부 패키지 / External packages used: `serde_json` (간접; 통해 `AppError` From)
- 부수효과 / Side effects: DB 읽기/쓰기.
- 동시성 모델 / Concurrency model: Db 내부 mutex.
- 테스트 커버리지 흔적 / Test coverage signal: 없음.

```rust path=src-tauri/src/core/settings.rs lines=1-6
use super::db::Db;
use super::error::AppResult;
use super::models::Settings;

const KEY: &str = "settings";
```
**분석 / Analysis (Level 2)**
- 단일 key 에 전체 Settings JSON 저장. 부분 업데이트 안 됨 → save 는 항상 full overwrite. / Whole-blob save.

```rust path=src-tauri/src/core/settings.rs lines=7-16
pub fn load(db: &Db) -> AppResult<Settings> {
    if let Some(raw) = db.get_setting(KEY)? {
        match serde_json::from_str::<Settings>(&raw) {
            Ok(s) => Ok(s),
            Err(_) => Ok(Settings::default()),
        }
    } else {
        Ok(Settings::default()),
    }
}
```
**분석 / Analysis (Level 2)**
- JSON 파싱 실패 시 silent default 로 fallback. 사용자가 손으로 DB 를 수정해 망가뜨려도 앱은 계속 동작. / Resilient but silent.
- (CWE-252) 파싱 에러를 무시하므로 어떤 사유로 손상됐는지 추적 어려움. `tracing::warn!` 권장.

**짚어볼 점 / Watch-outs**
- 마이그레이션 시 새 필드가 추가되면 기존 JSON 에서 missing → serde 가 에러 → silent default. 사용자가 설정한 API key/endpoint 가 모두 초기화 됨. Risky.

```rust path=src-tauri/src/core/settings.rs lines=18-21
pub fn save(db: &Db, s: &Settings) -> AppResult<()> {
    let raw = serde_json::to_string(s)?;
    db.set_setting(KEY, &raw)
}
```
**분석 / Analysis (Level 2)**
- JSON serialize 후 db.set_setting. 한 줄. / Compact.
- `serde_json::to_string` 의 에러는 `?` 로 위로 전파.

**파일 수준 평가 / File-Level Verdict**
- 강점 / Strengths:
  1. 21 행으로 최소 책임만 가짐.
  2. JSON 파싱 실패에 회복력.
- 위험 / Risks:
  1. (CWE-252) 파싱 실패가 silent default 로 → 손상 감지 안 됨, 사용자 키 초기화 위험.
  2. 새 필드 추가 시 기존 user 의 settings 가 default 로 reset.
- 개선 권고 / Recommendations:
  1. `#[serde(default)]` 필드 별로 적용 → 누락 필드만 default, 나머지 보존.
  2. 파싱 실패 시 `tracing::warn!` + backup row 보관.
- 재읽기 권고 레벨 / Suggested re-read level: 1.
