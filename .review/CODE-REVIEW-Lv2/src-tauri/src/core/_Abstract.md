# `src-tauri/src/core/` 디렉터리 요약

> [← INDEX.md](../../../INDEX.md)

## 파일 목록

| 파일 | 줄 수 | 리뷰 |
|------|-----:|------|
| mod.rs | 11 | [mod.rs.md](./mod.rs.md) |
| error.rs | 58 | [error.rs.md](./error.rs.md) |
| models.rs | 154 | [models.rs.md](./models.rs.md) |
| db.rs | 341 | [db.rs.md](./db.rs.md) |
| media.rs | 131 | [media.rs.md](./media.rs.md) |
| asr.rs | 187 | [asr.rs.md](./asr.rs.md) |
| translate.rs | 110 | [translate.rs.md](./translate.rs.md) |
| translit.rs | 1713 | [translit.rs.md](./translit.rs.md) |
| pipeline.rs | 179 | [pipeline.rs.md](./pipeline.rs.md) |
| settings.rs | 21 | [settings.rs.md](./settings.rs.md) |

## 구조 요약

도메인 코어. `models.rs`(직렬화 모델) ↔ `db.rs`(SQLite) ↔ `pipeline.rs`(오케스트레이션) ↔ `asr.rs/media.rs/translate.rs/translit.rs`(외부 시스템 어댑터). 웹 서버도 `#[path]`로 이 모듈을 그대로 참조.

## 핵심 의존

- `rusqlite`, `whisper-rs`, `reqwest`, `tokio`, `serde`, `thiserror`, `parking_lot`, `once_cell`, `uuid`

## 리스크 시그널

- `db.rs::Db`가 단일 `Arc<Mutex<Connection>>` — 모든 쓰기 직렬화. 짧은 락 보유 시간이지만 트랜잭션 미사용 흐름이 잦음.
- `media.rs::probe_duration_ms`가 ffmpeg stderr 파싱 — locale/포맷 변화에 깨질 수 있음.
- `asr.rs::ensure_model`이 HuggingFace 직접 다운로드 — TLS root 없음 환경에선 실패.
- `translate.rs::translate_many`가 모든 세그먼트를 순차 HTTP — 응답 시간 N배.
- `translit.rs`는 거대(1,713라인) 모놀리식 + 인라인 테스트만 — 모듈 분리/스냅샷 테스트 권장.
