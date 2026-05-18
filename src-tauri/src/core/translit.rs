//! Multilingual audio phonetic → Korean Hangul transliteration.
//!
//! Detects the dominant script in each sentence and routes to:
//!   Latin (ASCII + extended) → English / French / Spanish / German (auto-detected by diacritics)
//!   Hiragana / Katakana      → Japanese kana direct mapping
//!   Hangul                   → pass-through (already Korean)
//!   Cyrillic                 → Russian syllable-assembly rules
//!   CJK / Arabic / other     → pass-through

use once_cell::sync::Lazy;
use std::collections::HashMap;

// ─── Hangul composition ──────────────────────────────────────────────────────

fn compose(onset: u32, vowel: u32, coda: u32) -> char {
    let code = 0xAC00 + (onset * 21 + vowel) * 28 + coda;
    char::from_u32(code).unwrap_or('?')
}

// choseong indices
// 0 ㄱ 2 ㄴ 3 ㄷ 5 ㄹ 6 ㅁ 7 ㅂ 9 ㅅ 11 ㅇ 12 ㅈ 14 ㅊ 15 ㅋ 16 ㅌ 17 ㅍ 18 ㅎ
const NO_CODA: u32 = 0;
const C_NG: u32 = 21; // ㅇ jongseong
const C_L: u32 = 8;  // ㄹ jongseong
const C_M: u32 = 16; // ㅁ jongseong
const C_N: u32 = 4;  // ㄴ jongseong
const C_K: u32 = 1;  // ㄱ jongseong
const C_T: u32 = 19; // ㅅ jongseong (loanword convention for -t)
const C_P: u32 = 17; // ㅂ jongseong

#[derive(Clone, Copy)]
enum Vowel {
    A, Ae, Eo, E, I, O, U, Eu, Ya, Yeo, Yu, Yo, Wa, Wi, Oe, Ui,
}

impl Vowel {
    fn idx(self) -> u32 {
        match self {
            Vowel::A   => 0,  Vowel::Ae  => 1,  Vowel::Ya  => 2,
            Vowel::Eo  => 4,  Vowel::E   => 5,  Vowel::Yeo => 6,
            Vowel::O   => 8,  Vowel::Wa  => 9,  Vowel::Oe  => 11,
            Vowel::Yo  => 12, Vowel::U   => 13, Vowel::Wi  => 16,
            Vowel::Yu  => 17, Vowel::Eu  => 18, Vowel::Ui  => 19,
            Vowel::I   => 20,
        }
    }
}

fn push_syllable(out: &mut String, onset: u32, vowel: Vowel, coda: u32) {
    out.push(compose(onset, vowel.idx(), coda));
}

fn onset_to_coda(onset: u32) -> Option<u32> {
    match onset {
        0 | 15 => Some(C_K),
        2      => Some(C_N),
        3 | 16 => Some(C_T),
        5      => Some(C_L),
        6      => Some(C_M),
        7 | 17 => Some(C_P),
        9      => Some(C_T),
        _      => None,
    }
}

fn try_attach_coda(out: &mut String, onset: u32) -> bool {
    if let Some(last) = out.pop() {
        let cu = last as u32;
        if (0xAC00..=0xD7A3).contains(&cu) {
            let rel = cu - 0xAC00;
            if rel % 28 == 0 {
                if let Some(coda) = onset_to_coda(onset) {
                    if let Some(ch) = char::from_u32(cu + coda) {
                        out.push(ch);
                        return true;
                    }
                }
            }
        }
        out.push(last);
    }
    false
}

fn is_vowel_char(c: char) -> bool {
    matches!(c.to_ascii_lowercase(), 'a' | 'e' | 'i' | 'o' | 'u' | 'y')
}

// ─── Script detection ────────────────────────────────────────────────────────

#[derive(Clone, Copy, PartialEq)]
enum Script { Latin, Kana, Hangul, Cyrillic, Cjk, Other }

fn char_script(c: char) -> Script {
    let cp = c as u32;
    match cp {
        0x3040..=0x309F | 0x30A0..=0x30FF => Script::Kana,
        0xAC00..=0xD7AF | 0x1100..=0x11FF | 0x3130..=0x318F => Script::Hangul,
        0x0400..=0x04FF => Script::Cyrillic,
        0x4E00..=0x9FFF | 0x3400..=0x4DBF | 0x20000..=0x2A6DF => Script::Cjk,
        0x0021..=0x007E | 0x00C0..=0x024F => Script::Latin,
        _ => Script::Other,
    }
}

fn dominant_script(text: &str) -> Script {
    let mut counts = [0usize; 5]; // Latin, Kana, Hangul, Cyrillic, Cjk
    for c in text.chars().filter(|c| c.is_alphabetic()) {
        match char_script(c) {
            Script::Latin    => counts[0] += 1,
            Script::Kana     => counts[1] += 1,
            Script::Hangul   => counts[2] += 1,
            Script::Cyrillic => counts[3] += 1,
            Script::Cjk      => counts[4] += 1,
            Script::Other    => {}
        }
    }
    let max_idx = counts
        .iter()
        .enumerate()
        .max_by_key(|&(_, &v)| v)
        .map(|(i, _)| i)
        .unwrap_or(0);
    [Script::Latin, Script::Kana, Script::Hangul, Script::Cyrillic, Script::Cjk][max_idx]
}

// ─── Japanese kana → Korean ──────────────────────────────────────────────────

fn kata_to_hira(c: char) -> char {
    let cp = c as u32;
    if (0x30A1..=0x30F6).contains(&cp) {
        char::from_u32(cp - 0x60).unwrap_or(c)
    } else {
        c
    }
}

static KANA_MAP: Lazy<HashMap<&'static str, &'static str>> = Lazy::new(|| {
    #[rustfmt::skip]
    let entries: &[(&str, &str)] = &[
        // Combined yōon (match before single)
        ("きゃ","캬"),("きゅ","큐"),("きょ","쿄"),
        ("ぎゃ","갸"),("ぎゅ","규"),("ぎょ","교"),
        ("しゃ","샤"),("しゅ","슈"),("しょ","쇼"),
        ("じゃ","자"),("じゅ","주"),("じょ","조"),
        ("ちゃ","차"),("ちゅ","추"),("ちょ","초"),
        ("にゃ","냐"),("にゅ","뉴"),("にょ","뇨"),
        ("ひゃ","햐"),("ひゅ","휴"),("ひょ","효"),
        ("びゃ","뱌"),("びゅ","뷰"),("びょ","뵤"),
        ("ぴゃ","퍄"),("ぴゅ","퓨"),("ぴょ","표"),
        ("みゃ","먀"),("みゅ","뮤"),("みょ","묘"),
        ("りゃ","랴"),("りゅ","류"),("りょ","료"),
        // Vowels
        ("あ","아"),("い","이"),("う","우"),("え","에"),("お","오"),
        // K/G
        ("か","카"),("き","키"),("く","쿠"),("け","케"),("こ","코"),
        ("が","가"),("ぎ","기"),("ぐ","구"),("げ","게"),("ご","고"),
        // S/Z
        ("さ","사"),("し","시"),("す","스"),("せ","세"),("そ","소"),
        ("ざ","자"),("じ","지"),("ず","즈"),("ぜ","제"),("ぞ","조"),
        // T/D
        ("た","타"),("ち","치"),("つ","츠"),("て","테"),("と","토"),
        ("だ","다"),("ぢ","지"),("づ","즈"),("で","데"),("ど","도"),
        // N
        ("な","나"),("に","니"),("ぬ","누"),("ね","네"),("の","노"),
        // H/B/P
        ("は","하"),("ひ","히"),("ふ","후"),("へ","헤"),("ほ","호"),
        ("ば","바"),("び","비"),("ぶ","부"),("べ","베"),("ぼ","보"),
        ("ぱ","파"),("ぴ","피"),("ぷ","푸"),("ぺ","페"),("ぽ","포"),
        // M
        ("ま","마"),("み","미"),("む","무"),("め","메"),("も","모"),
        // Y
        ("や","야"),("ゆ","유"),("よ","요"),
        // R
        ("ら","라"),("り","리"),("る","루"),("れ","레"),("ろ","로"),
        // W
        ("わ","와"),("ゐ","이"),("ゑ","에"),("を","오"),
    ];
    let mut map = HashMap::new();
    for (k, v) in entries {
        map.insert(*k, *v);
    }
    map
});

fn kana_sentence_to_hangul(text: &str) -> String {
    let normalized: String = text.chars().map(kata_to_hira).collect();
    let chars: Vec<char> = normalized.chars().collect();
    let n = chars.len();
    let mut out = String::new();
    let mut i = 0;
    while i < n {
        let c = chars[i];
        // Pass ASCII (spaces, punctuation, digits) as-is
        if c.is_ascii() {
            out.push(c);
            i += 1;
            continue;
        }
        // ん → 응 (nasal coda)
        if c == 'ん' {
            if let Some(last) = out.pop() {
                let cu = last as u32;
                if (0xAC00..=0xD7A3).contains(&cu) && (cu - 0xAC00) % 28 == 0 {
                    out.push(char::from_u32(cu + C_NG).unwrap_or(last));
                } else {
                    out.push(last);
                    out.push_str("응");
                }
            } else {
                out.push_str("응");
            }
            i += 1;
            continue;
        }
        // っ → geminate: attach coda to previous syllable
        if c == 'っ' {
            if let Some(last) = out.pop() {
                let cu = last as u32;
                if (0xAC00..=0xD7A3).contains(&cu) && (cu - 0xAC00) % 28 == 0 {
                    out.push(char::from_u32(cu + C_T).unwrap_or(last));
                } else {
                    out.push(last);
                }
            }
            i += 1;
            continue;
        }
        // Try 2-char combined
        if i + 1 < n {
            let two: String = chars[i..=i + 1].iter().collect();
            if let Some(h) = KANA_MAP.get(two.as_str()) {
                out.push_str(h);
                i += 2;
                continue;
            }
        }
        // Single kana
        let one: String = std::iter::once(c).collect();
        if let Some(h) = KANA_MAP.get(one.as_str()) {
            out.push_str(h);
        } else {
            out.push(c); // keep kanji, punctuation, etc.
        }
        i += 1;
    }
    out
}

// ─── Cyrillic → Korean ───────────────────────────────────────────────────────
// Uses onset/vowel syllable assembly (same approach as English).

fn cyr_is_vowel(c: char) -> bool {
    matches!(
        c,
        'а' | 'А' | 'е' | 'Е' | 'ё' | 'Ё' | 'и' | 'И' | 'й' | 'Й'
        | 'о' | 'О' | 'у' | 'У' | 'э' | 'Э' | 'ю' | 'Ю' | 'я' | 'Я'
    )
}

fn cyr_vowel_to_hangul(c: char) -> (Vowel, bool /* y-glide */) {
    match c {
        'а' | 'А' => (Vowel::A,   false),
        'е' | 'Е' => (Vowel::E,   true),  // ye
        'ё' | 'Ё' => (Vowel::Yo,  true),  // yo
        'и' | 'И' => (Vowel::I,   false),
        'й' | 'Й' => (Vowel::I,   false),
        'о' | 'О' => (Vowel::O,   false),
        'у' | 'У' => (Vowel::U,   false),
        'э' | 'Э' => (Vowel::E,   false),
        'ю' | 'Ю' => (Vowel::Yu,  true),
        'я' | 'Я' => (Vowel::Ya,  true),
        _          => (Vowel::Eu,  false),
    }
}

fn cyr_onset(c: char) -> Option<u32> {
    match c.to_lowercase().next().unwrap_or(c) {
        'г' | 'к' => Some(0),           // ㄱ
        'н'       => Some(2),            // ㄴ
        'д' | 'т' => Some(3),           // ㄷ
        'л' | 'р' => Some(5),           // ㄹ
        'м'       => Some(6),            // ㅁ
        'б' | 'в' | 'п' | 'ф' => Some(7), // ㅂ
        'с' | 'з' | 'ш' | 'щ' | 'ж' => Some(9), // ㅅ
        'ц' | 'ч' => Some(14),          // ㅊ
        'х'       => Some(18),           // ㅎ
        _         => None,
    }
}

fn cyrillic_word_to_hangul(word: &str) -> String {
    let chars: Vec<char> = word.chars().collect();
    let n = chars.len();
    let mut out = String::new();
    let mut pending: Option<u32> = None;

    let flush = |p: &mut Option<u32>, o: &mut String| {
        if let Some(onset) = p.take() {
            if !try_attach_coda(o, onset) {
                push_syllable(o, onset, Vowel::Eu, NO_CODA);
            }
        }
    };

    let mut i = 0;
    while i < n {
        let c = chars[i];
        // Soft/hard sign: skip
        if matches!(c, 'ъ' | 'ь' | 'Ъ' | 'Ь') {
            i += 1;
            continue;
        }
        if cyr_is_vowel(c) {
            let (vowel, y_glide) = cyr_vowel_to_hangul(c);
            let onset = pending.take().unwrap_or(11); // ㅇ
            if y_glide && onset == 11 {
                // y-glide with no preceding consonant: ㅇ+Ya/Ye etc.
                push_syllable(&mut out, 11, vowel, NO_CODA);
            } else {
                push_syllable(&mut out, onset, vowel, NO_CODA);
            }
        } else if let Some(cons) = cyr_onset(c) {
            flush(&mut pending, &mut out);
            if i + 1 == n {
                if !try_attach_coda(&mut out, cons) {
                    push_syllable(&mut out, cons, Vowel::Eu, NO_CODA);
                }
            } else {
                pending = Some(cons);
            }
        } else if !c.is_alphabetic() {
            flush(&mut pending, &mut out);
            out.push(c);
        }
        i += 1;
    }
    flush(&mut pending, &mut out);
    out
}

fn cyrillic_sentence_to_hangul(sentence: &str) -> String {
    let (words, sep) = split_mixed(sentence);
    words
        .into_iter()
        .zip(sep)
        .map(|(w, s)| {
            let h = cyrillic_word_to_hangul(w);
            format!("{h}{s}")
        })
        .collect()
}

// ─── Latin language detection ────────────────────────────────────────────────

#[derive(Clone, Copy, PartialEq, Debug)]
enum LatinLang { English, French, Spanish, German }

fn detect_latin_lang(sentence: &str) -> LatinLang {
    // Detect from diacritics / distinctive characters
    if sentence.chars().any(|c| matches!(c, 'ä' | 'ö' | 'ü' | 'ß')) {
        return LatinLang::German;
    }
    if sentence.chars().any(|c| matches!(c, 'ñ' | 'á' | 'í' | 'ó' | 'ú' | '¿' | '¡')) {
        return LatinLang::Spanish;
    }
    if sentence.chars().any(|c| {
        matches!(c, 'à' | 'â' | 'ç' | 'è' | 'é' | 'ê' | 'ë'
                  | 'î' | 'ï' | 'ô' | 'ù' | 'û' | 'œ' | 'æ')
    }) {
        return LatinLang::French;
    }
    LatinLang::English
}

// ─── English (existing rules, preserved) ────────────────────────────────────

fn onset_for_en(c: char) -> Option<u32> {
    Some(match c {
        'g' | 'G' => 0,
        'n' | 'N' => 2,
        'd' | 'D' => 3,
        'r' | 'R' | 'l' | 'L' => 5,
        'm' | 'M' => 6,
        'b' | 'B' | 'v' | 'V' => 7,
        's' | 'S' | 'z' | 'Z' => 9,
        'j' | 'J' => 12,
        'c' | 'C' => 14,
        'k' | 'K' | 'q' | 'Q' => 15,
        't' | 'T' => 16,
        'p' | 'P' | 'f' | 'F' => 17,
        'h' | 'H' => 18,
        'w' | 'W' | 'y' | 'Y' => 11,
        'x' | 'X' => 15,
        _ => return None,
    })
}

fn rule_transliterate_en(word: &str) -> String {
    let word = word.to_ascii_lowercase();
    let bytes = word.as_bytes();
    let n = bytes.len();
    if n == 0 {
        return String::new();
    }

    let mut out = String::new();
    let mut i = 0;
    let mut pending: Option<u32> = None;

    let ch = |i: usize| -> Option<char> { bytes.get(i).map(|b| *b as char) };
    let is_at_vowel = |i: usize| ch(i).map(is_vowel_char).unwrap_or(false);

    while i < n {
        let slice2: &str = std::str::from_utf8(&bytes[i..(i + 2).min(n)]).unwrap_or("");
        let slice3: &str = std::str::from_utf8(&bytes[i..(i + 3).min(n)]).unwrap_or("");

        let flush_pending = |pending: &mut Option<u32>, out: &mut String| {
            if let Some(p) = pending.take() {
                if !try_attach_coda(out, p) {
                    push_syllable(out, p, Vowel::Eu, NO_CODA);
                }
            }
        };

        // Trigraphs
        if slice3 == "tch" {
            flush_pending(&mut pending, &mut out);
            pending = Some(14);
            i += 3;
            continue;
        }
        if slice3 == "igh" || slice3 == "eye" {
            flush_pending(&mut pending, &mut out);
            push_syllable(&mut out, 11, Vowel::A, NO_CODA);
            push_syllable(&mut out, 11, Vowel::I, NO_CODA);
            i += 3;
            continue;
        }

        // Word-initial unstressed "re-" prefix (e.g., receive/respect/return) → 리
        if i == 0 && slice2 == "re" && n > 3 {
            push_syllable(&mut out, 5, Vowel::I, NO_CODA);
            pending = None;
            i += 2;
            continue;
        }

        // "we" + (end or apostrophe) → /wi:/ = 위 (covers "we", "we've", "we'll", "we're")
        if slice2 == "we"
            && (i + 2 == n || bytes.get(i + 2) == Some(&b'\''))
        {
            let o = pending.take().unwrap_or(11);
            push_syllable(&mut out, o, Vowel::Wi, NO_CODA);
            i += 2;
            continue;
        }

        // "-ed" past-tense suffix at end of word (after consonant) → silent e + 드
        if i + 2 == n
            && bytes[i] == b'e'
            && bytes[i + 1] == b'd'
            && i > 0
            && !is_vowel_char(bytes[i - 1] as char)
        {
            if let Some(p) = pending.take() {
                push_syllable(&mut out, p, Vowel::Eu, NO_CODA);
            }
            push_syllable(&mut out, 3, Vowel::Eu, NO_CODA); // 드
            i += 2;
            continue;
        }

        // Vowel digraphs
        match slice2 {
            "er" | "ir" | "ur" => {
                let o = pending.take().unwrap_or(11);
                push_syllable(&mut out, o, Vowel::Eo, NO_CODA);
                i += 2;
                continue;
            }
            "ar" => {
                let o = pending.take().unwrap_or(11);
                push_syllable(&mut out, o, Vowel::A, NO_CODA);
                i += 2;
                continue;
            }
            "or" => {
                let o = pending.take().unwrap_or(11);
                push_syllable(&mut out, o, Vowel::O, NO_CODA);
                i += 2;
                continue;
            }
            "oo" => {
                let o = pending.take().unwrap_or(11);
                push_syllable(&mut out, o, Vowel::U, NO_CODA);
                i += 2;
                continue;
            }
            "ee" | "ea" | "ei" => {
                // English "ei" (receive, ceiling) is most often long /i:/ → 이
                let o = pending.take().unwrap_or(11);
                push_syllable(&mut out, o, Vowel::I, NO_CODA);
                i += 2;
                continue;
            }
            "ai" | "ay" => {
                let o = pending.take().unwrap_or(11);
                push_syllable(&mut out, o, Vowel::E, NO_CODA);
                push_syllable(&mut out, 11, Vowel::I, NO_CODA);
                i += 2;
                continue;
            }
            "ou" | "ow" => {
                let o = pending.take().unwrap_or(11);
                push_syllable(&mut out, o, Vowel::A, NO_CODA);
                push_syllable(&mut out, 11, Vowel::U, NO_CODA);
                i += 2;
                continue;
            }
            "oi" | "oy" => {
                let o = pending.take().unwrap_or(11);
                push_syllable(&mut out, o, Vowel::O, NO_CODA);
                push_syllable(&mut out, 11, Vowel::I, NO_CODA);
                i += 2;
                continue;
            }
            "au" | "aw" => {
                let o = pending.take().unwrap_or(11);
                push_syllable(&mut out, o, Vowel::O, NO_CODA);
                i += 2;
                continue;
            }
            "ie" => {
                let o = pending.take().unwrap_or(11);
                push_syllable(&mut out, o, Vowel::I, NO_CODA);
                i += 2;
                continue;
            }
            "ew" => {
                let o = pending.take().unwrap_or(11);
                push_syllable(&mut out, o, Vowel::Yu, NO_CODA);
                i += 2;
                continue;
            }
            "ue" => {
                let o = pending.take().unwrap_or(11);
                push_syllable(&mut out, o, Vowel::U, NO_CODA);
                i += 2;
                continue;
            }
            _ => {}
        }

        // Consonant digraphs
        match slice2 {
            "th" => {
                flush_pending(&mut pending, &mut out);
                pending = Some(9);
                i += 2;
                continue;
            }
            "sh" => {
                flush_pending(&mut pending, &mut out);
                push_syllable(&mut out, 9, Vowel::Wi, NO_CODA);
                i += 2;
                continue;
            }
            "ch" => {
                flush_pending(&mut pending, &mut out);
                pending = Some(14);
                i += 2;
                continue;
            }
            "ph" => {
                flush_pending(&mut pending, &mut out);
                pending = Some(17);
                i += 2;
                continue;
            }
            "gh" => {
                i += 2;
                continue;
            }
            "ck" => {
                flush_pending(&mut pending, &mut out);
                if let Some(last) = out.pop() {
                    let c = last as u32;
                    if (0xAC00..=0xD7A3).contains(&c) {
                        let rel = c - 0xAC00;
                        if rel % 28 == 0 {
                            let new = c + C_K;
                            out.push(char::from_u32(new).unwrap_or(last));
                            i += 2;
                            continue;
                        }
                    }
                    out.push(last);
                }
                pending = Some(15);
                i += 2;
                continue;
            }
            "qu" => {
                flush_pending(&mut pending, &mut out);
                push_syllable(&mut out, 15, Vowel::U, NO_CODA);
                i += 2;
                continue;
            }
            "wh" => {
                flush_pending(&mut pending, &mut out);
                i += 1;
                continue;
            }
            "ng" => {
                flush_pending(&mut pending, &mut out);
                if let Some(last) = out.pop() {
                    let c = last as u32;
                    if (0xAC00..=0xD7A3).contains(&c) {
                        let rel = c - 0xAC00;
                        if rel % 28 == 0 {
                            let new = c + C_NG;
                            out.push(char::from_u32(new).unwrap_or(last));
                            i += 2;
                            continue;
                        }
                    }
                    out.push(last);
                }
                push_syllable(&mut out, 11, Vowel::Eu, C_NG);
                i += 2;
                continue;
            }
            _ => {}
        }

        let c = bytes[i] as char;
        if is_vowel_char(c) {
            let vowel = match c {
                'a' => {
                    // Magic-e: "ate" pattern at end → long /eɪ/ → 에이
                    if i + 2 == n && bytes[i + 1] as char == 'e' {
                        let o = pending.take().unwrap_or(11);
                        push_syllable(&mut out, o, Vowel::E, NO_CODA);
                        push_syllable(&mut out, 11, Vowel::I, NO_CODA);
                        i += 2;
                        continue;
                    }
                    // Open syllable: 'a' + single consonant + vowel → long /eɪ/
                    // (catering, lady, baby, paper, table)
                    if i + 2 < n
                        && !is_vowel_char(bytes[i + 1] as char)
                        && bytes[i + 1] != b'r'
                        && is_vowel_char(bytes[i + 2] as char)
                    {
                        let o = pending.take().unwrap_or(11);
                        push_syllable(&mut out, o, Vowel::E, NO_CODA);
                        push_syllable(&mut out, 11, Vowel::I, NO_CODA);
                        i += 1;
                        continue;
                    }
                    Vowel::A
                }
                'e' => {
                    if i + 1 == n {
                        if let Some(p) = pending.take() {
                            push_syllable(&mut out, p, Vowel::Eu, NO_CODA);
                        }
                        i += 1;
                        continue;
                    }
                    Vowel::E
                }
                'i' => {
                    if i + 2 < n && !is_at_vowel(i + 1) && bytes[i + 2] as char == 'e' {
                        let o = pending.take().unwrap_or(11);
                        push_syllable(&mut out, o, Vowel::A, NO_CODA);
                        push_syllable(&mut out, 11, Vowel::I, NO_CODA);
                        i += 1;
                        continue;
                    }
                    Vowel::I
                }
                'o' => Vowel::O,
                'u' => Vowel::Eo,
                'y' => {
                    if i == 0 && i + 1 < n && is_at_vowel(i + 1) {
                        i += 1;
                        continue;
                    }
                    Vowel::I
                }
                _ => Vowel::Eu,
            };
            let onset = pending.take().unwrap_or(11);
            push_syllable(&mut out, onset, vowel, NO_CODA);
            i += 1;
            continue;
        }

        if let Some(cons) = onset_for_en(c) {
            // Context-sensitive: 'c' and 'g' sound depends on following vowel
            let cons = match c {
                'c' => if ch(i + 1).map(|n| matches!(n, 'e' | 'i' | 'y')).unwrap_or(false) { 9 } else { 15 },
                'g' => if ch(i + 1).map(|n| matches!(n, 'e' | 'i')).unwrap_or(false) { 12 } else { 0 },
                _ => cons,
            };
            flush_pending(&mut pending, &mut out);
            if i + 1 == n {
                if let Some(last) = out.pop() {
                    let cu = last as u32;
                    if (0xAC00..=0xD7A3).contains(&cu) {
                        let rel = cu - 0xAC00;
                        if rel % 28 == 0 {
                            let coda = match c {
                                'n' => Some(C_N), 'm' => Some(C_M),
                                'l' | 'r' => Some(C_L), 'k' | 'g' => Some(C_K),
                                't' | 'd' | 's' | 'z' => Some(C_T),
                                'p' | 'b' => Some(C_P), _ => None,
                            };
                            if let Some(code) = coda {
                                let new = cu + code;
                                out.push(char::from_u32(new).unwrap_or(last));
                                i += 1;
                                continue;
                            }
                        }
                    }
                    out.push(last);
                }
                push_syllable(&mut out, cons, Vowel::Eu, NO_CODA);
                i += 1;
                continue;
            }
            pending = Some(cons);
            i += 1;
            continue;
        } else {
            i += 1;
        }
    }

    if let Some(p) = pending.take() {
        push_syllable(&mut out, p, Vowel::Eu, NO_CODA);
    }
    out
}

// ─── French phonetic rules ───────────────────────────────────────────────────

fn onset_for_latin(c: char) -> Option<u32> {
    Some(match c.to_ascii_lowercase() {
        'g' => 0, 'n' => 2, 'd' => 3, 'r' | 'l' => 5, 'm' => 6,
        'b' | 'v' => 7, 's' | 'z' => 9, 'j' => 12,
        'c' => 14, 'k' | 'q' => 15, 't' => 16, 'p' | 'f' => 17, 'h' => 18,
        'w' | 'y' => 11,
        _ => return None,
    })
}

fn rule_transliterate_fr(word: &str) -> String {
    // Normalize accented Latin → base letters for rule processing
    let word_lc: String = word.chars().map(|c| match c {
        'À' | 'Â' => 'a', 'à' | 'â' => 'a',
        'È' | 'É' | 'Ê' | 'Ë' => 'e', 'è' | 'é' | 'ê' | 'ë' => 'e',
        'Î' | 'Ï' => 'i', 'î' | 'ï' => 'i',
        'Ô' => 'o', 'ô' => 'o',
        'Ù' | 'Û' | 'Ü' => 'u', 'ù' | 'û' | 'ü' => 'u',
        'Ç' => 's', 'ç' => 's',
        'Œ' | 'œ' => 'e', 'Æ' | 'æ' => 'a',
        other => other.to_ascii_lowercase(),
    }).collect();

    let chars: Vec<char> = word_lc.chars().collect();
    let n = chars.len();
    let mut out = String::new();
    let mut pending: Option<u32> = None;
    let mut i = 0;

    let flush = |p: &mut Option<u32>, o: &mut String| {
        if let Some(onset) = p.take() {
            if !try_attach_coda(o, onset) {
                push_syllable(o, onset, Vowel::Eu, NO_CODA);
            }
        }
    };

    let peek = |j: usize| -> Option<char> { chars.get(j).copied() };
    // After position j, is there a consonant or end of word? (for nasal detection)
    let before_cons_or_end = |j: usize| -> bool {
        peek(j).map(|c| !is_vowel_char(c)).unwrap_or(true)
    };
    let slice_at = |j: usize, len: usize| -> String {
        chars[j..n.min(j + len)].iter().collect()
    };

    while i < n {
        let s4 = slice_at(i, 4);
        let s3 = slice_at(i, 3);
        let s2 = slice_at(i, 2);
        let c = chars[i];

        // -tion → 시옹
        if s4 == "tion" {
            flush(&mut pending, &mut out);
            push_syllable(&mut out, 9,  Vowel::I, NO_CODA);
            push_syllable(&mut out, 11, Vowel::O, C_NG);
            i += 4;
            continue;
        }
        // eau → 오
        if s3 == "eau" {
            let o = pending.take().unwrap_or(11);
            push_syllable(&mut out, o, Vowel::O, NO_CODA);
            i += 3;
            continue;
        }
        // gn → 뉴
        if s2 == "gn" {
            flush(&mut pending, &mut out);
            push_syllable(&mut out, 2, Vowel::Yu, NO_CODA);
            i += 2;
            continue;
        }
        // ch → 슈
        if s2 == "ch" {
            flush(&mut pending, &mut out);
            push_syllable(&mut out, 9, Vowel::Wi, NO_CODA);
            i += 2;
            continue;
        }
        // ph → ㅍ
        if s2 == "ph" {
            flush(&mut pending, &mut out);
            pending = Some(17);
            i += 2;
            continue;
        }
        // ou → 우
        if s2 == "ou" {
            let o = pending.take().unwrap_or(11);
            push_syllable(&mut out, o, Vowel::U, NO_CODA);
            i += 2;
            continue;
        }
        // au → 오
        if s2 == "au" {
            let o = pending.take().unwrap_or(11);
            push_syllable(&mut out, o, Vowel::O, NO_CODA);
            i += 2;
            continue;
        }
        // oi → 와
        if s2 == "oi" {
            let o = pending.take().unwrap_or(11);
            push_syllable(&mut out, o, Vowel::Wa, NO_CODA);
            i += 2;
            continue;
        }
        // ai / ei → 에
        if s2 == "ai" || s2 == "ei" {
            let o = pending.take().unwrap_or(11);
            push_syllable(&mut out, o, Vowel::E, NO_CODA);
            i += 2;
            continue;
        }
        // eu → 으
        if s2 == "eu" {
            let o = pending.take().unwrap_or(11);
            push_syllable(&mut out, o, Vowel::Eu, NO_CODA);
            i += 2;
            continue;
        }
        // Nasal vowels: an/am/en/em → 앙
        if (s2 == "an" || s2 == "am" || s2 == "en" || s2 == "em") && before_cons_or_end(i + 2) {
            let o = pending.take().unwrap_or(11);
            push_syllable(&mut out, o, Vowel::A, C_NG);
            i += 2;
            continue;
        }
        // in/im → 앵
        if (s2 == "in" || s2 == "im") && before_cons_or_end(i + 2) {
            let o = pending.take().unwrap_or(11);
            push_syllable(&mut out, o, Vowel::E, C_NG);
            i += 2;
            continue;
        }
        // on/om → 옹
        if (s2 == "on" || s2 == "om") && before_cons_or_end(i + 2) {
            let o = pending.take().unwrap_or(11);
            push_syllable(&mut out, o, Vowel::O, C_NG);
            i += 2;
            continue;
        }
        // un/um → 앙
        if (s2 == "un" || s2 == "um") && before_cons_or_end(i + 2) {
            let o = pending.take().unwrap_or(11);
            push_syllable(&mut out, o, Vowel::A, C_NG);
            i += 2;
            continue;
        }

        // Silent final consonants: d/t/s/x/z/p at word end
        if i + 1 == n && matches!(c, 'd' | 't' | 's' | 'x' | 'z' | 'p') {
            flush(&mut pending, &mut out);
            i += 1;
            continue;
        }

        // r → ㅎ (French uvular r approximated)
        if c == 'r' {
            flush(&mut pending, &mut out);
            pending = Some(18);
            i += 1;
            continue;
        }

        // Vowels
        if is_vowel_char(c) {
            let vowel = match c {
                'a' => Vowel::A, 'e' => { if i + 1 == n { flush(&mut pending, &mut out); i += 1; continue; } Vowel::E },
                'i' => Vowel::I, 'o' => Vowel::O, 'u' => Vowel::U,
                'y' => Vowel::I, _ => Vowel::Eu,
            };
            let o = pending.take().unwrap_or(11);
            push_syllable(&mut out, o, vowel, NO_CODA);
        } else if let Some(cons) = onset_for_latin(c) {
            flush(&mut pending, &mut out);
            if i + 1 == n {
                if !try_attach_coda(&mut out, cons) {
                    push_syllable(&mut out, cons, Vowel::Eu, NO_CODA);
                }
            } else {
                pending = Some(cons);
            }
        }
        i += 1;
    }
    flush(&mut pending, &mut out);
    out
}

// ─── Spanish phonetic rules ──────────────────────────────────────────────────

fn rule_transliterate_es(word: &str) -> String {
    let word_lc: String = word.chars().map(|c| match c {
        'Á' => 'a', 'á' => 'a',
        'É' => 'e', 'é' => 'e',
        'Í' => 'i', 'í' => 'i',
        'Ó' => 'o', 'ó' => 'o',
        'Ú' | 'Ü' => 'u', 'ú' | 'ü' => 'u',
        other => other.to_ascii_lowercase(),
    }).collect();

    let chars: Vec<char> = word_lc.chars().collect();
    let n = chars.len();
    let mut out = String::new();
    let mut pending: Option<u32> = None;
    let mut i = 0;

    let flush = |p: &mut Option<u32>, o: &mut String| {
        if let Some(onset) = p.take() {
            if !try_attach_coda(o, onset) {
                push_syllable(o, onset, Vowel::Eu, NO_CODA);
            }
        }
    };

    let peek = |j: usize| -> Option<char> { chars.get(j).copied() };
    let slice_at = |j: usize, len: usize| -> String {
        chars[j..n.min(j + len)].iter().collect()
    };

    while i < n {
        let s2 = slice_at(i, 2);
        let c = chars[i];

        // ñ → 뉴
        if c == 'ñ' || c == 'Ñ' {
            flush(&mut pending, &mut out);
            push_syllable(&mut out, 2, Vowel::Yu, NO_CODA);
            i += 1;
            continue;
        }
        // ll → 이
        if s2 == "ll" {
            flush(&mut pending, &mut out);
            push_syllable(&mut out, 11, Vowel::I, NO_CODA);
            i += 2;
            continue;
        }
        // rr → same as r
        if s2 == "rr" {
            flush(&mut pending, &mut out);
            pending = Some(5);
            i += 2;
            continue;
        }
        // ch → 치
        if s2 == "ch" {
            flush(&mut pending, &mut out);
            pending = Some(14);
            i += 2;
            continue;
        }
        // qu + e/i → ㅋ (silent u)
        if s2 == "qu" {
            flush(&mut pending, &mut out);
            pending = Some(15);
            i += 2; // skip u
            continue;
        }
        // gu + e/i → ㄱ (silent u)
        if s2 == "gu" && peek(i + 2).map(|x| matches!(x, 'e' | 'i')).unwrap_or(false) {
            flush(&mut pending, &mut out);
            pending = Some(0);
            i += 2;
            continue;
        }
        // h is silent
        if c == 'h' {
            i += 1;
            continue;
        }
        // j / g+e/i → ㅎ (Spanish jota)
        if c == 'j'
            || (c == 'g' && peek(i + 1).map(|x| matches!(x, 'e' | 'i')).unwrap_or(false))
        {
            flush(&mut pending, &mut out);
            pending = Some(18); // ㅎ
            i += 1;
            continue;
        }
        // v → same as b
        if c == 'v' {
            flush(&mut pending, &mut out);
            pending = Some(7);
            i += 1;
            continue;
        }
        // z / c+e/i → 스
        if c == 'z'
            || (c == 'c' && peek(i + 1).map(|x| matches!(x, 'e' | 'i')).unwrap_or(false))
        {
            flush(&mut pending, &mut out);
            pending = Some(9);
            i += 1;
            continue;
        }

        if is_vowel_char(c) {
            let vowel = match c {
                'a' => Vowel::A, 'e' => Vowel::E,
                'i' | 'y' => Vowel::I, 'o' => Vowel::O, 'u' => Vowel::U,
                _ => Vowel::Eu,
            };
            let o = pending.take().unwrap_or(11);
            push_syllable(&mut out, o, vowel, NO_CODA);
        } else if let Some(cons) = onset_for_latin(c) {
            flush(&mut pending, &mut out);
            if i + 1 == n {
                if !try_attach_coda(&mut out, cons) {
                    push_syllable(&mut out, cons, Vowel::Eu, NO_CODA);
                }
            } else {
                pending = Some(cons);
            }
        }
        i += 1;
    }
    flush(&mut pending, &mut out);
    out
}

// ─── German phonetic rules ───────────────────────────────────────────────────

fn rule_transliterate_de(word: &str) -> String {
    let chars: Vec<char> = word.chars().collect();
    let n = chars.len();
    let mut out = String::new();
    let mut pending: Option<u32> = None;
    let mut i = 0;

    let flush = |p: &mut Option<u32>, o: &mut String| {
        if let Some(onset) = p.take() {
            if !try_attach_coda(o, onset) {
                push_syllable(o, onset, Vowel::Eu, NO_CODA);
            }
        }
    };

    let peek = |j: usize| -> Option<char> { chars.get(j).copied() };
    let slice_lc = |j: usize, len: usize| -> String {
        chars[j..n.min(j + len)].iter().map(|c| c.to_ascii_lowercase()).collect()
    };

    while i < n {
        let c = chars[i].to_ascii_lowercase();
        let s3 = slice_lc(i, 3);
        let s2 = slice_lc(i, 2);

        // sch → 슈
        if s3 == "sch" {
            flush(&mut pending, &mut out);
            push_syllable(&mut out, 9, Vowel::Wi, NO_CODA);
            i += 3;
            continue;
        }
        // ch after front vowel → 히; otherwise → 흐
        if s2 == "ch" {
            flush(&mut pending, &mut out);
            let prev_front = out.chars().last().map(|lc| {
                let cp = lc as u32;
                if (0xAC00..=0xD7A3).contains(&cp) {
                    let v = ((cp - 0xAC00) / 28) % 21;
                    matches!(v, 5 | 20) // ㅔ or ㅣ
                } else {
                    false
                }
            }).unwrap_or(false);
            if prev_front {
                push_syllable(&mut out, 18, Vowel::I, NO_CODA); // 히
            } else {
                push_syllable(&mut out, 18, Vowel::Eu, NO_CODA); // 흐
            }
            i += 2;
            continue;
        }
        // qu → 크
        if s2 == "qu" {
            flush(&mut pending, &mut out);
            pending = Some(15);
            i += 2;
            continue;
        }
        // st/sp at word start → 슈트/슈프
        if (s2 == "st" || s2 == "sp") && i == 0 {
            push_syllable(&mut out, 9, Vowel::Wi, NO_CODA); // 슈
            pending = Some(if c == 's' && peek(i + 1) == Some('t') { 16 } else { 17 });
            i += 2;
            continue;
        }
        // ei → 아이
        if s2 == "ei" {
            let o = pending.take().unwrap_or(11);
            push_syllable(&mut out, o, Vowel::A, NO_CODA);
            push_syllable(&mut out, 11, Vowel::I, NO_CODA);
            i += 2;
            continue;
        }
        // ie → 이
        if s2 == "ie" {
            let o = pending.take().unwrap_or(11);
            push_syllable(&mut out, o, Vowel::I, NO_CODA);
            i += 2;
            continue;
        }
        // eu / äu → 오이
        if s2 == "eu" || s2 == "äu" {
            let o = pending.take().unwrap_or(11);
            push_syllable(&mut out, o, Vowel::O, NO_CODA);
            push_syllable(&mut out, 11, Vowel::I, NO_CODA);
            i += 2;
            continue;
        }
        // au → 아우
        if s2 == "au" {
            let o = pending.take().unwrap_or(11);
            push_syllable(&mut out, o, Vowel::A, NO_CODA);
            push_syllable(&mut out, 11, Vowel::U, NO_CODA);
            i += 2;
            continue;
        }
        // ß → 스
        if chars[i] == 'ß' {
            flush(&mut pending, &mut out);
            push_syllable(&mut out, 9, Vowel::Eu, NO_CODA);
            i += 1;
            continue;
        }

        // Umlauts
        let c_orig = chars[i];
        if matches!(c_orig, 'ä' | 'Ä') {
            let o = pending.take().unwrap_or(11);
            push_syllable(&mut out, o, Vowel::E, NO_CODA);
            i += 1;
            continue;
        }
        if matches!(c_orig, 'ö' | 'Ö') {
            let o = pending.take().unwrap_or(11);
            push_syllable(&mut out, o, Vowel::Oe, NO_CODA);
            i += 1;
            continue;
        }
        if matches!(c_orig, 'ü' | 'Ü') {
            let o = pending.take().unwrap_or(11);
            push_syllable(&mut out, o, Vowel::Wi, NO_CODA);
            i += 1;
            continue;
        }

        // w → ㅂ
        if c == 'w' {
            flush(&mut pending, &mut out);
            pending = Some(7);
            i += 1;
            continue;
        }
        // v → ㅍ
        if c == 'v' {
            flush(&mut pending, &mut out);
            pending = Some(17);
            i += 1;
            continue;
        }
        // z → 츠
        if c == 'z' {
            flush(&mut pending, &mut out);
            push_syllable(&mut out, 14, Vowel::Eu, NO_CODA); // 츠
            i += 1;
            continue;
        }
        // -ig at word end → 이히
        if c == 'i' && peek(i + 1) == Some('g') && i + 2 == n {
            let o = pending.take().unwrap_or(11);
            push_syllable(&mut out, o, Vowel::I, NO_CODA);
            push_syllable(&mut out, 18, Vowel::I, NO_CODA); // 히
            i += 2;
            continue;
        }

        if is_vowel_char(c) {
            let vowel = match c {
                'a' => Vowel::A, 'e' => Vowel::E,
                'i' | 'y' => Vowel::I, 'o' => Vowel::O, 'u' => Vowel::U,
                _ => Vowel::Eu,
            };
            let o = pending.take().unwrap_or(11);
            push_syllable(&mut out, o, vowel, NO_CODA);
        } else if let Some(cons) = onset_for_latin(c) {
            flush(&mut pending, &mut out);
            if i + 1 == n {
                if !try_attach_coda(&mut out, cons) {
                    push_syllable(&mut out, cons, Vowel::Eu, NO_CODA);
                }
            } else {
                pending = Some(cons);
            }
        }
        i += 1;
    }
    flush(&mut pending, &mut out);
    out
}

// ─── English dictionary (preserved) ─────────────────────────────────────────

static DICT: Lazy<HashMap<&'static str, &'static str>> = Lazy::new(|| {
    #[rustfmt::skip]
    let entries: &[(&str, &str)] = &[
        // NOTE: "a" intentionally omitted — text "a" maps to different sounds
        // (article /ə/ vs. letter /eɪ/). Rule-based + letter_name_ko (for uppercase)
        // handles both cases sound-correctly.
        ("the","더"),("an","언"),("and","앤드"),("or","오어"),
        ("but","벗"),("is","이즈"),("are","아"),("am","앰"),("was","워즈"),
        ("were","워"),("be","비"),("been","빈"),("being","비잉"),
        ("do","두"),("does","더즈"),("did","디드"),("done","던"),
        ("have","해브"),("has","해즈"),("had","해드"),
        ("will","윌"),("would","우드"),("can","캔"),("could","쿠드"),
        ("should","슈드"),("shall","쉘"),("may","메이"),("might","마이트"),("must","머스트"),
        ("i","아이"),("you","유"),("he","히"),("she","쉬"),("it","잇"),
        ("we","위"),("they","데이"),("me","미"),("him","힘"),("her","허"),
        ("us","어스"),("them","뎀"),
        ("my","마이"),("your","유어"),("his","히즈"),("its","잇츠"),
        ("our","아워"),("their","데어"),
        ("this","디스"),("that","댓"),("these","디즈"),("those","도우즈"),
        ("here","히어"),("there","데어"),("where","웨어"),
        ("what","왓"),("when","웬"),("why","와이"),("how","하우"),
        ("who","후"),("which","위치"),
        ("hello","헬로"),("hi","하이"),("hey","헤이"),
        ("bye","바이"),("goodbye","굿바이"),
        ("yes","예스"),("no","노"),("ok","오케이"),("okay","오케이"),
        ("thanks","땡스"),("thank","땡크"),("please","플리즈"),
        ("sorry","쏘리"),("welcome","웰컴"),
        ("love","러브"),("like","라이크"),("want","원트"),("need","니드"),
        ("know","노우"),("think","씽크"),("see","씨"),("look","룩"),
        ("go","고"),("come","컴"),("get","겟"),("make","메이크"),
        ("take","테이크"),("give","기브"),("find","파인드"),
        ("use","유즈"),("work","워크"),("call","콜"),("try","트라이"),
        ("ask","애스크"),("help","헬프"),("say","세이"),("tell","텔"),
        ("talk","토크"),("speak","스피크"),("read","리드"),("write","라이트"),
        ("play","플레이"),("run","런"),("walk","워크"),("drive","드라이브"),
        ("eat","잇"),("drink","드링크"),("sleep","슬립"),
        ("today","투데이"),("tomorrow","투마로우"),("yesterday","예스터데이"),
        ("now","나우"),("then","덴"),("soon","순"),
        ("good","굿"),("bad","배드"),("new","뉴"),("old","올드"),
        ("big","빅"),("small","스몰"),("high","하이"),("low","로우"),
        ("fast","패스트"),("slow","슬로우"),("hot","핫"),("cold","콜드"),
        ("first","퍼스트"),("last","라스트"),("next","넥스트"),
        ("one","원"),("two","투"),("three","쓰리"),("four","포"),
        ("five","파이브"),("six","식스"),("seven","세븐"),
        ("eight","에잇"),("nine","나인"),("ten","텐"),
        ("of","오브"),("in","인"),("on","온"),("at","앳"),
        ("to","투"),("for","포"),("from","프롬"),("with","위드"),
        ("without","위다웃"),("about","어바웃"),("as","애즈"),("so","쏘"),
        ("if","이프"),("because","비코즈"),
        ("very","베리"),("just","저스트"),("only","온리"),("also","올쏘"),
        ("more","모어"),("most","모스트"),("some","썸"),("any","애니"),
        ("all","올"),("every","에브리"),("each","이치"),
        ("many","매니"),("much","머치"),("few","퓨"),("little","리틀"),
        ("people","피플"),("person","퍼슨"),("man","맨"),("woman","우먼"),
        ("child","차일드"),("boy","보이"),("girl","걸"),
        ("friend","프렌드"),("family","패밀리"),
        ("home","홈"),("house","하우스"),("school","스쿨"),
        ("time","타임"),("day","데이"),("night","나이트"),
        ("year","이어"),("month","먼스"),("week","위크"),
        ("hour","아워"),("minute","미닛"),
        ("world","월드"),("country","컨트리"),("city","시티"),
        ("street","스트릿"),("car","카"),("bus","버스"),("train","트레인"),
        ("book","북"),("phone","폰"),("computer","컴퓨터"),
        ("music","뮤직"),("movie","무비"),("video","비디오"),("audio","오디오"),
        ("english","잉글리시"),("korean","코리안"),
        ("app","앱"),("apps","앱스"),("sounds","사운즈"),("sound","사운드"),
        ("start","스타트"),("started","스타티드"),("starting","스타팅"),("stop","스톱"),
        ("hello,","헬로,"),
        ("number","넘버"),("numbers","넘버즈"),("zero","지로"),
        ("hundred","헌드레드"),("thousand","따우전드"),("million","밀리언"),
        ("coming","커밍"),("going","고잉"),("doing","두잉"),("having","해빙"),
        ("getting","게팅"),("making","메이킹"),("looking","루킹"),
        ("taking","테이킹"),("saying","세이잉"),("working","워킹"),
        ("knew","뉴"),("fix","픽스"),("fixed","픽스트"),("fixing","픽싱"),
        ("lights","라이츠"),("light","라이트"),("came","케임"),
        ("isn't","이즌트"),("aren't","아른트"),("wasn't","워즌트"),
        ("weren't","워른트"),("don't","돈트"),("doesn't","더즌트"),
        ("didn't","디든트"),("won't","원트"),("wouldn't","우든트"),
        ("can't","캔트"),("couldn't","쿠든트"),("shouldn't","슈든트"),
        ("haven't","해븐트"),("hasn't","해즌트"),("hadn't","해든트"),
        ("i'm","아임"),("i've","아이브"),("i'll","아일"),("i'd","아이드"),
        ("you're","유어"),("you've","유브"),("you'll","유일"),
        ("he's","히즈"),("she's","쉬즈"),("it's","잇츠"),("we're","위어"),
        ("they're","데어"),("that's","댓츠"),("there's","데어즈"),
        ("here's","히어즈"),("what's","왓츠"),("let's","렛츠"),
        ("electrician","일렉트리션"),("electricity","일렉트리시티"),
        ("electric","일렉트릭"),("election","일렉션"),("elections","일렉션즈"),
        ("october","옥토버"),("january","재뉴어리"),("february","페브루어리"),
        ("march","마치"),("april","에이프릴"),("june","준"),("july","줄라이"),
        ("august","오거스트"),("september","셉템버"),("november","노벰버"),
        ("december","디셈버"),
        ("monday","먼데이"),("tuesday","튜즈데이"),("wednesday","웬즈데이"),
        ("thursday","써즈데이"),("friday","프라이데이"),
        ("saturday","새터데이"),("sunday","선데이"),
        ("morning","모닝"),("afternoon","애프터눈"),("evening","이브닝"),
        ("water","워터"),("money","머니"),("food","푸드"),
        ("door","도어"),("window","윈도우"),("floor","플로어"),
        ("room","룸"),("kitchen","키친"),("bathroom","배쓰룸"),
        ("office","오피스"),("store","스토어"),("market","마켓"),
        ("hospital","하스피털"),("doctor","닥터"),("teacher","티처"),
        ("student","스튜던트"),
        ("mother","마더"),("father","파더"),("brother","브라더"),
        ("sister","시스터"),("daughter","도터"),("son","선"),
        ("husband","허즈번드"),("wife","와이프"),("baby","베이비"),
        ("animal","애니멀"),("coffee","커피"),("tea","티"),
        ("pizza","피자"),("burger","버거"),
        ("happy","해피"),("sad","새드"),("angry","앵그리"),
        ("tired","타이어드"),("hungry","헝그리"),("thirsty","써스티"),
        ("beautiful","뷰티풀"),("pretty","프리티"),("handsome","핸섬"),
        ("smart","스마트"),("funny","퍼니"),("important","임포턴트"),
        ("different","디퍼런트"),("other","어더"),("same","세임"),
        ("right","라이트"),("wrong","롱"),("long","롱"),("short","쇼트"),
        ("easy","이지"),("hard","하드"),("difficult","디피컬트"),
        ("possible","파시블"),("ready","레디"),("sure","슈어"),
        ("true","트루"),("false","폴스"),("free","프리"),
        ("open","오픈"),("close","클로즈"),("closed","클로즈드"),
        ("before","비포"),("after","애프터"),("during","듀링"),
        ("through","쓰루"),("between","비트윈"),("behind","비하인드"),
        ("under","언더"),("over","오버"),("around","어라운드"),
        ("against","어겐스트"),("until","언틸"),("while","와일"),
        ("although","올도우"),("however","하우에버"),("therefore","데어포"),
        ("whether","웨더"),("actually","액추얼리"),("usually","유주얼리"),
        ("really","릴리"),("already","올레디"),("still","스틸"),
        ("again","어겐"),("never","네버"),("always","올웨이즈"),
        ("sometimes","썸타임즈"),("often","오픈"),("ever","에버"),
        ("maybe","메이비"),("probably","프라버블리"),("almost","올모스트"),
        ("enough","이너프"),("together","투게더"),
        ("everything","에브리씽"),("something","썸씽"),
        ("nothing","낫씽"),("anything","애니씽"),
        ("someone","썸원"),("everyone","에브리원"),
        ("nobody","노바디"),("anybody","애니바디"),
        ("somewhere","썸웨어"),("everywhere","에브리웨어"),
        ("nowhere","노웨어"),("anyway","애니웨이"),
    ];
    let mut map = HashMap::new();
    for (k, v) in entries {
        map.insert(*k, *v);
    }
    map
});

// ─── English alphabet letter names ──────────────────────────────────────────

fn letter_name_ko(c: char) -> Option<&'static str> {
    match c.to_ascii_uppercase() {
        'A' => Some("에이"), 'B' => Some("비"),  'C' => Some("씨"),
        'D' => Some("디"),   'E' => Some("이"),   'F' => Some("에프"),
        'G' => Some("지"),   'H' => Some("에이치"),'I' => Some("아이"),
        'J' => Some("제이"), 'K' => Some("케이"), 'L' => Some("엘"),
        'M' => Some("엠"),   'N' => Some("엔"),   'O' => Some("오"),
        'P' => Some("피"),   'Q' => Some("큐"),   'R' => Some("아르"),
        'S' => Some("에스"), 'T' => Some("티"),   'U' => Some("유"),
        'V' => Some("브이"), 'W' => Some("더블유"),'X' => Some("엑스"),
        'Y' => Some("와이"), 'Z' => Some("지"),
        _ => None,
    }
}

// ─── Latin token dispatch ─────────────────────────────────────────────────────

fn split_punct(s: &str) -> (&str, &str, &str) {
    let is_word = |c: char| c.is_ascii_alphabetic() || c == '\'';
    let start = s.find(is_word).unwrap_or(s.len());
    let end = s
        .rfind(is_word)
        .map(|i| i + s[i..].chars().next().unwrap().len_utf8())
        .unwrap_or(0);
    if start >= end { (s, "", "") } else { (&s[..start], &s[start..end], &s[end..]) }
}

/// Like split_punct but also accepts non-ASCII alphabetic (for French/German/Spanish).
fn split_punct_unicode(s: &str) -> (&str, &str, &str) {
    let is_word = |c: char| c.is_alphabetic() || c == '\'' || c == '-';
    let start = s.find(is_word).unwrap_or(s.len());
    let end = s
        .rfind(is_word)
        .map(|i| i + s[i..].chars().next().unwrap().len_utf8())
        .unwrap_or(0);
    if start >= end { (s, "", "") } else { (&s[..start], &s[start..end], &s[end..]) }
}

fn transliterate_latin_token(token: &str, lang: LatinLang) -> String {
    let (prefix, core, suffix) = match lang {
        LatinLang::English => split_punct(token),
        _ => split_punct_unicode(token),
    };
    if core.is_empty() {
        return token.to_string();
    }
    // Single uppercase letter → Korean letter name (e.g., "A." → "에이.", "B)" → "비)")
    if core.len() == 1 {
        if let Some(ch) = core.chars().next() {
            if ch.is_ascii_uppercase() {
                if let Some(name) = letter_name_ko(ch) {
                    return format!("{prefix}{name}{suffix}");
                }
            }
        }
    }
    let lower = core.to_ascii_lowercase();
    let han = match lang {
        LatinLang::English => {
            if let Some(v) = DICT.get(lower.as_str()) {
                v.to_string()
            } else {
                rule_transliterate_en(&lower)
            }
        }
        LatinLang::French  => rule_transliterate_fr(core),
        LatinLang::Spanish => rule_transliterate_es(core),
        LatinLang::German  => rule_transliterate_de(core),
    };
    format!("{prefix}{han}{suffix}")
}

// ─── Utility: split sentence preserving delimiters ───────────────────────────

fn split_mixed(sentence: &str) -> (Vec<&str>, Vec<&str>) {
    // Returns (word_slices, separator_slices) interleaved: w0 s0 w1 s1 ...
    // last separator is empty string
    let mut words = Vec::new();
    let mut seps = Vec::new();
    let mut in_word = false;
    let mut word_start = 0;
    let mut sep_start = 0;
    let mut char_indices = sentence.char_indices();
    while let Some((idx, c)) = char_indices.next() {
        let is_alpha = c.is_alphabetic();
        if !in_word && is_alpha {
            seps.push(&sentence[sep_start..idx]);
            word_start = idx;
            in_word = true;
        } else if in_word && !is_alpha && !matches!(c, '\'' | '-') {
            words.push(&sentence[word_start..idx]);
            sep_start = idx;
            in_word = false;
        }
    }
    if in_word {
        words.push(&sentence[word_start..]);
        seps.push("");
    } else {
        seps.push(&sentence[sep_start..]);
    }
    (words, seps)
}

// ─── Main entry point ────────────────────────────────────────────────────────

/// Convert audio transcript text to Korean Hangul phonetics.
/// Language is auto-detected from the script and diacritics; no user input needed.
pub fn transliterate(sentence: &str) -> String {
    if sentence.trim().is_empty() {
        return sentence.to_string();
    }
    match dominant_script(sentence) {
        Script::Kana => kana_sentence_to_hangul(sentence),
        Script::Hangul => sentence.to_string(),
        Script::Cyrillic => cyrillic_sentence_to_hangul(sentence),
        Script::Cjk | Script::Other => sentence.to_string(),
        Script::Latin => {
            let lang = detect_latin_lang(sentence);
            sentence
                .split_whitespace()
                .map(|token| transliterate_latin_token(token, lang))
                .collect::<Vec<_>>()
                .join(" ")
        }
    }
}

// ─── Tests ───────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn english_dict_hit() {
        assert!(transliterate("Hello, world!").contains("헬로"));
    }

    #[test]
    fn english_fallback_hangul() {
        assert!(!rule_transliterate_en("crackle").is_empty());
    }

    #[test]
    fn english_number_dict() {
        assert_eq!(transliterate("Number 2"), "넘버 2");
    }

    #[test]
    fn english_coda_cluster() {
        assert_eq!(rule_transliterate_en("number"), "넘버");
    }

    #[test]
    fn english_er_digraph() {
        let out = rule_transliterate_en("butter");
        assert!(out.ends_with('어') || out.contains("터"), "got: {out}");
    }

    #[test]
    fn english_electrician() {
        let out = transliterate("The electrician is coming today, isn't he?");
        assert!(out.contains("일렉트리션"), "got: {out}");
        assert!(out.contains("커밍"), "got: {out}");
        assert!(out.contains("이즌트"), "got: {out}");
    }

    #[test]
    fn japanese_kana_basic() {
        // "こんにちは" → 콩니치하 (ん attaches as coda to こ → 콩)
        let out = kana_sentence_to_hangul("こんにちは");
        assert!(out.contains("콩") && out.contains("치") && out.contains("하"), "got: {out}");
    }

    #[test]
    fn japanese_katakana() {
        // コーヒー (coffee)
        let out = transliterate("コーヒー");
        assert!(out.contains("코") && out.contains("히"), "got: {out}");
    }

    #[test]
    fn russian_basic() {
        // "Москва" (Moscow)
        let out = transliterate("Москва");
        assert!(!out.is_empty() && out.chars().all(|c| !c.is_ascii_alphabetic()), "got: {out}");
    }

    #[test]
    fn french_bonjour() {
        let out = rule_transliterate_fr("bonjour");
        assert!(!out.is_empty(), "got: {out}");
    }

    #[test]
    fn french_nasal_detection() {
        let lang = detect_latin_lang("bonjour le monde");
        // no diacritics → English fallback; with é → French
        assert_eq!(lang, LatinLang::English);
        let lang2 = detect_latin_lang("très bien");
        assert_eq!(lang2, LatinLang::French);
    }

    #[test]
    fn german_umlaut_detection() {
        assert_eq!(detect_latin_lang("schön"), LatinLang::German);
    }

    #[test]
    fn spanish_detection() {
        assert_eq!(detect_latin_lang("mañana"), LatinLang::Spanish);
    }

    #[test]
    fn hangul_passthrough() {
        let s = "안녕하세요";
        assert_eq!(transliterate(s), s);
    }

    #[test]
    fn hard_c_corner() {
        // 'c' before 'o' → ㅋ (not ㅊ)
        let out = rule_transliterate_en("corner");
        assert!(out.starts_with('코'), "corner should start with 코, got: {out}");
    }

    #[test]
    fn soft_c_city() {
        // 'c' before 'i' → ㅅ
        let out = rule_transliterate_en("city");
        assert!(out.starts_with('시'), "city should start with 시, got: {out}");
    }

    #[test]
    fn letter_name_a() {
        // Uppercase single letter → Korean letter name
        let out = transliterate_latin_token("A.", LatinLang::English);
        assert_eq!(out, "에이.", "got: {out}");
    }

    #[test]
    fn letter_name_b() {
        let out = transliterate_latin_token("B.", LatinLang::English);
        assert_eq!(out, "비.", "got: {out}");
    }

    #[test]
    fn letter_name_c() {
        let out = transliterate_latin_token("C.", LatinLang::English);
        assert_eq!(out, "씨.", "got: {out}");
    }

    #[test]
    fn choice_sentence() {
        // Simulates TOEIC answer choice transcription
        let out = transliterate("C. Right around the corner.");
        assert!(out.starts_with("씨."), "got: {out}");
        assert!(out.contains("코너"), "corner should be 코너, got: {out}");
    }

    #[test]
    fn weve_contraction() {
        // "we've" should sound-map to 위브 (long /wi:v/), not 에브
        let out = rule_transliterate_en("we've");
        assert!(out.starts_with("위"), "we've should start with 위, got: {out}");
    }

    #[test]
    fn ei_long_e() {
        // English "ei" (receive, deceive) → /i:/ = 이
        let out = rule_transliterate_en("receive");
        assert!(out.contains("시"), "receive should contain 시 (long e), got: {out}");
    }

    #[test]
    fn ed_suffix() {
        // Past-tense -ed at end → silent e + 드
        let out = rule_transliterate_en("received");
        assert!(out.ends_with("드"), "received should end with 드, got: {out}");
    }

    #[test]
    fn re_prefix() {
        // Word-initial "re-" → 리 (unstressed long e)
        let out = rule_transliterate_en("received");
        assert!(out.starts_with("리"), "received should start with 리, got: {out}");
    }

    #[test]
    fn lowercase_a_no_dict() {
        // Lowercase "a" (article) goes through phonetic rules, not dictionary lookup
        let out = rule_transliterate_en("a");
        assert!(!out.is_empty(), "got: {out}");
    }

    #[test]
    fn open_syllable_long_a() {
        // Open syllable 'a' + C + V → long /eɪ/ = 에이
        let out = rule_transliterate_en("catering");
        assert!(out.starts_with("케이"), "catering should start with 케이, got: {out}");
    }

    #[test]
    fn full_sample_sentence() {
        // Full TOEIC-style sample: "A. I don't think we've received it."
        let out = transliterate("A. I don't think we've received it.");
        assert!(out.starts_with("에이."), "got: {out}");
        assert!(out.contains("위브"), "we've should be 위브, got: {out}");
        assert!(out.contains("리시브드"), "received should be 리시브드, got: {out}");
    }
}
