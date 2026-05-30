# As It Sounds — 데스크톱 + 웹 테스트 시나리오

> 🌐 **한국어** · [English](SCENARIOS.en.md)

Tauri 데스크톱 앱, Rust 코어(`src-tauri`), 웹 프론트엔드에 대한 테스트 계획입니다.
결과는 `test/results/` 에 저장됩니다.

| 환경 | 도구 |
| :-- | :-- |
| Rust 코어 | `cargo test --manifest-path src-tauri/Cargo.toml --release` |
| CLI 파이프라인 | `cargo run --example ais_test -- <audio>` |
| 웹 | `pnpm typecheck` / `pnpm build` / `pnpm preview` + 헤드리스 브라우저 |
| 데스크톱 | `pnpm tauri:build` + `.app` 실행 |

---

## D1 — Rust 코어 (`src-tauri`)

| ID | 시나리오 | 기대 결과 |
| :-- | :-- | :-- |
| D1.1 | `cargo test` (translit / asr / media 유닛) | 27개 테스트 통과 |
| D1.2 | `.samples/01_19_04.mp3` 에 대해 `ais_test` CLI 실행 (model=tiny/base) | 전사 → 한글 발음 + 번역 출력, exit 0 |

## D2 — 웹 프론트엔드

| ID | 시나리오 | 기대 결과 |
| :-- | :-- | :-- |
| D2.1 | `pnpm typecheck` | tsc 오류 없음, exit 0 |
| D2.2 | `pnpm build` | vite 빌드 성공, dist 생성 |
| D2.3 | `pnpm preview` HTTP 스모크 테스트 | `GET /` → 200, 타이틀 "As It Sounds" |
| D2.4 | 헤드리스 브라우저 로드 | 앱 셸 렌더링, 콘솔 치명적 오류 없음, 내비게이션 동작 |

## D3 — 데스크톱 (Tauri)

| ID | 시나리오 | 기대 결과 |
| :-- | :-- | :-- |
| D3.1 | `pnpm tauri:build` (mac) | `.app` + `.dmg` 번들 생성 |
| D3.2 | 빌드된 `.app` 실행 | 창 열림, 크래시 없음 |

---

## 통과 기준

D1–D3 시나리오 전체 통과. 실패 시: 로그 캡처, 원인 분석, 수정 또는 문서화 후 재실행.
전체 테스트 스위트가 통과된 경우에만 `test/results/RESULTS.md` 에 결과를 기록합니다.
