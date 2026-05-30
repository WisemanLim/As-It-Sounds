# As It Sounds — Installation & Usage Guide (Multi-platform)

> 🌐 [한국어](Usage.md) · **English**

**Platforms:** macOS 11+ (ARM / Intel) · Windows 10/11 · Linux (Ubuntu 20.04+, Fedora, Arch)

| Platform | Build file (v0.1.0) |
| :-- | :-- |
| macOS Apple Silicon | `As It Sounds_0.1.0_aarch64.dmg` |
| macOS Intel | `As It Sounds_0.1.0_x64.dmg` |
| Windows (x64) | `As It Sounds_0.1.0_x64_en-US.msi` (or `.exe` NSIS installer) |
| Linux (Debian/Ubuntu) | `as-it-sounds_0.1.0_amd64.deb` / `as-it-sounds_0.1.0_arm64.deb` |
| Linux (Fedora/RHEL) | `as-it-sounds-0.1.0-1.x86_64.rpm` / `as-it-sounds-0.1.0-1.aarch64.rpm` |
| Linux (distro-agnostic) | `as-it-sounds_0.1.0_amd64.AppImage` |

---

## 1. Prerequisites

### 1.1 System Requirements

| Item | Minimum | Recommended |
| :-- | :-- | :-- |
| OS | macOS 11 / Windows 10 / Ubuntu 20.04 | Latest |
| CPU | x86_64 or arm64 | Apple Silicon · AVX2-capable Intel · ARM64 |
| RAM | 4 GB | 8 GB or more |
| Disk | 500 MB + Whisper model (75 MB – 3 GB) | 5 GB or more |
| Network | Required only for initial model download + translation | — |

### 1.2 Required Dependency — `ffmpeg`

`ffmpeg` is required for audio/video decoding. **Install it before launching the app.**

**macOS** (Homebrew)
```bash
brew install ffmpeg
```

**Windows** (winget or Chocolatey)
```powershell
winget install Gyan.FFmpeg
# or
choco install ffmpeg
```
Open a new PowerShell/terminal after installation (to refresh PATH).

**Linux** (Ubuntu/Debian)
```bash
sudo apt update && sudo apt install -y ffmpeg
```

**Linux** (Fedora/RHEL)
```bash
sudo dnf install -y ffmpeg
```

**Linux** (Arch)
```bash
sudo pacman -S ffmpeg
```

Verify installation:
```bash
ffmpeg -version
```

> If `ffmpeg` is in a standard path (`/opt/homebrew/bin`, `/usr/local/bin`, `/usr/bin`, `C:\ffmpeg\bin`, etc.) the app will find it automatically. For non-standard locations, go to [3.5 Settings](#35-settings) and specify the path manually.

---

## 2. Installation

### 2.1 macOS (.dmg)

**Apple Silicon (M1/M2/M3/M4):** `As It Sounds_0.1.0_aarch64.dmg`
**Intel:** `As It Sounds_0.1.0_x64.dmg`

1. Double-click the DMG → mount
2. Drag the **As It Sounds.app** icon to the **Applications** folder
3. Close the DMG window → eject the disk

**First launch — bypassing Gatekeeper (unsigned build):**

Method A — Allow via context menu (recommended)
1. In the Applications folder, **right-click** `As It Sounds.app` → **Open**
2. Click **Open** again in the warning dialog

Method B — Remove quarantine attribute (Terminal)
```bash
xattr -cr "/Applications/As It Sounds.app"
```

> **"damaged or from an unidentified developer"** message is resolved the same way. The code is identical; it simply lacks an Apple Developer ID signature.

### 2.2 Windows (.msi / .exe)

**MSI installer (recommended):**
1. Double-click `As It Sounds_0.1.0_x64_en-US.msi`
2. **"Windows protected your PC"** SmartScreen warning → click **More info** → **Run anyway**
3. Follow the installation wizard (default path: `C:\Program Files\As It Sounds\`)
4. Start menu → launch "As It Sounds"

**NSIS installer (.exe):**
- Double-click the `.exe` → install → registers in the Start menu (same process)

**Ignoring the SmartScreen warning:** This appears when Authenticode signing is absent. Click **More info > Run anyway**.

### 2.3 Linux

**Debian / Ubuntu (.deb)**
```bash
sudo dpkg -i as-it-sounds_0.1.0_amd64.deb
# If dependencies are missing:
sudo apt --fix-broken install
```
Launch from the application menu or terminal:
```bash
as-it-sounds
```

**Fedora / RHEL (.rpm)**
```bash
sudo rpm -i as-it-sounds-0.1.0-1.x86_64.rpm
# or
sudo dnf install ./as-it-sounds-0.1.0-1.x86_64.rpm
```

**AppImage (distro-agnostic)**
```bash
chmod +x As\ It\ Sounds_0.1.0_amd64.AppImage
./As\ It\ Sounds_0.1.0_amd64.AppImage
```
> **FUSE required:** On Ubuntu 22.04+, run `sudo apt install libfuse2`.

**Runtime dependencies (Linux):**
- `libwebkit2gtk-4.1-0` (or `4.0-37`) — WebView
- `libappindicator3-1` — system tray
- `librsvg2-2` — SVG rendering
- `ffmpeg` — audio decoding

Install all missing dependencies on Ubuntu at once:
```bash
sudo apt install -y libwebkit2gtk-4.1-0 libappindicator3-1 librsvg2-2 ffmpeg
```

---

## 3. Usage

### 3.1 First Launch — Automatic Whisper Model Download

On the first conversion attempt, the Whisper `base` **multilingual model (≈142 MB)** is downloaded automatically from Hugging Face. Progress is shown at the top of the screen. Downloaded once, then cached.

Storage locations by OS:

| OS | Path |
| :-- | :-- |
| macOS | `~/Library/Application Support/com.asitsounds.app/models/ggml-base.bin` |
| Windows | `%APPDATA%\com.asitsounds.app\models\ggml-base.bin` |
| Linux | `~/.local/share/com.asitsounds.app/models/ggml-base.bin` |

### 3.2 Creating a Project

Click **"New Project"** in the left sidebar or **"Start with empty project"** on the home screen.

### 3.3 Adding Files

- **Drag and drop** — drag one or more files from your file manager (Finder/Explorer/Files) onto the app window
- **Add file button** — button in the top-right corner → OS native file picker dialog

Supported formats:
- Audio: MP3, WAV, M4A, FLAC, OGG
- Video: MP4, MOV, MKV, WEBM, AVI

### 3.4 Running a Conversion

1. After adding files, click **"Start conversion"** in the top-right
2. Processing stages:
   - `decode` — extracts 16 kHz mono PCM via ffmpeg
   - `asr` — Whisper inference (automatic language detection)
   - `translate` — Korean translation + Korean phonetic transcription

3. Per-segment output example:
```
Original (STT):         A. I don't think we've received it.
Korean phonetics:       에이. 아이 돈트 씽크 위브 리시브드 잇.
Korean translation:     A. 우리는 그것을 받지 못한 것 같습니다.
```

### 3.5 Settings (⚙️ bottom of sidebar)

| Item | Description | Default |
| :-- | :-- | :-- |
| **Whisper model** | tiny (75 MB) – large-v3 (2.9 GB). Multilingual recommended (no `.en` suffix) | `base` |
| **Translation engine** | MyMemory / LibreTranslate / None | MyMemory |
| **Translation endpoint** | Specify a self-hosted URL | MyMemory default |
| **LibreTranslate API Key** | (optional) | None |
| **ffmpeg path** | Enter absolute path if installed in a non-standard location | Auto-detect |

**Model selection guide (speed based on M2):**

| Model | Size | Speed | Accuracy |
| :-- | :-- | :-- | :-- |
| `tiny` | 75 MB | Very fast | Low |
| `base` | 142 MB | Fast | Moderate (default) |
| `small` | 466 MB | Moderate | Good |
| `medium` | 1.5 GB | Slow | Very good |
| `large-v3` | 2.9 GB | Very slow | Best |

Even for English-only content, the multilingual model is recommended — the app auto-detects language and then applies phonetic conversion.

### 3.6 Media Player Integration

- Segments are highlighted automatically as playback position advances
- Click a segment to seek to that timestamp
- ±15-second jump, volume and speed controls

---

## 4. Phonetic Transcription (Korean Romanization) Features

**Sound-based** transcription — Korean syllables matching actual pronunciation, not dictionary word matching:

| Input | Output | Reason |
| :-- | :-- | :-- |
| `A.` / `B.` / `C.` | 에이. / 비. / 씨. | Single uppercase letter → alphabet name |
| `corner` | 코너 | `c` context detection (c+o = ㅋ) |
| `city` | 시티 | `c` context detection (c+i = ㅅ) |
| `we've` | 위브 | English `we'` contraction handling |
| `received` | 리시브드 | `re-` unstressed + `ei` long vowel + `-ed` past tense |
| `catering` | 케이터잉 | open syllable long vowel |
| `こんにちは` (Japanese) | 콩니치하 | direct kana mapping |
| `bonjour` (French) | 봉주흐 | nasal + silent final consonant |
| `Москва` (Russian) | 모스크바 | Cyrillic syllable assembly |

**Supported languages (transcription rules):** English · Japanese · French · Spanish · German · Russian
**Other languages:** Displayed as-is from Whisper STT output.

---

## 5. Data Storage Locations

| OS | Data directory |
| :-- | :-- |
| macOS | `~/Library/Application Support/com.asitsounds.app/` |
| Windows | `%APPDATA%\com.asitsounds.app\` (typically `C:\Users\<user>\AppData\Roaming\com.asitsounds.app\`) |
| Linux | `~/.local/share/com.asitsounds.app/` |

Directory structure (all platforms):
```
com.asitsounds.app/
├── as-it-sounds.sqlite      # projects, files, segments, settings
└── models/
    └── ggml-base.bin        # downloaded Whisper model
```

### 5.1 Backup

**macOS / Linux**
```bash
cp -R ~/Library/Application\ Support/com.asitsounds.app/ ~/Backups/as-it-sounds/   # macOS
cp -R ~/.local/share/com.asitsounds.app/ ~/Backups/as-it-sounds/                    # Linux
```

**Windows (PowerShell)**
```powershell
Copy-Item -Recurse "$env:APPDATA\com.asitsounds.app" "$env:USERPROFILE\Backups\as-it-sounds"
```

### 5.2 Full Reset

**macOS**
```bash
rm -rf ~/Library/Application\ Support/com.asitsounds.app/
```

**Linux**
```bash
rm -rf ~/.local/share/com.asitsounds.app/
```

**Windows (PowerShell)**
```powershell
Remove-Item -Recurse -Force "$env:APPDATA\com.asitsounds.app"
```

> To reset only the database while keeping the model, delete only `as-it-sounds.sqlite`.

---

## 6. Troubleshooting

### 6.1 Common

| Symptom | Cause / Fix |
| :-- | :-- |
| **`ffmpeg not found`** | ffmpeg not installed or missing from PATH → install per section 1.2, or specify the absolute path in Settings |
| **Whisper model download fails** | Check firewall/proxy. Manual download: `https://huggingface.co/ggerganov/whisper.cpp/resolve/main/ggml-base.bin` → save as `ggml-base.bin` in the `models/` subdirectory of the data directory (section 5) |
| **Translation result is empty** | MyMemory daily limit (1000 words) exceeded or network blocked → switch to LibreTranslate or "None" in Settings |
| **Conversion is very slow** | Large model (medium/large) → switch to `base`/`small`. Use `tiny` if CPU cores are limited |
| **Video plays but audio is silent** | System WebView codec not supported. ASR results are unaffected |

### 6.2 macOS Only

| Symptom | Fix |
| :-- | :-- |
| **"damaged or from an unidentified developer"** | `xattr -cr "/Applications/As It Sounds.app"` |
| **App quits immediately** | Open `Console.app`, search "As It Sounds". Check that ffmpeg is installed first |
| **View live logs** | `log stream --predicate 'subsystem == "com.asitsounds.app"' --info` |
| **Intel DMG installed on Apple Silicon** | Rosetta will run it automatically, but not recommended. Use the ARM DMG |

### 6.3 Windows Only

| Symptom | Fix |
| :-- | :-- |
| **SmartScreen "Windows protected your PC"** | **More info > Run anyway** |
| **WebView2 missing error** | Included by default on Windows 11. On Win 10, install separately: `winget install Microsoft.EdgeWebView2Runtime` |
| **App shows white screen / closes immediately** | WebView2 not installed or corrupted. Reinstall with the command above |
| **Event Viewer logs** | `Windows Logs > Application` → search for source `As It Sounds` or `as-it-sounds` |
| **ffmpeg not recognized in PATH** | Open a new terminal/Explorer. If still not working, enter the absolute path `C:\path\to\ffmpeg.exe` in Settings |

### 6.4 Linux Only

| Symptom | Fix |
| :-- | :-- |
| **`error while loading shared libraries: libwebkit2gtk-4.1.so.0`** | `sudo apt install libwebkit2gtk-4.1-0` (or `-4.0-37` for older versions) |
| **`libappindicator3.so.1: cannot open shared object`** | `sudo apt install libappindicator3-1` |
| **AppImage fails with `dlopen(): libfuse.so.2`** | `sudo apt install libfuse2` |
| **Rendering broken under Wayland** | Run in X11 mode: `GDK_BACKEND=x11 as-it-sounds` |
| **System tray/menu not showing** | In GNOME, install `gnome-shell-extension-appindicator` and log out/in |
| **View logs** | `journalctl --user -f` (while app is running) or run `as-it-sounds` directly from a terminal |

---

## 7. Keyboard Shortcuts

| Function | Shortcut |
| :-- | :-- |
| Play/pause | `Space` (when HTML player is focused) |
| Developer tools (macOS) | `⌥ ⌘ I` |
| Developer tools (Win/Linux) | `Ctrl+Shift+I` |

> App-level shortcuts (new project, add file, start conversion, etc.) are not available in v0.1.0. Use the buttons.

---

## 8. Frequently Asked Questions

**Q. Does it work offline?**
A. ASR and phonetic transcription are fully offline. Only the initial model download and Korean translation require a network connection. Set translation to "None" for 100% offline operation.

**Q. Can I pre-download the model?**

macOS / Linux:
```bash
# macOS
MODELS_DIR="$HOME/Library/Application Support/com.asitsounds.app/models"
# Linux
MODELS_DIR="$HOME/.local/share/com.asitsounds.app/models"

mkdir -p "$MODELS_DIR"
curl -L -o "$MODELS_DIR/ggml-base.bin" \
  https://huggingface.co/ggerganov/whisper.cpp/resolve/main/ggml-base.bin
```

Windows (PowerShell):
```powershell
$dir = "$env:APPDATA\com.asitsounds.app\models"
New-Item -ItemType Directory -Force -Path $dir | Out-Null
Invoke-WebRequest -OutFile "$dir\ggml-base.bin" `
  https://huggingface.co/ggerganov/whisper.cpp/resolve/main/ggml-base.bin
```

**Q. Can other languages be transcribed into Korean phonetics?**
A. Yes. Whisper auto-detects the language → the transcription module detects the script (Latin/kana/Cyrillic/Hangul, etc.) → applies that language's pronunciation rules to produce Korean phonetics.

**Q. SRT/VTT export?**
A. Not supported in v0.1.0. Planned in **Phase 8** roadmap.

**Q. Real-time microphone STT?**
A. Not supported in v0.1.0. Planned in **Phase 9** roadmap.

**Q. Verify conversion results (CLI)?**
A. Developer-mode CLI tool:
```bash
./test.sh .samples/01_19_04.mp3      # macOS / Linux
.\test.sh .samples\01_19_04.mp3       # Windows (Git Bash or WSL)
```
Clone the GitHub repository → `pnpm install` → `./test.sh <audio-file>`.

**Q. Can I build installers for all platforms at once?**
A. **Each platform can only be built reliably on its native host.** This project has C/C++ native dependencies (whisper.cpp, ring, rusqlite) that make cross-compilation impractical:

| Host | Can build |
| :-- | :-- |
| macOS | macOS ARM/Intel/Universal |
| Windows | Windows x64 |
| Linux | Linux x64/arm64 |

`pnpm tauri:build:all` attempts all targets, but targets that differ from the host OS will fail. **GitHub Actions CI is recommended** — push a tag (`git tag v0.1.0 && git push --tags`) to trigger simultaneous builds on macOS/Win/Linux native runners, followed by an automatic release.

Alternative: build directly inside a Windows VM (Parallels/UTM/VMware) or a Linux VM.

---

## 9. License & Credits

- **As It Sounds** — MIT
- **whisper.cpp** — MIT
- **ffmpeg** — LGPL (installed separately)
- **Whisper model weights** — MIT (Hugging Face `ggerganov/whisper.cpp`)

Questions/issues: see GitHub Issues or the project README.
