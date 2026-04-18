use std::path::{Path, PathBuf};
use std::process::Stdio;

use tokio::io::AsyncReadExt;
use tokio::process::Command;

use super::error::{AppError, AppResult};

pub const TARGET_SAMPLE_RATE: u32 = 16_000;

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
