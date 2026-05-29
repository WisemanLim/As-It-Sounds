# src/components/Layout.tsx — 코드 리뷰 / Code Review

> 리뷰 레벨 / Review Level: 2
> [← INDEX.md](../../INDEX.md)

**파일 요약 / File Summary**
- 역할 / Role: 사이드바 + 메인 영역의 2단 그리드 레이아웃. react-router `<Outlet/>`이 들어가는 자리.
- 진입점에서의 거리 / Distance from entrypoint: 2 (App → here)
- 라인 수 / LOC: 13
- 주요 export / Exports: `Layout`
- 의존 모듈 / Depends on (internal): `./Sidebar`
- 사용된 외부 패키지 / External packages used: `react-router-dom`
- 부수효과 / Side effects: 없음
- 동시성 모델 / Concurrency model: N/A
- 테스트 커버리지 흔적 / Test coverage signal: 없음

## Step 5 — 라인별 분석 / Line-by-line walk

```tsx path=src/components/Layout.tsx lines=1-13
import { Outlet } from "react-router-dom";
import { Sidebar } from "./Sidebar";

export function Layout() {
  return (
    <div className="flex h-screen w-screen overflow-hidden bg-bg text-fg">
      <Sidebar />
      <main className="flex min-w-0 flex-1 flex-col">
        <Outlet />
      </main>
    </div>
  );
}
```

**분석 / Analysis (Level 2)**
- `<Outlet/>` 은 부모 라우트(`<Route element={<Layout/>}>`)의 자식 라우트가 렌더링되는 자리. App.tsx 의 라우트 구조와 짝.
- `h-screen w-screen overflow-hidden` — 전체 뷰포트를 잡고 외부 스크롤 차단. 내부 영역(`main`)이 자체 스크롤을 책임진다.
- `min-w-0` 패턴 — flex 컨테이너 안에서 자식의 truncate 가 작동하게 하는 표준 트릭.

**짚어볼 점 / Watch-outs**
- 모바일 뷰포트 고려 없음(`flex` 좌우 배치만). desktop/web 모두 데스크탑 가정.

**파일 수준 평가 / File-Level Verdict**
- 강점 / Strengths:
  - 매우 단순하고 의도가 명확.
  - `min-w-0` 트릭이 포함되어 child truncate 가 동작.
- 위험 / Risks:
  - 모바일 뷰포트 미지원 (낮은 위험, 의도된 선택일 수 있음).
- 개선 권고 / Recommendations:
  - 필요 시 `md:flex-row flex-col` 패턴으로 작은 화면 대응 가능.
- 재읽기 권고 레벨 / Suggested re-read level: 0.
