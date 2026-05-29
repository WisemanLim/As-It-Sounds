# src/App.tsx — 코드 리뷰 / Code Review

> 리뷰 레벨 / Review Level: 2
> [← INDEX.md](../INDEX.md)

**파일 요약 / File Summary**
- 역할 / Role: 최상위 React 컴포넌트. 부트스트랩 호출, 로딩/에러 화면, 라우트 정의를 담당.
- 진입점에서의 거리 / Distance from entrypoint: 1 (main.tsx 에서 직접 임포트)
- 라인 수 / LOC: 62
- 주요 export / Exports: `default App`
- 의존 모듈 / Depends on (internal): `./components/Layout`, `./pages/HomePage`, `./pages/ProjectPage`, `./pages/SettingsPage`, `./store/useAppStore`, `./components/PipelineListener`, `./components/ErrorBoundary`
- 사용된 외부 패키지 / External packages used: `react`, `react-router-dom`
- 부수효과 / Side effects: `useEffect` 안에서 `bootstrap()` 호출 (전역 zustand store 변경)
- 동시성 모델 / Concurrency model: 단일 비동기 부트스트랩 (Promise + setState)
- 테스트 커버리지 흔적 / Test coverage signal: 없음 (no `*.test.tsx`)

## Step 5 — 라인별 분석 / Line-by-line walk

```tsx path=src/App.tsx lines=1-9
import { useEffect, useState } from "react";
import { Route, Routes } from "react-router-dom";
import { Layout } from "./components/Layout";
import { HomePage } from "./pages/HomePage";
import { ProjectPage } from "./pages/ProjectPage";
import { SettingsPage } from "./pages/SettingsPage";
import { useAppStore } from "./store/useAppStore";
import { PipelineListener } from "./components/PipelineListener";
import { ErrorBoundary } from "./components/ErrorBoundary";
```

**분석 / Analysis (Level 2)**
- React 훅(`useEffect`, `useState`)과 react-router 라우팅을 함께 사용한다. 라우팅을 위해 `Routes`/`Route`만 가져오고 `BrowserRouter`/`HashRouter`는 `main.tsx`에서 감싸는 분리된 구조.
- 페이지·레이아웃·상태·리스너·에러바운더리를 모두 한 화면(`App`)에서 조립하는 전형적인 SPA 셸 패턴.
- 임포트가 상대경로뿐이라 alias(@/)가 없어 디렉토리 이동 시 손이 많이 가지만, 초보자가 따라가기는 쉽다.

**짚어볼 점 / Watch-outs**
- alias 없음 → 추후 폴더 재구성 시 일괄 변경 부담.

```tsx path=src/App.tsx lines=11-21
function App() {
  const bootstrap = useAppStore((s) => s.bootstrap);
  const loaded = useAppStore((s) => s.loaded);
  const [bootError, setBootError] = useState<string | null>(null);

  useEffect(() => {
    bootstrap().catch((e) => {
      console.error("bootstrap failed", e);
      setBootError(e instanceof Error ? e.message : String(e));
    });
  }, [bootstrap]);
```

**분석 / Analysis (Level 2)**
- `useAppStore((s) => s.xxx)` 형태로 셀렉터를 사용해 필요한 슬라이스만 구독한다. zustand의 함수 참조는 안정적이므로 `useEffect` 의존성에 넣어도 무한 루프가 나지 않는다.
- `bootstrap` 결과는 await 하지 않고 `catch` 만 다는 fire-and-forget 패턴. 에러는 로컬 state(`bootError`)와 console에 기록한다.
- `e instanceof Error ? e.message : String(e)` — Error 객체가 아닌 거부(rejection)에도 안전한 방어 코드. (TypeScript의 `catch (e)` 는 기본 `unknown` 타입)

**짚어볼 점 / Watch-outs**
- StrictMode(`main.tsx`에서 활성)에서는 dev 모드에서 `useEffect`가 두 번 실행될 수 있다. `bootstrap`이 멱등(idempotent) 해야 안전. 현재는 `ipc.projectList`를 두 번 호출하지만 부수효과는 없으므로 OK.
- `bootError`가 한 번 세팅되면 reset 경로가 없다 → "다시 시도" 버튼 부재. ErrorBoundary 와의 일관성 면에서 아쉬움.

```tsx path=src/App.tsx lines=23-37
  if (bootError) {
    return (
      <div className="flex h-screen w-screen flex-col items-center justify-center gap-3 bg-bg p-8 text-center">
        <div className="text-base font-semibold text-red-400">
          초기화에 실패했습니다
        </div>
        <pre className="max-w-2xl whitespace-pre-wrap break-words rounded-lg bg-bg-elevated p-4 text-left font-mono text-xs text-fg-muted">
          {bootError}
        </pre>
        <div className="text-xs text-fg-subtle">
          개발자 도구 콘솔(⌥⌘I)에서 자세한 내용을 확인하세요.
        </div>
      </div>
    );
  }
```

**분석 / Analysis (Level 2)**
- 부트스트랩 실패 시 Tailwind 클래스 기반의 풀스크린 에러 표시. `whitespace-pre-wrap` 으로 긴 메시지를 줄바꿈한다.
- 메시지에 macOS 단축키(`⌥⌘I`)를 하드코딩 — 윈도우/리눅스 사용자에게 약간 부정확하지만 데스크톱 우선 앱이므로 허용 범위.

**짚어볼 점 / Watch-outs**
- 사용자 행동(재시도/리포트) 버튼이 없다. 단순 안내 화면.

```tsx path=src/App.tsx lines=39-45
  if (!loaded) {
    return (
      <div className="flex h-screen w-screen items-center justify-center bg-bg text-sm text-fg-muted">
        초기화 중...
      </div>
    );
  }
```

**분석 / Analysis (Level 2)**
- 부트스트랩 완료 전 로딩 화면. `loaded` 플래그는 `useAppStore` 안에서 `bootstrap()` 완료 후 true 로 바뀐다.
- 텍스트만 표시 — 스피너 애니메이션은 없음. 초기 데이터가 가벼우니 충분.

```tsx path=src/App.tsx lines=47-59
  return (
    <ErrorBoundary>
      <PipelineListener />
      <Routes>
        <Route element={<Layout />}>
          <Route index element={<HomePage />} />
          <Route path="projects/:id" element={<ProjectPage />} />
          <Route path="settings" element={<SettingsPage />} />
        </Route>
      </Routes>
    </ErrorBoundary>
  );
}

export default App;
```

**분석 / Analysis (Level 2)**
- ErrorBoundary 가 라우트 전체를 감싼다 → 페이지 단위 크래시도 잡힌다.
- `PipelineListener`는 DOM에 아무것도 렌더링하지 않는 부수효과 전용 컴포넌트("invisible component" 패턴).
- 중첩 라우트(`<Route element={<Layout/>}>`)를 사용해 사이드바·헤더가 모든 페이지에서 공유된다.

**짚어볼 점 / Watch-outs**
- `*` 매칭 라우트(404)가 없다. 잘못된 URL이면 빈 화면.
- `PipelineListener` 가 `ErrorBoundary` 안에 있으므로 리스너의 IPC 에러가 ErrorBoundary 트리거 가능. 하지만 컴포넌트가 null만 반환하니 사실상 안전.

**파일 수준 평가 / File-Level Verdict**
- 강점 / Strengths:
  - 부트스트랩 실패, 로딩, 정상 경로의 3-상태를 명시적으로 처리.
  - ErrorBoundary + PipelineListener + Routes 의 3종 셸이 깔끔하게 분리.
- 위험 / Risks:
  - 부트 에러 후 재시도 경로 없음 (사용자가 앱 재실행해야 함).
  - 404/와일드카드 라우트 부재.
- 개선 권고 / Recommendations:
  - `bootError` 화면에 "다시 시도" 버튼 추가 (setBootError(null) + bootstrap() 재호출).
  - `<Route path="*" element={<NotFound/>}/>` 추가.
- 재읽기 권고 레벨 / Suggested re-read level: 1 — 매우 짧고 단순. 더 깊은 리뷰는 의존하는 store/page 쪽이 우선.
