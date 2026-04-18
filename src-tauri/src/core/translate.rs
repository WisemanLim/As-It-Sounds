use reqwest::Client;
use serde::Deserialize;

use super::error::{AppError, AppResult};
use super::models::Settings;

/// Translate a list of English sentences into Korean using the configured
/// engine.  Falls back to returning None on any failure so that the pipeline
/// still produces transcript + transliteration.
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

#[derive(Deserialize)]
struct LibreResp {
    #[serde(rename = "translatedText")]
    translated_text: String,
}

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
