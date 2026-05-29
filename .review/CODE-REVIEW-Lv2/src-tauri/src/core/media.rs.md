# src-tauri/src/core/media.rs — 코드 리뷰 / Code Review

> 리뷰 레벨 / Review Level: 2
> [← INDEX.md](../../../INDEX.md)

**파일 요약 / File Summary**
- 역할 / Role: ffmpeg 바이너리 탐색, 미디어를 16kHz mono f32 PCM 으로 디코드, duration 추출.
- 진입점에서의 거리 / Distance from entrypoint: 3 (pipeline → media)
- 라인 수 / LOC: 131
- 주요 export / Exports: `TARGET_SAMPLE_RATE`, `resolve_ffmpeg`, `extract_pcm_f32_mono_16k`, `probe_duration_ms`
- 의존 모듈 / Depends on (internal): `super::error`
- 사용된 외부 패키지 / External packages used: `tokio::process::Command`, `tokio::io`
- 부수효과 / Side effects: 외부 ffmpeg 프로세스 spawn, PATH/디렉토리 검색.
- 동시성 모델 / Concurrency model: tokio async/await. 자식 프로세스 stdout pipe.
- 테스트 커버리지 흔적 / Test coverage signal: 없음.

> **iOS 주의 / iOS Caveat**: 이 모듈은 외부 `ffmpeg` 바이너리에 의존. iOS 앱 샌드박스에서는 `Command::spawn` 으로 다른 실행파일을 띄울 수 없으므로 **iOS 빌드 시 동작 불가**. 이미 README/이슈에 알려진 사항이며, iOS 지원을 원한다면 `ffmpeg-next`(libavcodec 정적 링크) 또는 platform native AVFoundation 사용으로 대체 필요.

```rust path=src-tauri/src/core/media.rs lines=1-9
use std::path::{Path, PathBuf};
use std::process::Stdio;

use tokio::io::AsyncReadExt;
use tokio::process::Command;

use super::error::{AppError, AppResult};

pub const TARGET_SAMPLE_RATE: u32 = 16_000;
```
**분석 / Analysis (Level 2)**
- 16kHz 가 Whisper 기본 입력 sample rate. 상수로 노출. / Whisper expects 16kHz.

```rust path=src-tauri/src/core/media.rs lines=11-28
pub fn resolve_ffmpeg(custom: Option<&str>) -> AppResult<PathBuf> {
    if let Some(p) = custom {
        let path = PathBuf::from(p);
        if path.exists() {
            return Ok(path);
        }
    }
    for candidate in ["/opt/homebrew/bin/ffmpeg", "/usr/local/bin/ffmpeg", "/usr/bin/ffmpeg"] {
        let p = PathBuf::from(candidate);
        if p.exists() {
            return Ok(p);
        }
    }
    if which_in_path("ffmpeg") {
        return Ok(PathBuf::from("ffmpeg"));
    }
    Err(AppError::FfmpegNotFound)
}
```
**분석 / Analysis (Level 2)**
- 우선순위: custom path → /opt/homebrew (Apple Silicon brew) → /usr/local → /usr/bin → PATH. / macOS-friendly priority list.
- Windows 경로 후보 없음. Windows 사용자는 `ffmpeg.exe` 를 PATH 에 두거나 settings 에 명시.

**짚어볼 점 / Watch-outs**
- (CWE-426 untrusted search path) `which_in_path("ffmpeg")` 가 PATH 의 어떤 디렉토리에서든 ffmpeg 를 찾음. 악성 PATH 가 prepend 되면 임의 바이너리 실행 (실제 위험은 single-user desktop 에서는 낮음).
- iOS 에서는 모든 후보가 실패하여 `FfmpegNotFound` 반환. 이후 `Pipeline::run` 에서 즉시 종료.

```rust path=src-tauri/src/core/media.rs lines=30-40
fn which_in_path(cmd: &str) -> bool {
    if let Ok(paths) = std::env::var("PATH") {
        for p in std::env::split_paths(&paths) {
            let candidate = p.join(cmd);
            if candidate.exists() {
                return true;
            }
        }
    }
    false
}
```
**분석 / Analysis (Level 2)**
- `which` crate 의 미니 대용. Windows `.exe` 확장자 미고려 (Windows 에서 `ffmpeg.exe` 검색 못함). / Doesn't append `.exe` on Windows.

**짚어볼 점 / Watch-outs**
- 검색 결과를 PathBuf 으로 반환하지 않고 "ffmpeg" string 으로만 돌리므로 spawn 시점에 OS resolve. 큰 문제는 아니지만 명시적이지 않음.

```rust path=src-tauri/src/core/media.rs lines=42-67
/// Extract 16kHz mono f32 PCM samples from an audio or video file using ffmpeg.
pub async fn extract_pcm_f32_mono_16k(
    ffmpeg: &Path,
    input: &Path,
) -> AppResult<Vec<f32>> {
    let mut child = Command::new(ffmpeg)
        .args([
            "-hide_banner",
            "-loglevel",
            "error",
            "-i",
        ])
        .arg(input)
        .args([
            "-f",
            "f32le",
            "-ac",
            "1",
            "-ar",
            &TARGET_SAMPLE_RATE.to_string(),
            "-",
        ])
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .map_err(|e| AppError::Ffmpeg(format!("spawn failed: {e}")))?;
```
**분석 / Analysis (Level 2)**
- ffmpeg 인자: `-i <input> -f f32le -ac 1 -ar 16000 -` → stdout 으로 raw PCM. / Pipe to stdout.
- **Command injection 분석**: 인자는 모두 `.arg(...)` / `.args([...])` 로 개별 전달, shell 거치지 않음. `input` 이 `; rm -rf /` 같아도 ffmpeg 가 단순 파일명으로 해석. **안전**. / No shell — args passed as `&[OsStr]`, safe from injection.

**짚어볼 점 / Watch-outs**
- `-loglevel error` 로 stderr 노이즈 최소화하지만 디버깅 시 자세한 로그 부족할 수 있음.

```rust path=src-tauri/src/core/media.rs lines=69-101
    let mut stdout = child
        .stdout
        .take()
        .ok_or_else(|| AppError::Ffmpeg("no stdout".into()))?;
    let mut stderr = child
        .stderr
        .take()
        .ok_or_else(|| AppError::Ffmpeg("no stderr".into()))?;

    let mut bytes = Vec::<u8>::with_capacity(1024 * 1024);
    stdout.read_to_end(&mut bytes).await?;

    let status = child.wait().await?;
    if !status.success() {
        let mut err_bytes = Vec::new();
        let _ = stderr.read_to_end(&mut err_bytes).await;
        return Err(AppError::Ffmpeg(
            String::from_utf8_lossy(&err_bytes).to_string(),
        ));
    }

    // Reinterpret bytes as little-endian f32 samples.
    if bytes.len() % 4 != 0 {
        return Err(AppError::Ffmpeg(
            "pcm byte count not aligned to f32".into(),
        ));
    }
    let mut samples = Vec::with_capacity(bytes.len() / 4);
    for chunk in bytes.chunks_exact(4) {
        samples.push(f32::from_le_bytes([chunk[0], chunk[1], chunk[2], chunk[3]]));
    }
    Ok(samples)
}
```
**분석 / Analysis (Level 2)**
- (CWE-410 deadlock) 매우 큰 입력의 경우 stderr 가 OS pipe buffer (보통 64KB) 를 채우면 ffmpeg 가 멈춤. 여기서는 stdout 만 먼저 비우고 stderr 는 wait 후 읽음 → 큰 stderr 출력에 대해 잠재 데드락. 다행히 `-loglevel error` 로 stderr 가 거의 비기 때문에 실제 발생 어려움. / Potential pipe buffer deadlock on very large stderr.
- `read_to_end` 는 메모리에 전부 적재 → 1시간 분량 16kHz f32 = 230MB. 큰 영상은 OOM 가능. / Big files load fully into RAM.
- byte → f32 reinterpret 시 `chunks_exact(4)` 안전. 4 정렬 검사 후 진행.

**짚어볼 점 / Watch-outs**
- Streaming chunked 처리로 segment 별 partial transcription 도 고려해 봄직.
- stderr 도 동시에 비울 수 있도록 `tokio::join!` 사용 권장.

```rust path=src-tauri/src/core/media.rs lines=103-131
pub async fn probe_duration_ms(ffmpeg: &Path, input: &Path) -> AppResult<Option<i64>> {
    // ffprobe would be ideal; fall back to decoding header via ffmpeg -i.
    let output = Command::new(ffmpeg)
        .args(["-hide_banner", "-i"])
        .arg(input)
        .stdout(Stdio::null())
        .stderr(Stdio::piped())
        .output()
        .await
        .map_err(|e| AppError::Ffmpeg(e.to_string()))?;
    let stderr = String::from_utf8_lossy(&output.stderr);
    // Duration: 00:00:10.12,
    for line in stderr.lines() {
        if let Some(idx) = line.find("Duration:") {
            let rest = &line[idx + 9..];
            let dur = rest.split(',').next().unwrap_or("").trim();
            // hh:mm:ss.ms
            let parts: Vec<&str> = dur.split(':').collect();
            if parts.len() == 3 {
                let hours: f64 = parts[0].trim().parse().unwrap_or(0.0);
                let mins: f64 = parts[1].trim().parse().unwrap_or(0.0);
                let secs: f64 = parts[2].trim().parse().unwrap_or(0.0);
                let total_ms = ((hours * 3600.0 + mins * 60.0 + secs) * 1000.0) as i64;
                return Ok(Some(total_ms));
            }
        }
    }
    Ok(None)
}
```
**분석 / Analysis (Level 2)**
- `-i <input>` 만으로 ffmpeg 가 stderr 에 metadata 를 출력 → grep "Duration:". 흔한 hack.
- `ffmpeg -i ...` 만 호출하면 exit code 가 1 (no output specified). `.output()` 는 status code 무시. / Returns Ok regardless of exit status.
- 파싱이 라틴/로케일 의존 (`,` 분리). 일부 로케일에선 다른 구분자? ffmpeg 는 영어 출력이 일반이라 OK.

**짚어볼 점 / Watch-outs**
- ffmpeg 가 stderr 에 출력하는 형식은 버전마다 미세 차이 → ffprobe 사용이 더 robust. ffprobe 도 같은 binary suite 라 추가 설치 부담 없음.
- duration 추출 실패해도 Ok(None) 반환 → 호출자는 그냥 무시. 안전.

**파일 수준 평가 / File-Level Verdict**
- 강점 / Strengths:
  1. ffmpeg 인자를 OsStr slice 로 전달 → command injection 차단.
  2. 절대 경로 후보 + PATH fallback 으로 대부분의 macOS/Linux 환경 커버.
  3. PCM byte 재해석 시 4 정렬 검증.
- 위험 / Risks:
  1. **iOS 미지원**: ffmpeg 외부 spawn 불가. (이미 알려진 이슈)
  2. 큰 미디어 파일 시 전체 PCM 메모리 적재 → OOM 가능 (CWE-770).
  3. stderr pipe buffer 데드락 잠재성 (실제로는 -loglevel error 로 거의 비어 발생 어려움).
- 개선 권고 / Recommendations:
  1. `tokio::join!(stdout_read, stderr_read)` 로 동시 drain.
  2. 청크 단위 streaming (segment 단위 incremental whisper) 디자인.
  3. iOS 빌드 target 시 `cfg!(target_os = "ios")` 로 alternative backend 분기.
- 재읽기 권고 레벨 / Suggested re-read level: 3 — streaming/iOS 대안 고민이 필요하면.
