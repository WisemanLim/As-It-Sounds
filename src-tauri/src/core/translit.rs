//! English → Korean phonetic (hangul) transliteration.
//!
//! Strategy: (1) dictionary lookup for common words, (2) rule-based fallback
//! that composes Hangul syllables from onset/vowel/optional coda matched from
//! the input English letters.

use once_cell::sync::Lazy;
use std::collections::HashMap;

/// Compose a Hangul syllable block from (onset, vowel, coda) jamo indices.
/// Uses the Unicode formula: 0xAC00 + (onset*21 + vowel)*28 + coda.
fn compose(onset: u32, vowel: u32, coda: u32) -> char {
    let code = 0xAC00 + (onset * 21 + vowel) * 28 + coda;
    char::from_u32(code).unwrap_or('?')
}

// Hangul initial consonants (choseong) indices 0..18
// 0 ㄱ 1 ㄲ 2 ㄴ 3 ㄷ 4 ㄸ 5 ㄹ 6 ㅁ 7 ㅂ 8 ㅃ 9 ㅅ 10 ㅆ 11 ㅇ 12 ㅈ 13 ㅉ 14 ㅊ 15 ㅋ 16 ㅌ 17 ㅍ 18 ㅎ
// Vowels (jungseong) 0..20
// 0 ㅏ 1 ㅐ 2 ㅑ 3 ㅒ 4 ㅓ 5 ㅔ 6 ㅕ 7 ㅖ 8 ㅗ 9 ㅘ 10 ㅙ 11 ㅚ 12 ㅛ 13 ㅜ 14 ㅝ 15 ㅞ 16 ㅟ 17 ㅠ 18 ㅡ 19 ㅢ 20 ㅣ
// Finals (jongseong) 0..27 (0 means none)
const NO_CODA: u32 = 0;
const C_NG: u32 = 21; // ㅇ final
const C_L: u32 = 8; // ㄹ final
const C_M: u32 = 16; // ㅁ final
const C_N: u32 = 4; // ㄴ final
const C_K: u32 = 1; // ㄱ final
const C_T: u32 = 19; // ㅅ final (commonly used for t-stops in Korean loanwords)
const C_P: u32 = 17; // ㅂ final

#[derive(Clone, Copy)]
enum Vowel {
    A,   // 아
    Ae,  // 애
    Eo,  // 어
    E,   // 에
    I,   // 이
    O,   // 오
    U,   // 우
    Eu,  // 으
    Ya,  // 야
    Yeo, // 여
    Yu,  // 유
    Yo,  // 요
    Wa,  // 와
    Wi,  // 위
    Oe,  // 외
    Ui,  // 의
}

impl Vowel {
    fn idx(self) -> u32 {
        match self {
            Vowel::A => 0,
            Vowel::Ae => 1,
            Vowel::Ya => 2,
            Vowel::Eo => 4,
            Vowel::E => 5,
            Vowel::Yeo => 6,
            Vowel::O => 8,
            Vowel::Wa => 9,
            Vowel::Oe => 11,
            Vowel::Yo => 12,
            Vowel::U => 13,
            Vowel::Wi => 16,
            Vowel::Yu => 17,
            Vowel::Eu => 18,
            Vowel::Ui => 19,
            Vowel::I => 20,
        }
    }
}

/// Onset consonant index, or special "ㅇ" when the syllable has no consonant.
fn onset_for(c: char) -> Option<u32> {
    Some(match c {
        'g' | 'G' => 0,     // ㄱ
        'n' | 'N' => 2,     // ㄴ
        'd' | 'D' => 3,     // ㄷ
        'r' | 'R' | 'l' | 'L' => 5, // ㄹ
        'm' | 'M' => 6,     // ㅁ
        'b' | 'B' | 'v' | 'V' => 7, // ㅂ
        's' | 'S' | 'z' | 'Z' => 9, // ㅅ
        'j' | 'J' => 12,    // ㅈ
        'c' | 'C' => 14,    // ㅊ (context-dependent; will be overridden)
        'k' | 'K' | 'q' | 'Q' => 15, // ㅋ
        't' | 'T' => 16,    // ㅌ
        'p' | 'P' | 'f' | 'F' => 17, // ㅍ
        'h' | 'H' => 18,    // ㅎ
        'w' | 'W' | 'y' | 'Y' => 11, // placeholder ㅇ; vowel-glide handled
        'x' | 'X' => 15,    // "x" → ㅋ + ㅅ(s)
        _ => return None,
    })
}

fn push_syllable(out: &mut String, onset: u32, vowel: Vowel, coda: u32) {
    out.push(compose(onset, vowel.idx(), coda));
}

fn is_vowel(c: char) -> bool {
    matches!(
        c.to_ascii_lowercase(),
        'a' | 'e' | 'i' | 'o' | 'u' | 'y'
    )
}

/// Heuristic rule-based transliterator for a single English word.
fn rule_transliterate_word(word: &str) -> String {
    let word = word.to_ascii_lowercase();
    let bytes = word.as_bytes();
    let n = bytes.len();
    if n == 0 {
        return String::new();
    }

    let mut out = String::new();
    let mut i = 0;
    // Track the previous onset-consonant we emitted (pending, needs a vowel).
    // If we can't find a vowel, we'll emit it with 으 filler.
    let mut pending: Option<u32> = None;

    // Small helper closures that can read context
    let ch = |i: usize| -> Option<char> { bytes.get(i).map(|b| *b as char) };
    let is_at_vowel = |i: usize| ch(i).map(is_vowel).unwrap_or(false);

    while i < n {
        // Multi-letter digraphs first
        let slice2: &str = std::str::from_utf8(&bytes[i..(i + 2).min(n)]).unwrap_or("");
        let slice3: &str = std::str::from_utf8(&bytes[i..(i + 3).min(n)]).unwrap_or("");

        // Flush a pending onset with 으 if the current thing isn't a vowel cluster.
        let flush_pending = |pending: &mut Option<u32>, out: &mut String| {
            if let Some(p) = pending.take() {
                push_syllable(out, p, Vowel::Eu, NO_CODA);
            }
        };

        // Trigraphs
        if slice3 == "tch" {
            flush_pending(&mut pending, &mut out);
            pending = Some(14); // ㅊ
            i += 3;
            continue;
        }
        if slice3 == "igh" || slice3 == "eye" {
            flush_pending(&mut pending, &mut out);
            push_syllable(&mut out, 11, Vowel::A, NO_CODA); // 아
            push_syllable(&mut out, 11, Vowel::I, NO_CODA); // 이
            i += 3;
            continue;
        }

        // Vowel digraphs
        match slice2 {
            "oo" => {
                let o = pending.take().unwrap_or(11);
                push_syllable(&mut out, o, Vowel::U, NO_CODA);
                i += 2;
                continue;
            }
            "ee" | "ea" => {
                let o = pending.take().unwrap_or(11);
                push_syllable(&mut out, o, Vowel::I, NO_CODA);
                i += 2;
                continue;
            }
            "ai" | "ay" => {
                let o = pending.take().unwrap_or(11);
                push_syllable(&mut out, o, Vowel::E, NO_CODA); // 에
                push_syllable(&mut out, 11, Vowel::I, NO_CODA); // 이
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
                pending = Some(9); // ㅅ (voiceless "th")
                i += 2;
                continue;
            }
            "sh" => {
                flush_pending(&mut pending, &mut out);
                // "sh" + vowel → 쉬/샤/쇼 etc. approximate by emitting "쉬" if followed by vowel
                if i + 2 < n && is_at_vowel(i + 2) {
                    // leave pending=ㅅ but rely on vowel mapping; to simulate 쉬 we
                    // push 쉬(si) followed by the vowel.
                    push_syllable(&mut out, 9, Vowel::Wi, NO_CODA); // 쉬 approximation
                    i += 2;
                    continue;
                } else {
                    push_syllable(&mut out, 9, Vowel::Wi, NO_CODA); // 쉬
                    i += 2;
                    continue;
                }
            }
            "ch" => {
                flush_pending(&mut pending, &mut out);
                pending = Some(14); // ㅊ
                i += 2;
                continue;
            }
            "ph" => {
                flush_pending(&mut pending, &mut out);
                pending = Some(17); // ㅍ
                i += 2;
                continue;
            }
            "gh" => {
                // often silent; skip
                i += 2;
                continue;
            }
            "ck" => {
                flush_pending(&mut pending, &mut out);
                // coda ㄱ attached to previous or standalone 크
                if let Some(last) = out.pop() {
                    // Try to put ㄱ as coda on the previous syllable if it has no coda.
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
                pending = Some(15); // ㅋ
                i += 2;
                continue;
            }
            "qu" => {
                flush_pending(&mut pending, &mut out);
                push_syllable(&mut out, 15, Vowel::U, NO_CODA); // 쿠
                i += 2;
                continue;
            }
            "wh" => {
                flush_pending(&mut pending, &mut out);
                i += 1; // treat as 'w'
                continue;
            }
            "ng" => {
                flush_pending(&mut pending, &mut out);
                // attach ㅇ coda to previous syllable if none, else emit 응
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
                push_syllable(&mut out, 11, Vowel::Eu, C_NG); // 응
                i += 2;
                continue;
            }
            _ => {}
        }

        // Single char
        let c = bytes[i] as char;
        if is_vowel(c) {
            let vowel = match c {
                'a' => {
                    // silent 'e' at end -> 'a' becomes long
                    if i + 2 == n && bytes[i + 1] as char == 'e' {
                        // "ate" handled earlier as digraph? not exactly. handle here.
                        // emit 에이 and consume 2 chars.
                        let o = pending.take().unwrap_or(11);
                        push_syllable(&mut out, o, Vowel::E, NO_CODA);
                        push_syllable(&mut out, 11, Vowel::I, NO_CODA);
                        i += 2;
                        continue;
                    }
                    Vowel::A
                }
                'e' => {
                    if i + 1 == n {
                        // silent final e: don't emit anything (but flush pending as consonant+eu if alone)
                        if let Some(p) = pending.take() {
                            push_syllable(&mut out, p, Vowel::Eu, NO_CODA);
                        }
                        i += 1;
                        continue;
                    }
                    Vowel::E
                }
                'i' => {
                    // "ite"/"ide"/"ice" → 아이 type; approximate with 아이 if followed by consonant+e_final
                    if i + 2 < n && !is_at_vowel(i + 1) && bytes[i + 2] as char == 'e' {
                        let o = pending.take().unwrap_or(11);
                        push_syllable(&mut out, o, Vowel::A, NO_CODA);
                        push_syllable(&mut out, 11, Vowel::I, NO_CODA);
                        // fall through to consonant on next iter; don't consume 'e'
                        i += 1;
                        continue;
                    }
                    Vowel::I
                }
                'o' => Vowel::O,
                'u' => Vowel::Eo, // closer to ʌ for most common cases
                'y' => {
                    if i == 0 && i + 1 < n && is_at_vowel(i + 1) {
                        // "ye", "yo" etc handled in next iteration as glide
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

        // Consonant
        if let Some(cons) = onset_for(c) {
            flush_pending(&mut pending, &mut out);
            // At end of word: attach as coda if possible, else 으 filler
            if i + 1 == n {
                // Try to attach as coda on previous syllable
                if let Some(last) = out.pop() {
                    let cu = last as u32;
                    if (0xAC00..=0xD7A3).contains(&cu) {
                        let rel = cu - 0xAC00;
                        if rel % 28 == 0 {
                            let coda = match c {
                                'n' => Some(C_N),
                                'm' => Some(C_M),
                                'l' | 'r' => Some(C_L),
                                'k' => Some(C_K),
                                'g' => Some(C_K),
                                't' | 'd' | 's' | 'z' => Some(C_T),
                                'p' | 'b' => Some(C_P),
                                _ => None,
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
                // fallback 으 filler
                push_syllable(&mut out, cons, Vowel::Eu, NO_CODA);
                i += 1;
                continue;
            }
            pending = Some(cons);
            i += 1;
            continue;
        } else {
            // Skip unknown chars
            i += 1;
        }
    }

    if let Some(p) = pending.take() {
        push_syllable(&mut out, p, Vowel::Eu, NO_CODA);
    }

    out
}

/// Dictionary of common English words mapped to carefully-chosen Korean
/// phonetic readings that outperform the rule-based fallback.
static DICT: Lazy<HashMap<&'static str, &'static str>> = Lazy::new(|| {
    let entries: &[(&str, &str)] = &[
        ("a", "어"),
        ("the", "더"),
        ("an", "언"),
        ("and", "앤드"),
        ("or", "오어"),
        ("but", "벗"),
        ("is", "이즈"),
        ("are", "아"),
        ("am", "앰"),
        ("was", "워즈"),
        ("were", "워"),
        ("be", "비"),
        ("been", "빈"),
        ("being", "비잉"),
        ("do", "두"),
        ("does", "더즈"),
        ("did", "디드"),
        ("done", "던"),
        ("have", "해브"),
        ("has", "해즈"),
        ("had", "해드"),
        ("will", "윌"),
        ("would", "우드"),
        ("can", "캔"),
        ("could", "쿠드"),
        ("should", "슈드"),
        ("shall", "쉘"),
        ("may", "메이"),
        ("might", "마이트"),
        ("must", "머스트"),
        ("i", "아이"),
        ("you", "유"),
        ("he", "히"),
        ("she", "쉬"),
        ("it", "잇"),
        ("we", "위"),
        ("they", "데이"),
        ("me", "미"),
        ("him", "힘"),
        ("her", "허"),
        ("us", "어스"),
        ("them", "뎀"),
        ("my", "마이"),
        ("your", "유어"),
        ("his", "히즈"),
        ("its", "잇츠"),
        ("our", "아워"),
        ("their", "데어"),
        ("this", "디스"),
        ("that", "댓"),
        ("these", "디즈"),
        ("those", "도우즈"),
        ("here", "히어"),
        ("there", "데어"),
        ("where", "웨어"),
        ("what", "왓"),
        ("when", "웬"),
        ("why", "와이"),
        ("how", "하우"),
        ("who", "후"),
        ("which", "위치"),
        ("hello", "헬로"),
        ("hi", "하이"),
        ("hey", "헤이"),
        ("bye", "바이"),
        ("goodbye", "굿바이"),
        ("yes", "예스"),
        ("no", "노"),
        ("ok", "오케이"),
        ("okay", "오케이"),
        ("thanks", "땡스"),
        ("thank", "땡크"),
        ("please", "플리즈"),
        ("sorry", "쏘리"),
        ("welcome", "웰컴"),
        ("love", "러브"),
        ("like", "라이크"),
        ("want", "원트"),
        ("need", "니드"),
        ("know", "노우"),
        ("think", "씽크"),
        ("see", "씨"),
        ("look", "룩"),
        ("go", "고"),
        ("come", "컴"),
        ("get", "겟"),
        ("make", "메이크"),
        ("take", "테이크"),
        ("give", "기브"),
        ("find", "파인드"),
        ("use", "유즈"),
        ("work", "워크"),
        ("call", "콜"),
        ("try", "트라이"),
        ("ask", "애스크"),
        ("help", "헬프"),
        ("say", "세이"),
        ("tell", "텔"),
        ("talk", "토크"),
        ("speak", "스피크"),
        ("read", "리드"),
        ("write", "라이트"),
        ("play", "플레이"),
        ("run", "런"),
        ("walk", "워크"),
        ("drive", "드라이브"),
        ("eat", "잇"),
        ("drink", "드링크"),
        ("sleep", "슬립"),
        ("today", "투데이"),
        ("tomorrow", "투마로우"),
        ("yesterday", "예스터데이"),
        ("now", "나우"),
        ("then", "덴"),
        ("soon", "순"),
        ("good", "굿"),
        ("bad", "배드"),
        ("new", "뉴"),
        ("old", "올드"),
        ("big", "빅"),
        ("small", "스몰"),
        ("high", "하이"),
        ("low", "로우"),
        ("fast", "패스트"),
        ("slow", "슬로우"),
        ("hot", "핫"),
        ("cold", "콜드"),
        ("first", "퍼스트"),
        ("last", "라스트"),
        ("next", "넥스트"),
        ("one", "원"),
        ("two", "투"),
        ("three", "쓰리"),
        ("four", "포"),
        ("five", "파이브"),
        ("six", "식스"),
        ("seven", "세븐"),
        ("eight", "에잇"),
        ("nine", "나인"),
        ("ten", "텐"),
        ("of", "오브"),
        ("in", "인"),
        ("on", "온"),
        ("at", "앳"),
        ("to", "투"),
        ("for", "포"),
        ("from", "프롬"),
        ("with", "위드"),
        ("without", "위다웃"),
        ("about", "어바웃"),
        ("as", "애즈"),
        ("so", "쏘"),
        ("if", "이프"),
        ("because", "비코즈"),
        ("very", "베리"),
        ("just", "저스트"),
        ("only", "온리"),
        ("also", "올쏘"),
        ("more", "모어"),
        ("most", "모스트"),
        ("some", "썸"),
        ("any", "애니"),
        ("all", "올"),
        ("every", "에브리"),
        ("each", "이치"),
        ("many", "매니"),
        ("much", "머치"),
        ("few", "퓨"),
        ("little", "리틀"),
        ("people", "피플"),
        ("person", "퍼슨"),
        ("man", "맨"),
        ("woman", "우먼"),
        ("child", "차일드"),
        ("boy", "보이"),
        ("girl", "걸"),
        ("friend", "프렌드"),
        ("family", "패밀리"),
        ("home", "홈"),
        ("house", "하우스"),
        ("school", "스쿨"),
        ("work", "워크"),
        ("time", "타임"),
        ("day", "데이"),
        ("night", "나이트"),
        ("year", "이어"),
        ("month", "먼스"),
        ("week", "위크"),
        ("hour", "아워"),
        ("minute", "미닛"),
        ("world", "월드"),
        ("country", "컨트리"),
        ("city", "시티"),
        ("street", "스트릿"),
        ("car", "카"),
        ("bus", "버스"),
        ("train", "트레인"),
        ("book", "북"),
        ("phone", "폰"),
        ("computer", "컴퓨터"),
        ("music", "뮤직"),
        ("movie", "무비"),
        ("video", "비디오"),
        ("audio", "오디오"),
        ("english", "잉글리시"),
        ("korean", "코리안"),
        ("app", "앱"),
        ("apps", "앱스"),
        ("sounds", "사운즈"),
        ("sound", "사운드"),
        ("start", "스타트"),
        ("started", "스타티드"),
        ("starting", "스타팅"),
        ("stop", "스톱"),
        ("hello,", "헬로,"),
    ];
    let mut map = HashMap::new();
    for (k, v) in entries {
        map.insert(*k, *v);
    }
    map
});

fn transliterate_token(token: &str) -> String {
    // strip wrapping punctuation, preserve prefix/suffix
    let (prefix, core, suffix) = split_punct(token);
    let lower = core.to_ascii_lowercase();
    if lower.is_empty() {
        return token.to_string();
    }
    let han = if let Some(v) = DICT.get(lower.as_str()) {
        v.to_string()
    } else {
        rule_transliterate_word(&lower)
    };
    format!("{prefix}{han}{suffix}")
}

fn split_punct(s: &str) -> (&str, &str, &str) {
    let is_word = |c: char| c.is_ascii_alphabetic() || c == '\'';
    let start = s.find(is_word).unwrap_or(s.len());
    let end = s
        .rfind(is_word)
        .map(|i| i + s[i..].chars().next().unwrap().len_utf8())
        .unwrap_or(0);
    if start >= end {
        (s, "", "")
    } else {
        (&s[..start], &s[start..end], &s[end..])
    }
}

/// Transliterate an English sentence into Korean phonetic (hangul).
pub fn transliterate(sentence: &str) -> String {
    sentence
        .split_whitespace()
        .map(transliterate_token)
        .collect::<Vec<_>>()
        .join(" ")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn sentence_dict_hits() {
        let out = transliterate("Hello, world!");
        assert!(out.contains("헬로"));
    }

    #[test]
    fn fallback_produces_hangul() {
        let out = rule_transliterate_word("crackle");
        assert!(!out.is_empty());
    }
}
