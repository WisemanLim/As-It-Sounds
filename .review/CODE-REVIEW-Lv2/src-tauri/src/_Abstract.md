# `src-tauri/src/` 디렉터리 요약

> [← INDEX.md](../../INDEX.md)

## 파일 목록

| 파일 | 줄 수 | 리뷰 |
|------|-----:|------|
| main.rs | 6 | [main.rs.md](./main.rs.md) |
| lib.rs | 52 | [lib.rs.md](./lib.rs.md) |
| commands.rs | 144 | [commands.rs.md](./commands.rs.md) |

## 구조 요약

Tauri 데스크톱 셸 진입부. `main.rs`는 `lib::run()` 호출만, `lib.rs`는 Builder + AppState 초기화 + invoke_handler 등록. `commands.rs`는 프런트의 `ipc.*` 호출과 1:1 매칭되는 핸들러를 정의.

## 핵심 의존

- `tauri`, `tauri_plugin_*`
- 내부: `./core/*`

## 리스크 시그널

- `lib.rs`에서 db/asr 초기화 실패를 `expect`로 panic — UX적으로 노출되지 않고 프로세스 종료. 디스크 부재 등 흔한 케이스에 약함.
- `commands.rs::convert_file_src_helper`는 invoke_handler 미등록 dead code.
- `pipeline_run` 명령이 spawn 후 즉시 OK 반환 → 실패가 이벤트로만 전달됨(상태와 분리). 호출 측에서 await 의미가 줄어듦.
