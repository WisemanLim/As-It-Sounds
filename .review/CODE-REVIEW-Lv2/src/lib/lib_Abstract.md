# lib/ — 디렉토리 개요 / Directory Abstract

> [← INDEX.md](../../INDEX.md)

## 파일 목록 / File List
| 파일 / File | LOC | 역할 / Role | 리뷰 링크 / Review |
|---|---|---|---|
| cn.ts | 6 | Tailwind 클래스 합성 (clsx + twMerge) | [cn.ts.md](./cn.ts.md) |
| toast.ts | 15 | 4초 자동 사라지는 미니 토스트 | [toast.ts.md](./toast.ts.md) |
| format.ts | 37 | ms 시간/상대시간 포맷팅 | [format.ts.md](./format.ts.md) |
| filePicker.ts | 99 | desktop/web 통합 파일 picker | [filePicker.ts.md](./filePicker.ts.md) |
| env.ts | 21 | `isTauri` + `apiBaseUrl` | [env.ts.md](./env.ts.md) |

## 디렉토리 구조 / Structure Summary
순수 유틸들이 모인 폴더. UI 헬퍼(`cn`, `toast`), 데이터 포맷팅(`format`), 환경 감지(`env`), 그리고 데스크톱/웹 양쪽을 추상화하는 `filePicker` 가 위치한다. 외부 의존성은 `clsx` + `tailwind-merge` (cn) 와 dynamic import 된 `@tauri-apps/*` (filePicker) 뿐이라 트리쉐이킹 친화.

## 주요 의존 관계 / Key Dependencies
- `filePicker` → `env.isTauri`, dynamic `@tauri-apps/plugin-dialog/plugin-fs`, `store/useAppStore.detectKind`.
- `cn` 은 거의 모든 컴포넌트가 사용.
- `toast` 는 사용자 피드백 표준 경로.

## 위험 신호 / Risk Signals
- `filePicker` 의 EXTS / baseName 가 `useTauriDrop.ts` 와 중복 — drift 위험.
- `format.formatRelative` 의 사용처가 불분명 (dead code 가능).
- `toast` 가 단일 색상(빨강) 만 지원 — UX 다양성 부족.
