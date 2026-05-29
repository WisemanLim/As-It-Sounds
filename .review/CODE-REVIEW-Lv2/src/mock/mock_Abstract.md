# mock/ — 디렉토리 개요 / Directory Abstract

> [← INDEX.md](../../INDEX.md)

## 파일 목록 / File List
| 파일 / File | LOC | 역할 / Role | 리뷰 링크 / Review |
|---|---|---|---|
| fixtures.ts | 45 | 데모/스토리북용 더미 세그먼트 생성기 | [fixtures.ts.md](./fixtures.ts.md) |

## 디렉토리 구조 / Structure Summary
개발/데모 용 더미 데이터 폴더. 현재는 `mockSegmentsForFile` 한 함수만 있으며, 4개의 영어/한글음차/번역 데모 세그먼트를 결정적으로 생성한다.

## 주요 의존 관계 / Key Dependencies
- `../types.Segment` 타입만 의존.

## 위험 신호 / Risk Signals
- `mockSegmentsForFile` 가 어디서도 import 되지 않는 것으로 보임 → dead code 가능 (grep 으로 사용처 확인 권장).
