# src-tauri/src/main.rs — 코드 리뷰 / Code Review

> 리뷰 레벨 / Review Level: 2
> [← INDEX.md](../../INDEX.md)

**파일 요약 / File Summary**
- 역할 / Role: Windows 릴리즈 빌드 콘솔 숨김 어트리뷰트 + `as_it_sounds_lib::run()` 호출하는 최소 main.
- 진입점에서의 거리 / Distance from entrypoint: 0 (실제 진입점)
- 라인 수 / LOC: 6
- 주요 export / Exports: `fn main()`
- 의존 모듈 / Depends on (internal): `as_it_sounds_lib` (즉 `lib.rs`)
- 사용된 외부 패키지 / External packages used: 없음 (직접 사용)
- 부수효과 / Side effects: 프로세스 시작.
- 동시성 모델 / Concurrency model: 단일 호출, 내부 Tauri runtime 이 차후 멀티스레드 구동.
- 테스트 커버리지 흔적 / Test coverage signal: 없음.

```rust path=src-tauri/src/main.rs lines=1-6
// Prevents additional console window on Windows in release, DO NOT REMOVE!!
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

fn main() {
    as_it_sounds_lib::run()
}
```
**분석 / Analysis (Level 2)**
- 1행 주석은 Tauri 표준 보일러플레이트. release 빌드에서 추가 console 창이 뜨는 것을 막음. / Tauri boilerplate, hides extra console on Windows release.
- 2행 `#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]` 는 debug 빌드에서는 콘솔 출력 유지, release 에서는 GUI subsystem 사용.
- 4-6행: 실제 로직은 `lib.rs` 에 두고 main 은 진입만 담당. / Thin shim; logic delegated to lib.

**짚어볼 점 / Watch-outs**
- (CWE-252) `run()` 이 패닉할 수 있는데 main 에서는 잡지 않음. 정책상 OK (Tauri 권장).
- macOS/iOS/Linux 빌드에서는 `windows_subsystem` 어트리뷰트가 무시되므로 cross-platform 안전.

**파일 수준 평가 / File-Level Verdict**
- 강점 / Strengths:
  1. Tauri 표준 패턴을 정확히 따름.
  2. 6행으로 모든 의도가 명확.
  3. 모바일 cfg 가 lib.rs 쪽에 집중되어 main 은 깨끗.
- 위험 / Risks:
  1. 특별한 위험 없음. 다만 panic backtrace 가 콘솔 숨김 빌드에서는 사용자에게 안 보임.
- 개선 권고 / Recommendations:
  1. 패닉 시 OS 알림이나 파일 로그를 남기는 hook 을 `run()` 내부에서 설치.
- 재읽기 권고 레벨 / Suggested re-read level: 1 — 이 파일은 더 볼 게 없음.
