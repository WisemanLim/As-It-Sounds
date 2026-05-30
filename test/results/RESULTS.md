# As It Sounds — Desktop + Web Test Results

Date: 2026-05-30 · Host: macOS (Darwin 25.4) · Toolchain: Rust 1.95.0, Node 26, pnpm 10.33, Tauri 2

**Status: ALL GREEN ✅**

| ID | Scenario | Result | Evidence |
| :-- | :-- | :-- | :-- |
| D1.1 | Rust core `cargo test` (translit/asr/media) | ✅ PASS | 27 passed / 0 failed |
| D1.2 | `ais_test` CLI on `.samples/01_19_04.mp3` (base) | ✅ PASS | 4 segments → STT + Hangul phonetics, exit 0 |
| D2.1 | `pnpm typecheck` | ✅ PASS | tsc clean |
| D2.2 | `pnpm build` | ✅ PASS | vite build, dist emitted (80 modules) |
| D2.3 | `pnpm preview` HTTP smoke | ✅ PASS | `GET /` → 200, title "As It Sounds" |
| D2.4 | Web full-stack (axum backend + headless browser) | ✅ PASS | `web_home.png` — projects loaded from `/api`, no fatal overlay |
| D3.1 | `pnpm tauri:build:mac` (universal) | ✅ PASS | `.app` + `As It Sounds_0.1.0_universal.dmg` bundled |
| D3.2 | Launch built `.app` | ✅ PASS | process `as-it-sounds` alive after launch, no crash |

## Artifacts
- `test/results/web_home.png` — web UI (project list + upload dropzone) against live axum backend
- `src-tauri/target/universal-apple-darwin/release/bundle/dmg/As It Sounds_0.1.0_universal.dmg`

## Notes
- CLI pipeline used cached `ggml-base.bin`; whisper.cpp BLAS backend; auto-detected language en.
- Web run topology: `as-it-sounds-web` (axum) on `127.0.0.1:8080` serving `/api` + SSE; frontend built with `VITE_AIS_API_BASE=http://127.0.0.1:8080`, served via `vite preview`. `/api/healthz` 200, `/api/settings` + `/api/projects` returned valid JSON.
- First web attempt (bare `vite preview`, no backend) showed "초기화에 실패했습니다" — expected: web client requires the axum `/api` backend (prod serves it behind nginx-proxy). Re-tested with backend → green. Not a defect.
- Desktop `.app` launch verified by live process; window screenshot blocked by macOS Screen Recording permission (environment limitation, not an app fault).
