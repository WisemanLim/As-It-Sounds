//! CLI pipeline tester for As It Sounds.
//! Usage: cargo run --manifest-path src-tauri/Cargo.toml --bin ais_test -- <audio_file> [model]
//! Default model: base (multilingual)

use std::path::PathBuf;
use std::sync::Arc;

use as_it_sounds_lib::core::{asr::AsrEngine, media, translit};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let args: Vec<String> = std::env::args().collect();
    if args.len() < 2 {
        eprintln!("Usage: ais_test <audio_file> [model=base]");
        eprintln!("  model examples: tiny, base, small, medium, large-v3");
        std::process::exit(1);
    }

    let audio = PathBuf::from(&args[1]);
    if !audio.exists() {
        eprintln!("Error: file not found: {}", audio.display());
        std::process::exit(1);
    }

    let model = args.get(2).map(String::as_str).unwrap_or("base");

    // Use same data dir as the app
    let models_dir = dirs::data_dir()
        .unwrap_or_else(std::env::temp_dir)
        .join("com.asitsounds.app")
        .join("models");

    let asr = Arc::new(AsrEngine::new(models_dir)?);

    // Ensure model (download if missing)
    eprintln!("▸ 모델: {model}");
    let model_path = asr
        .ensure_model(model, |pct, msg| {
            eprint!("\r  {msg} [{:.0}%]   ", pct * 100.0);
        })
        .await?;
    eprintln!();

    // Extract PCM
    eprintln!("▸ 오디오 추출: {}", audio.display());
    let ffmpeg = media::resolve_ffmpeg(None)?;
    let samples = media::extract_pcm_f32_mono_16k(&ffmpeg, &audio).await?;
    eprintln!("  {} 샘플 ({:.1}s)", samples.len(), samples.len() as f32 / 16000.0);

    // ASR
    eprintln!("▸ Whisper 추론 중...");
    let segments = asr.transcribe(model, model_path, samples).await?;
    eprintln!("  {} 세그먼트", segments.len());
    eprintln!();

    // Output
    let col_w = 46usize;
    println!(
        "{:<12} {:<12}  {:<col_w$}  {}",
        "시작", "종료", "원문 (STT)", "한글 음차"
    );
    println!("{}", "─".repeat(100));

    for seg in &segments {
        let hangul = translit::transliterate(&seg.text);
        println!(
            "{:<12} {:<12}  {:<col_w$}  {}",
            fmt_ms(seg.start_ms),
            fmt_ms(seg.end_ms),
            seg.text,
            hangul,
        );
    }

    Ok(())
}

fn fmt_ms(ms: i64) -> String {
    let total_s = ms / 1000;
    let ms_r = ms % 1000;
    format!("{:02}:{:02}.{:03}", total_s / 60, total_s % 60, ms_r)
}
