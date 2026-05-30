# As It Sounds — 웹 버전 브라우저 시험 결과 / Web Browser Test Results

날짜(Date): 2026-05-30 · 호스트: macOS (Darwin 25.4) · 헤드리스 Chromium (gstack browse)

**상태(Status): ALL GREEN ✅ (버그 1건 발견 → 수정 → 재시험 통과)**

## 구성 (Topology)
- 백엔드: `as-it-sounds-web` (axum) — `127.0.0.1:8080`, `/api` + SSE
- 프론트엔드: Vite 빌드(`VITE_AIS_API_BASE=http://127.0.0.1:8080`) → `vite preview :4318`
- 데이터: `~/Library/Application Support/com.asitsounds.app`

## 시나리오 (Scenarios)

| ID | 시나리오 | 방법 | 결과 |
| :-- | :-- | :-- | :-- |
| W1 | 초기화 / 홈 렌더 | `GET /` 로드, 콘솔/오버레이 점검 | ✅ 콘솔 오류 0, "초기화 실패" 없음, 프로젝트 목록 `/api`에서 로드 |
| W2 | 새 프로젝트 생성 + 이동 | 사이드바 "새 프로젝트" 클릭 | ✅ **수정 후** `#/projects/<uuid>` 이동, 프로젝트 페이지 렌더 |
| W3 | 프로젝트 열기 | 목록 항목 클릭 | ✅ 해당 프로젝트로 이동, 업로드 드롭존 표시 |
| W4 | 설정 페이지 | "설정" 클릭 | ✅ `#/settings`, Whisper 모델·번역 엔진·ffmpeg 경로 `/api/settings`에서 로드 |
| W5 | 음차 미리보기 API | `POST /api/translit/preview` | ✅ `"Right around the corner"` → `"라이트 어라운드 더 코너"` |
| W6 | 프로젝트 삭제 API | `DELETE /api/projects/:id` | ✅ 204, 목록에서 제거 |

증거 스크린샷: `test/results/web_home.png`, `web_settings.png`, `web_project.png`

## 발견 → 수정 (Bug found → fixed)

**버그:** 사이드바 "새 프로젝트" 클릭 시 `#/projects/[object%20Promise]` 로 이동 → "프로젝트를 찾을 수 없습니다".

- **원인(Root cause):** `src/components/Sidebar.tsx` `handleNewProject` 가 `createProject()`(반환 타입 `Promise<Project>`)를 await 하지 않고 그대로 라우트에 사용. (`HomePage.tsx`는 `await ... p.id`로 올바름.)
- **수정(Fix):**
  ```diff
  - const handleNewProject = () => {
  -   const id = createProject();
  -   navigate(`/projects/${id}`);
  - };
  + const handleNewProject = async () => {
  +   const p = await createProject();
  +   navigate(`/projects/${p.id}`);
  + };
  ```
- **재시험(Retest):** `pnpm typecheck` 통과 → 재빌드 → 새 프로젝트 클릭 시 `#/projects/b0b619b6-...`(실제 UUID) 이동, 프로젝트 페이지 정상 렌더, 콘솔 오류 0. ✅

## 정리 (Cleanup)
- 시험 중 생성된 프로젝트 2건 `DELETE /api/projects/:id` 로 제거 (원래 1건 상태 복원).
- 서버/프리뷰 종료, `dist` 를 API 베이스 없는 기본 빌드로 재생성.

## 비고 (Notes)
- 웹 모드는 axum `/api` 백엔드 필요 (운영: nginx-proxy 뒤). 백엔드 없는 단독 `vite preview` 는 의도적으로 "초기화에 실패했습니다" 표시 — 정상.
- 동일 `createProject` Promise 패턴이 데스크톱(Tauri)에서도 동작하나, 데스크톱은 `tauriIpc` 가 동기 결과를 즉시 반환하지 않으므로 이 수정으로 양쪽 모두 안전.
