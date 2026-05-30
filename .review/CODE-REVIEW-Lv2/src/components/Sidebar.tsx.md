# src/components/Sidebar.tsx — 코드 리뷰 / Code Review

> 리뷰 레벨 / Review Level: 2
> [← INDEX.md](../../INDEX.md)

**파일 요약 / File Summary**
- 역할 / Role: 좌측 사이드바 — 프로젝트 목록/검색/생성/삭제, 설정 진입 버튼.
- 진입점에서의 거리 / Distance from entrypoint: 3 (App → Layout → here)
- 라인 수 / LOC: 181
- 주요 export / Exports: `Sidebar`
- 의존 모듈 / Depends on (internal): `../store/useAppStore`, `../lib/env`, `./icons`, `../lib/cn`
- 사용된 외부 패키지 / External packages used: `react`, `react-router-dom`, dynamic `@tauri-apps/plugin-dialog`
- 부수효과 / Side effects: 라우트 변경(`navigate`), 프로젝트 store mutation, native dialog import
- 동시성 모델 / Concurrency model: async 핸들러 + Promise 기반 confirm dialog
- 테스트 커버리지 흔적 / Test coverage signal: 없음

## Step 5 — 라인별 분석 / Line-by-line walk

```tsx path=src/components/Sidebar.tsx lines=1-12
import { useMemo, useState } from "react";
import { useNavigate, useLocation } from "react-router-dom";
import { useAppStore } from "../store/useAppStore";
import { isTauri } from "../lib/env";

async function confirmDialog(message: string, title: string): Promise<boolean> {
  if (isTauri) {
    const { confirm } = await import("@tauri-apps/plugin-dialog");
    return confirm(message, { title, kind: "warning" });
  }
  return window.confirm(message);
}
```

**분석 / Analysis (Level 2)**
- `confirmDialog` 가 imports 사이에 정의되어 있어 파일 구조가 약간 어색하지만 동작에는 문제 없음.
- 동적 import — Tauri 모드일 때만 `@tauri-apps/plugin-dialog`을 로드해 웹 번들에 포함되지 않게 한다.
- 웹 빌드에서는 `window.confirm` fallback. 둘 다 boolean 통일.

**짚어볼 점 / Watch-outs**
- `import` 정의가 일반 import 뒤에 또 등장(라인 13~21) → 파일 상단의 임포트 정리 컨벤션 위반.

```tsx path=src/components/Sidebar.tsx lines=13-21
import {
  IconMenu,
  IconPlus,
  IconSearch,
  IconSettings,
  IconSparkles,
  IconTrash,
} from "./icons";
import { cn } from "../lib/cn";
```

**분석 / Analysis (Level 2)**
- import가 비-import 코드 뒤에 다시 시작 — 린터(eslint-plugin-import) 가 잡을 수 있는 패턴. 작동에는 문제 없음.

```tsx path=src/components/Sidebar.tsx lines=23-44
export function Sidebar() {
  const navigate = useNavigate();
  const location = useLocation();
  const [collapsed, setCollapsed] = useState(false);
  const [query, setQuery] = useState("");

  const projects = useAppStore((s) => s.projects);
  const activeProjectId = useAppStore((s) => s.activeProjectId);
  const createProject = useAppStore((s) => s.createProject);
  const deleteProject = useAppStore((s) => s.deleteProject);
  const setActiveProject = useAppStore((s) => s.setActiveProject);

  const filtered = useMemo(() => {
    const q = query.trim().toLowerCase();
    if (!q) return projects;
    return projects.filter((p) => p.name.toLowerCase().includes(q));
  }, [projects, query]);

  const handleNewProject = async () => {
    const p = await createProject();
    navigate(`/projects/${p.id}`);
  };
```

**분석 / Analysis (Level 2)**
- 검색 필터는 case-insensitive substring — 충분히 단순.
- `useMemo` 로 검색이 매 입력마다 재계산되지 않도록 → projects 또는 query 가 변할 때만 갱신.
- **(수정됨 / FIXED, 2026-05-30)**: 이전 버전은 `const id = createProject();` 로 `Promise<Project>` 를 그대로 보간해 `/projects/[object Promise]` → "프로젝트를 찾을 수 없습니다" 로 깨졌다. 이제 `async` + `await createProject()` 후 `p.id` 를 사용 → 실제 UUID 라우트로 이동. `HomePage.tsx` 의 패턴(`await createProject(); p.id`)과 일치.
- `createProject` 는 zustand store 에서 `Promise<Project>` 를 반환(웹은 `POST /api/projects`, 데스크톱은 tauri IPC) 하므로 await 가 반드시 필요.

**짚어볼 점 / Watch-outs**
- (해결됨 / Resolved) 브라우저 시험으로 회귀 확인: 클릭 시 `#/projects/<uuid>` 이동, 콘솔 오류 0. → `test/results/WEB-RESULTS.md` (W2).

```tsx path=src/components/Sidebar.tsx lines=46-49
  const handleOpenProject = (id: string) => {
    setActiveProject(id);
    navigate(`/projects/${id}`);
  };
```

**분석 / Analysis (Level 2)**
- `setActiveProject` 는 async 이지만 await 없이 실행 → fire-and-forget. 라우트 전환 + 파일 로딩이 병렬로 진행되어 빠르게 느껴지지만, 실패 시 silent.

**짚어볼 점 / Watch-outs**
- `setActiveProject` 의 거부(rejection)가 unhandledrejection 으로 빠져 main.tsx 의 fatal 오버레이를 트리거할 수 있다.

```tsx path=src/components/Sidebar.tsx lines=51-99
  return (
    <aside ...>
      <div className="flex items-center gap-2 px-3 py-3">
        <button ... onClick={() => setCollapsed((c) => !c)} ...>
          <IconMenu />
        </button>
        ...
      </div>

      <div className="px-3">
        <button onClick={handleNewProject} ...>
          <IconPlus />
          {!collapsed && <span>새 프로젝트</span>}
        </button>
      </div>

      {!collapsed && (
        <div className="mt-3 px-3">
          <div className="flex items-center gap-2 ...">
            <IconSearch className="text-fg-subtle" width={16} height={16} />
            <input value={query} onChange={(e) => setQuery(e.target.value)} ... />
          </div>
        </div>
      )}
```

**분석 / Analysis (Level 2)**
- collapsed 토글이 단순 width 트랜지션(`transition-[width]`). 로직이 클래스에 위임되어 매우 가독성 좋다.
- 검색 입력이 collapsed 일 때는 사라진다 — UX 자연스럽다.

```tsx path=src/components/Sidebar.tsx lines=101-165
      <nav className="mt-3 flex-1 overflow-y-auto px-2">
        ...
        <ul className="space-y-0.5">
          {filtered.length === 0 && !collapsed && (
            <li className="px-2 py-3 text-xs text-fg-subtle">
              아직 프로젝트가 없습니다.
            </li>
          )}
          {filtered.map((p) => {
            const isActive =
              activeProjectId === p.id || location.pathname === `/projects/${p.id}`;
            return (
              <li key={p.id}>
                <div ...>
                  <button onClick={() => handleOpenProject(p.id)} ...>
                    <span className="inline-block h-1.5 w-1.5 shrink-0 rounded-full bg-brand/60" />
                    {!collapsed && <span className="truncate">{p.name}</span>}
                  </button>
                  {!collapsed && (
                    <button
                      ...
                      onClick={async (e) => {
                        e.stopPropagation();
                        try {
                          const ok = await confirmDialog(
                            `"${p.name}" 프로젝트를 삭제할까요? 이 작업은 되돌릴 수 없습니다.`,
                            "프로젝트 삭제",
                          );
                          if (ok) {
                            await deleteProject(p.id);
                            if (isActive) navigate("/");
                          }
                        } catch (err) {
                          console.error("delete failed", err);
                        }
                      }}
                      ...
                    >
                      <IconTrash width={14} height={14} />
                    </button>
                  )}
                </div>
              </li>
            );
          })}
        </ul>
      </nav>
```

**분석 / Analysis (Level 2)**
- `isActive` 가 store + URL 양쪽을 본다 → 둘 중 하나만 맞아도 강조. 새로고침 직후처럼 store/URL이 어긋난 잠깐을 메꿔준다.
- 삭제는 confirm + try/catch → 가장 신중한 흐름. 파일 삭제(FileList)와 일관성을 비교하면 여기가 모범.
- `e.stopPropagation()` 으로 div 부모 클릭과의 이벤트 충돌 방지.
- 삭제 후 현재 페이지가 그 프로젝트면 홈("/")으로 이동 — orphan URL 방지.

**짚어볼 점 / Watch-outs**
- `name` 문자열 그대로 confirm 메시지에 들어간다 — react가 텍스트 노드로 렌더하지 않고 native dialog로 가지만, name 자체에 줄바꿈/긴 문자열이 있으면 다이얼로그가 어색해질 수 있다.

```tsx path=src/components/Sidebar.tsx lines=167-180
      <div className="border-t border-line p-2">
        <button
          className={cn(
            "btn-ghost w-full justify-start",
            collapsed && "justify-center",
          )}
          onClick={() => navigate("/settings")}
        >
          <IconSettings />
          {!collapsed && <span>설정</span>}
        </button>
      </div>
    </aside>
  );
}
```

**분석 / Analysis (Level 2)**
- 설정 버튼은 사이드바 하단 고정. 표준 데스크탑 UI 패턴.

**파일 수준 평가 / File-Level Verdict**
- 강점 / Strengths:
  - 데스크톱 native dialog 와 웹 confirm 를 단일 함수로 통합.
  - 삭제 흐름이 확인 → 액션 → 라우트 정리까지 깔끔.
  - collapsed 토글이 클래스 기반으로 단순.
- 위험 / Risks:
  - ~~**명백한 버그**: `handleNewProject` 가 Promise 객체를 그대로 URL 에 넣는다 → 깨진 라우트.~~ **(2026-05-30 수정 완료 / FIXED)**
  - import 가 코드 사이에 끼어 있어 린트 규칙 위반.
  - `setActiveProject` 등 async 호출 fire-and-forget — 실패가 fatal 오버레이로 빠질 가능성.
- 개선 권고 / Recommendations:
  - ~~`handleNewProject` 를 `async` + `await createProject()` 로 수정.~~ **완료 / Done** (commit `02429c0`).
  - `confirmDialog` 함수를 `../lib/dialog.ts` 등 별도 모듈로 분리 (DRY + 린트 친화).
  - async 호출에 `.catch(console.error)` 추가 (잔존: `handleOpenProject` 의 `setActiveProject`).
- 재읽기 권고 레벨 / Suggested re-read level: 2 — 신규 프로젝트 버그는 해결됨. 남은 항목은 import 정리 + fire-and-forget catch.
