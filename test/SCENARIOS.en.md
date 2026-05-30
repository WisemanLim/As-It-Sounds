# As It Sounds — Desktop + Web Test Scenarios

> 🌐 [한국어](SCENARIOS.md) · **English**

Test plan for the Tauri desktop app, the Rust core (`src-tauri`), and the web frontend.
Results saved under `test/results/`.

| Env | Tool |
| :-- | :-- |
| Rust core | `cargo test --manifest-path src-tauri/Cargo.toml --release` |
| CLI pipeline | `cargo run --example ais_test -- <audio>` |
| Web | `pnpm typecheck` / `pnpm build` / `pnpm preview` + headless browser |
| Desktop | `pnpm tauri:build` + launch `.app` |

---

## D1 — Rust core (`src-tauri`)

| ID | Scenario | Expected |
| :-- | :-- | :-- |
| D1.1 | `cargo test` (translit / asr / media units) | 27 tests pass |
| D1.2 | `ais_test` CLI on `.samples/01_19_04.mp3` (model=tiny/base) | Transcription → Hangul phonetics + translation printed, exit 0 |

## D2 — Web frontend

| ID | Scenario | Expected |
| :-- | :-- | :-- |
| D2.1 | `pnpm typecheck` | tsc clean, exit 0 |
| D2.2 | `pnpm build` | vite build succeeds, dist emitted |
| D2.3 | `pnpm preview` HTTP smoke | `GET /` → 200, title "As It Sounds" |
| D2.4 | Headless browser load | App shell renders, no console fatal, nav works |

## D3 — Desktop (Tauri)

| ID | Scenario | Expected |
| :-- | :-- | :-- |
| D3.1 | `pnpm tauri:build` (mac) | `.app` + `.dmg` bundled |
| D3.2 | Launch built `.app` | Window opens, no crash |

---

## Pass criteria

All D1–D3 scenarios pass. On failure: capture log, root-cause, fix or document, re-run.
Results written to `test/results/RESULTS.md` only when the full suite is green.
