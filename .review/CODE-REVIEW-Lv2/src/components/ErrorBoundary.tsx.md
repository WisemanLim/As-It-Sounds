# src/components/ErrorBoundary.tsx — 코드 리뷰 / Code Review

> 리뷰 레벨 / Review Level: 2
> [← INDEX.md](../../INDEX.md)

**파일 요약 / File Summary**
- 역할 / Role: React 렌더 사이클에서 던져진 예외를 잡아 폴백 UI 표시.
- 진입점에서의 거리 / Distance from entrypoint: 2 (App → here)
- 라인 수 / LOC: 45
- 주요 export / Exports: `ErrorBoundary` (class component)
- 의존 모듈 / Depends on (internal): 없음
- 사용된 외부 패키지 / External packages used: `react`
- 부수효과 / Side effects: `console.error`
- 동시성 모델 / Concurrency model: N/A
- 테스트 커버리지 흔적 / Test coverage signal: 없음

## Step 5 — 라인별 분석 / Line-by-line walk

```tsx path=src/components/ErrorBoundary.tsx lines=1-10
import { Component, type ReactNode } from "react";

interface Props {
  children: ReactNode;
}

interface State {
  error: Error | null;
}
```

**분석 / Analysis (Level 2)**
- 함수 컴포넌트 + hook 환경에서 유일하게 클래스 컴포넌트가 필요한 케이스 = ErrorBoundary. React 가 hook 기반 API 를 아직 제공하지 않는다.

```tsx path=src/components/ErrorBoundary.tsx lines=11-20
export class ErrorBoundary extends Component<Props, State> {
  state: State = { error: null };

  static getDerivedStateFromError(error: Error): State {
    return { error };
  }

  componentDidCatch(error: Error, info: React.ErrorInfo) {
    console.error("[UI crash]", error, info.componentStack);
  }
```

**분석 / Analysis (Level 2)**
- `getDerivedStateFromError` (정적) → state 갱신만 담당.
- `componentDidCatch` (인스턴스) → 로깅 등 부수효과 담당.
- 두 메서드의 책임 분리는 React 공식 권장.

**짚어볼 점 / Watch-outs**
- `componentDidCatch` 에서 외부 텔레메트리(Sentry 등)로 전송하지 않음 → 운영 시 사용자 환경의 크래시 추적이 어렵다 (data observability 한계).

```tsx path=src/components/ErrorBoundary.tsx lines=22-43
  render() {
    if (this.state.error) {
      return (
        <div className="flex h-screen w-screen flex-col items-center justify-center gap-4 bg-bg p-8 text-center">
          <div className="text-base font-semibold text-red-400">
            화면을 그리는 중 오류가 발생했습니다
          </div>
          <pre className="max-w-2xl whitespace-pre-wrap break-words rounded-lg bg-bg-elevated p-4 text-left font-mono text-xs text-fg-muted">
            {this.state.error.message}
            {"\n\n"}
            {this.state.error.stack}
          </pre>
          <button
            className="btn-outline"
            onClick={() => this.setState({ error: null })}
          >
            다시 시도
          </button>
        </div>
      );
    }
    return this.props.children;
  }
}
```

**분석 / Analysis (Level 2)**
- "다시 시도" 버튼이 state 를 reset 하므로 사용자가 직접 복구 가능 → App.tsx 의 bootError 화면에는 없는 강점.
- React 가 `{string}` 을 텍스트 노드로 그리므로 stack 출력은 XSS 안전.

**짚어볼 점 / Watch-outs**
- 같은 에러가 즉시 다시 발생하면 다시 폴백으로 떨어짐 — 무한 루프는 React 가 막지만 사용자에겐 깜빡임.
- Production 빌드에서 stack 노출 → 보안상 큰 문제는 없으나 minified 라 가독성 낮음. (개발용 메시지로 충분)

**파일 수준 평가 / File-Level Verdict**
- 강점 / Strengths:
  - 표준 ErrorBoundary 패턴 충실히 구현.
  - "다시 시도" 버튼으로 사용자 복구 경로 제공.
- 위험 / Risks:
  - 운영 환경 에러 텔레메트리 없음.
- 개선 권고 / Recommendations:
  - `componentDidCatch` 에 Sentry/Statsig 등 외부 로그 훅 자리 마련.
  - 에러 종류에 따른 차등 메시지(예: "변환 실패" vs "예기치 못한 오류") 고려.
- 재읽기 권고 레벨 / Suggested re-read level: 0.
