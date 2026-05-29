# src-tauri/src/ — 디렉토리 개요 / Directory Abstract

> [← INDEX.md](../../INDEX.md)

## 파일 목록 / File List
| 파일 / File | LOC | 역할 / Role | 리뷰 링크 / Review |
|---|---|---|---|
| main.rs | 6 | 프로세스 진입점 / Process entry | [main.rs.md](main.rs.md) |
| lib.rs | 52 | Tauri 앱 부팅 + invoke_handler / Tauri boot | [lib.rs.md](lib.rs.md) |
| commands.rs | 144 | `#[tauri::command]` 핸들러 / IPC commands | [commands.rs.md](commands.rs.md) |
| core/ | 2925 | 도메인 로직 (별도 abstract) / Domain core | [core/core_Abstract.md](core/core_Abstract.md) |

## 디렉토리 구조 / Structure Summary
세 개의 “shell” 파일 (main/lib/commands) + 하나의 `core/` 서브트리. `main.rs` 는 1-line stub, `lib.rs` 는 builder/setup, `commands.rs` 는 thin IPC 어댑터. 비즈니스 로직은 모두 `core/` 안에 있어 web-server 가 그대로 재사용 가능한 모듈 경계.

## 주요 의존 관계 / Key Dependencies
- `lib.rs` 가 `commands::AppState` 를 `app.manage(...)` 로 저장.
- 모든 command 가 `State<'_, AppState>` 로 db/pipeline 접근.
- `pipeline_run` command 만 `tauri::async_runtime::spawn` 으로 background 실행.

## 위험 신호 / Risk Signals
- 명령 등록 누락 가능성 (lib.rs `generate_handler!` 와 commands.rs 함수 사이의 sync 수동).
- `commands.rs` 의 `convert_file_src_helper` 가 dead code.
