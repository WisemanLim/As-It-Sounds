# src-tauri/src/core/translate.rs — 코드 리뷰 / Code Review

> 리뷰 레벨 / Review Level: 2
> [← INDEX.md](../../../INDEX.md)

**파일 요약 / File Summary**
- 역할 / Role: 영어→한국어 번역 어댑터. MyMemory(기본) 또는 LibreTranslate, 또는 "none" 지원.
- 진입점에서의 거리 / Distance from entrypoint: 3 (pipeline → translate)
- 라인 수 / LOC: 110
- 주요 export / Exports: `translate_many`
- 의존 모듈 / Depends on (internal): `super::error`, `super::models::Settings`
- 사용된 외부 패키지 / External packages used: `reqwest::Client`, `serde`, `serde_json`, `urlencoding`
- 부수효과 / Side effects: 네트워크 HTTP GET/POST.
- 동시성 모델 / Concurrency model: 순차 await loop. 동시 호출 가능하지만 본 함수 자체는 직렬.
- 테스트 커버리지 흔적 / Test coverage signal: 없음.

```rust path=src-tauri/src/core/translate.rs lines=1-6
use reqwest::Client;
use serde::Deserialize;

use super::error::{AppError, AppResult};
use super::models::Settings;
```
**분석 / Analysis (Level 2)**
- `reqwest::Client` 는 호출자(pipeline) 가 공유 instance 전달 → connection pool 재사용. / Shared HTTP client passed in.

```rust path=src-tauri/src/core/translate.rs lines=10-35
pub async fn translate_many(
    settings: &Settings,
    client: &Client,
    texts: &[String],
) -> Vec<Option<String>> {
    let mut out = Vec::with_capacity(texts.len());
    match settings.translator.as_str() {
        "libretranslate" => {
            for t in texts {
                out.push(translate_libre(settings, client, t).await.ok());
            }
        }
        "none" => {
            for _ in texts {
                out.push(None);
            }
        }
        _ => {
            // default: mymemory
            for t in texts {
                out.push(translate_mymemory(settings, client, t).await.ok());
            }
        }
    }
    out
}
```
**분석 / Analysis (Level 2)**
- (CWE-252) `.ok()` 로 에러 swallow → 실패 시 Some 대신 None 만 남고 사용자에게 사유 미전달.
- 순차 호출 (`for ... await`) → N 개 세그먼트면 latency N 배. `futures::stream::iter(...).buffer_unordered(K)` 등으로 병렬화하면 큰 개선.
- "none" 처리는 모든 슬롯 None. 빈 결과로 표시. / Translator off.

**짚어볼 점 / Watch-outs**
- 외부 API rate limiting → 동시성 너무 높이면 429. 보수적으로 4~8 정도 적절.
- 번역 실패 통계/로그 없음. 디버깅 어려움.

```rust path=src-tauri/src/core/translate.rs lines=37-47
#[derive(Deserialize)]
struct MyMemoryResp {
    #[serde(rename = "responseData")]
    response_data: MyMemoryData,
}

#[derive(Deserialize)]
struct MyMemoryData {
    #[serde(rename = "translatedText")]
    translated_text: String,
}
```
**분석 / Analysis (Level 2)**
- MyMemory 응답 JSON 부분만 deserialize. 다른 필드(quotaFinished 등)는 무시. / Minimal schema, safe.

```rust path=src-tauri/src/core/translate.rs lines=49-70
async fn translate_mymemory(
    settings: &Settings,
    client: &Client,
    text: &str,
) -> AppResult<String> {
    if text.trim().is_empty() {
        return Ok(String::new());
    }
    let url = format!(
        "{}?q={}&langpair=en|ko",
        settings.translate_endpoint,
        urlencoding::encode(text)
    );
    let resp = client
        .get(&url)
        .send()
        .await?
        .error_for_status()?
        .json::<MyMemoryResp>()
        .await?;
    Ok(resp.response_data.translated_text)
}
```
**분석 / Analysis (Level 2)**
- `urlencoding::encode(text)` 로 query string 안전화. / Query injection prevented.
- 그러나 `settings.translate_endpoint` 는 사용자 설정. `"https://attacker/foo?x="` 같은 endpoint 면 `?q=...&langpair=en|ko` 가 attacker 도메인에 추가됨 → 사용자가 본인 설정한 것이라 self-pwn 수준. / User-controlled endpoint — typical for self-hosted.
- timeout 미설정 → 외부 API 가 느리면 무한 대기 가능 (CWE-1088). reqwest 기본 connect timeout 은 없지만 OS TCP 가 결국 timeout 던지긴 함.

**짚어볼 점 / Watch-outs**
- `error_for_status()` 가 4xx/5xx 를 에러로 승격. 즉, 429 (rate limit) 도 그냥 Err 로 swallow 된 후 `.ok()` 에 의해 None.
- MyMemory 무료 한도(예: IP 당 일일 5000자) 초과 시 사용자는 영문도 없이 빈 번역만 보게 됨.

```rust path=src-tauri/src/core/translate.rs lines=72-76
#[derive(Deserialize)]
struct LibreResp {
    #[serde(rename = "translatedText")]
    translated_text: String,
}
```
**분석 / Analysis (Level 2)**
- LibreTranslate 응답 형식. self-hosted 인스턴스에서 동일 필드명. / Stable schema.

```rust path=src-tauri/src/core/translate.rs lines=78-110
async fn translate_libre(
    settings: &Settings,
    client: &Client,
    text: &str,
) -> AppResult<String> {
    if text.trim().is_empty() {
        return Ok(String::new());
    }
    let mut body = serde_json::json!({
        "q": text,
        "source": "en",
        "target": "ko",
        "format": "text",
    });
    if let Some(key) = &settings.translate_api_key {
        body["api_key"] = serde_json::Value::String(key.clone());
    }
    let url = if settings.translate_endpoint.is_empty() {
        "https://libretranslate.com/translate".to_string()
    } else {
        settings.translate_endpoint.clone()
    };
    let resp = client
        .post(&url)
        .json(&body)
        .send()
        .await
        .map_err(AppError::from)?
        .error_for_status()?
        .json::<LibreResp>()
        .await?;
    Ok(resp.translated_text)
}
```
**분석 / Analysis (Level 2)**
- API key 는 JSON body 에 평문 전송. HTTPS 라면 in-transit 안전. / OK over HTTPS.
- endpoint default `libretranslate.com` 은 공식 호스트 이지만 무료는 제한적. self-host 가 일반.
- `format: "text"` 로 plain text 모드 → HTML 안 끼임.

**짚어볼 점 / Watch-outs**
- (CWE-319 cleartext over network) endpoint 가 `http://` 면 키도 평문. 사용자 자유 입력이므로 사전 경고가 좋음.
- retry/back-off 없음. 일시적 5xx 한 번이면 그 세그먼트는 영구 미번역.

**파일 수준 평가 / File-Level Verdict**
- 강점 / Strengths:
  1. `translate_many` 가 settings.translator 분기를 깔끔히 처리.
  2. 빈 텍스트 단락(`text.trim().is_empty()`) shortcut.
  3. URL encoding 사용으로 query 안전.
- 위험 / Risks:
  1. (CWE-252) `.ok()` 로 모든 번역 에러 swallow.
  2. HTTP timeout 없음 → 무한 대기 가능.
  3. 순차 await → 세그먼트 많을수록 매우 느림.
- 개선 권고 / Recommendations:
  1. `tracing::warn!` 로 실패 사유 로깅.
  2. `Client::builder().timeout(Duration::from_secs(15))` 적용.
  3. `buffer_unordered(4)` 등으로 적당한 동시성.
- 재읽기 권고 레벨 / Suggested re-read level: 2.
