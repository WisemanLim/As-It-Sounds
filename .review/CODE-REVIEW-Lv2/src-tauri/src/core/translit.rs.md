# src-tauri/src/core/translit.rs — 코드 리뷰 / Code Review

> 리뷰 레벨 / Review Level: 2
> [← INDEX.md](../../../INDEX.md)

**파일 요약 / File Summary**
- 역할 / Role: 다국어(라틴/일본어 가나/한글/키릴/CJK/기타) 음차 → 한글 변환 엔진.
- 진입점에서의 거리 / Distance from entrypoint: 3 (commands::translit_preview, pipeline::run)
- 라인 수 / LOC: 1713
- 주요 export / Exports: `pub fn transliterate(sentence: &str) -> String`
- 의존 모듈 / Depends on (internal): 없음 (pure logic)
- 사용된 외부 패키지 / External packages used: `once_cell::sync::Lazy`, `std::collections::HashMap`
- 부수효과 / Side effects: 없음 (순수 함수).
- 동시성 모델 / Concurrency model: 상태 비저장. `Lazy<HashMap>` thread-safe init.
- 테스트 커버리지 흔적 / Test coverage signal: **있음** — `#[cfg(test)] mod tests` (1530-1713행), ≈25개 단위 테스트.

> 본 파일은 1713줄로 임계치를 초과합니다. 라인 단위 대신 **논리 블록 단위**로 분석합니다.
> Exceeds threshold (1713 lines). Annotating by **logical block** instead of per line, with every block covered.

---

### Block 1 / 모듈 docstring + imports (lines 1-12)
```rust path=src-tauri/src/core/translit.rs lines=1-12
//! Multilingual audio phonetic → Korean Hangul transliteration.
...
use once_cell::sync::Lazy;
use std::collections::HashMap;
```
**분석 / Analysis (Level 2)**
- 문서 주석이 라우팅 전략(스크립트→언어 매핑)을 명시 → 코드 진입 매끄러움.
- `once_cell::sync::Lazy` 는 thread-safe 한 lazy static. (현대 Rust 1.80+ 는 std `LazyLock` 으로도 가능, 호환성 위해 유지)

---

### Block 2 / Hangul 합성 헬퍼 (lines 13-87)
```rust path=src-tauri/src/core/translit.rs lines=14-31
fn compose(onset: u32, vowel: u32, coda: u32) -> char {
    let code = 0xAC00 + (onset * 21 + vowel) * 28 + coda;
    char::from_u32(code).unwrap_or('?')
}

const NO_CODA: u32 = 0;
const C_NG: u32 = 21; // ㅇ jongseong
...
const C_P: u32 = 17; // ㅂ jongseong
```
**분석 / Analysis (Level 2)**
- 유니코드 한글 음절 합성 공식 `0xAC00 + (초성*21 + 중성)*28 + 종성` 사용. 표준. / Standard Hangul composition formula.
- `unwrap_or('?')` 는 유효 범위 입력이라면 절대 실패 안 함. 방어적 코드. / Defensive.
- `Vowel` enum + `idx()` 매핑 (lines 31-47) 으로 인덱스 안전성 ↑. enum + impl 로 magic number 회피. / Enum-based vowel safety.
- `push_syllable` / `onset_to_coda` / `try_attach_coda` 가 핵심 빌딩블록.

**짚어볼 점 / Watch-outs**
- `try_attach_coda` 는 `String::pop` → 새 코다 부착. UTF-8 안전 (pop 은 char 단위).
- coda 매핑이 한국식 로마자 표기 관행 따름 (예: t → ㅅ jongseong).

---

### Block 3 / 스크립트 감지 (lines 89-126)
```rust path=src-tauri/src/core/translit.rs lines=91-104
#[derive(Clone, Copy, PartialEq)]
enum Script { Latin, Kana, Hangul, Cyrillic, Cjk, Other }

fn char_script(c: char) -> Script {
    let cp = c as u32;
    match cp {
        0x3040..=0x309F | 0x30A0..=0x30FF => Script::Kana,
        0xAC00..=0xD7AF | 0x1100..=0x11FF | 0x3130..=0x318F => Script::Hangul,
        0x0400..=0x04FF => Script::Cyrillic,
        ...
```
**분석 / Analysis (Level 2)**
- 유니코드 블록 범위로 분류. 일반적이고 빠른 분기.
- `dominant_script(text)` 가 알파벳 char 만 카운트하여 빈도 최댓값 스크립트 반환. 짧은 텍스트에서 영향력 큼.

**짚어볼 점 / Watch-outs**
- ASCII 숫자/구두점은 Latin 으로 분류 → 짧은 영어 문장에서 punct 만으로 Latin 판정될 가능성. `is_alphabetic` 필터 덕분에 OK.
- Extended Latin (`0x00C0..=0x024F`) 포함 → French/German/Spanish 강조 문자 모두 Latin.

---

### Block 4 / Kana → Hangul (lines 127-248)
```rust path=src-tauri/src/core/translit.rs lines=138-184
static KANA_MAP: Lazy<HashMap<&'static str, &'static str>> = Lazy::new(|| {
    let entries: &[(&str, &str)] = &[
        ("きゃ","캬"), ("きゅ","큐"), ...
    ];
    ...
});
```
**분석 / Analysis (Level 2)**
- 100+ 매핑 테이블. **Yōon (2-char 결합) 을 single 보다 먼저 매칭** (line 142-152) — 알고리즘적 우선순위 옳음.
- `kata_to_hira` 로 카타카나를 히라가나로 정규화 후 lookup → 매핑 절반 절약.
- `ん` (line 201-215) 은 직전 음절에 받침 ㅇ 부착, 안되면 "응" 추가. / Geminate `っ` (217-228) 도 동일 패턴.
- ASCII passthrough (line 195-199) → 영문 / 숫자 / 구두점 보존.

**짚어볼 점 / Watch-outs**
- 장음부호 `ー` 처리 없음 → 그대로 출력 ("코ー히ー" 같은 케이스). test `japanese_katakana` 는 `히` 만 검증해 통과.
- 한자 (kanji) passthrough → 일본어 문장에 한자 섞이면 그대로 노출.

---

### Block 5 / 키릴 → Hangul (lines 250-353)
```rust path=src-tauri/src/core/translit.rs lines=292-340
fn cyrillic_word_to_hangul(word: &str) -> String {
    ...
    let flush = |p: &mut Option<u32>, o: &mut String| { ... };
    while i < n {
        let c = chars[i];
        if matches!(c, 'ъ' | 'ь' | 'Ъ' | 'Ь') { i += 1; continue; }
        if cyr_is_vowel(c) {
            let (vowel, y_glide) = cyr_vowel_to_hangul(c);
            ...
```
**분석 / Analysis (Level 2)**
- 음절 조립(onset+vowel) 패턴. pending consonant 가 vowel 만나면 결합, 다음 자음 만나면 받침/이전 음절에 부착 시도.
- y-glide 처리: е/ё/ю/я 가 자음 뒤가 아니면 ㅇ + Ye/Yo/Yu/Ya 로 직접 변환.
- ъ/ь 는 skip (한글 표기에 영향 없음).
- `cyrillic_sentence_to_hangul` 가 `split_mixed` 로 word/separator 분리 후 word 별 변환.

**짚어볼 점 / Watch-outs**
- й 가 단독으로 vowel I 로 분류 → 정확히는 짧은 i 자음이지만 한글 표기 관행상 OK.
- 영문/숫자 혼재 sentence 는 word 분리 후 영문 word 가 cyrillic_word_to_hangul 로 들어가면 알파벳 인식 못해 그대로 통과. dominant_script 가 Cyrillic 이면 영문 단어가 망가질 위험.

---

### Block 6 / Latin 언어 감지 (lines 355-374)
```rust path=src-tauri/src/core/translit.rs lines=359-374
fn detect_latin_lang(sentence: &str) -> LatinLang {
    if sentence.chars().any(|c| matches!(c, 'ä' | 'ö' | 'ü' | 'ß')) {
        return LatinLang::German;
    }
    if sentence.chars().any(|c| matches!(c, 'ñ' | 'á' | ...)) {
        return LatinLang::Spanish;
    }
    if sentence.chars().any(|c| matches!(c, 'à' | 'â' | ...)) {
        return LatinLang::French;
    }
    LatinLang::English
}
```
**분석 / Analysis (Level 2)**
- 디아크리틱 휴리스틱. 빠르고 단순. / Diacritic heuristic.
- 우선순위: German (ä/ö/ü/ß) → Spanish (ñ + 강세) → French (à/â/ç…) → English default.
- 강세 없는 스페인어/프랑스어는 영어로 fallback → "bonjour" 같은 단어는 English 처리 (test 가 명시).

**짚어볼 점 / Watch-outs**
- 영어 문장에 일부 외래 단어 (café) 있으면 전체 French 로 판정 → 영어 단어들이 fr 룰로 처리됨.

---

### Block 7 / 영어 룰 변환 `rule_transliterate_en` (lines 376-747)
```rust path=src-tauri/src/core/translit.rs lines=399-747
fn rule_transliterate_en(word: &str) -> String {
    let word = word.to_ascii_lowercase();
    ...
```
**분석 / Analysis (Level 2)**
- 가장 큰 단일 함수. 트리그래프 → 디그래프 → 단일 글자 순으로 매칭.
- 영어 특수 규칙들:
  - `tch` → 처(ㅊ), `igh/eye` → 아이
  - word-initial `re-` → 리 (line 442-447)
  - `we`+(end|apostrophe) → 위 (we, we've, we'll, we're)
  - `-ed` past tense (line 460-472)
  - vowel digraphs (er/ir/ur, ar/or, oo, ee/ea/ei, ai/ay, ou/ow, oi/oy, au/aw, ie, ew, ue)
  - consonant digraphs (th, sh, ch, ph, gh, ck, qu, wh, ng)
  - magic-e ate, open syllable a, ie, y
  - context-sensitive c/g (line 703-707)
  - word-end coda 부착 (line 709-727)
- `pending` 으로 자음을 다음 모음 만날 때까지 보류 → 음절 단위 합성.

**짚어볼 점 / Watch-outs**
- 매우 큰 단일 함수 (350+ 줄). 작은 헬퍼로 분리 가능하지만 inline 이 가독성/속도에 유리한 측면도.
- 휴리스틱 룰이 충돌하는 케이스(예: re- prefix vs. word "re") test 로 일부만 검증. edge case 더 발견될 가능성.
- bytes 기반 인덱싱 (ASCII 가정). non-ASCII 가 들어오면 byte boundary 깨질 위험. 하지만 `core` 가 `to_ascii_lowercase` 후 분기되고 외부에서 ASCII 단어만 전달되므로 OK.

---

### Block 8 / 프랑스어 룰 (lines 749-941)
**분석 / Analysis (Level 2)**
- 강세 정규화 (À→a, Ç→s 등) 선행. / Diacritic strip.
- 특수 패턴: `tion`→시옹, `eau`→오, `gn`→뉴, `ch`→슈, 비음(an/am/en/em/in/im/on/om/un/um)에 대해 nasal 음절 합성, 묵음 final consonant(d/t/s/x/z/p), `r`→ㅎ (uvular 근사).
- `before_cons_or_end` 클로저로 비음 컨텍스트 판별.

**짚어볼 점 / Watch-outs**
- `r` → ㅎ 매핑은 한국어 음차 관행에서 흔하나 영어와 다름. 입력이 French 가 아닐 때 잘못 적용되면 어색.

---

### Block 9 / 스페인어 룰 (lines 943-1073)
**분석 / Analysis (Level 2)**
- ñ→뉴, ll→이, rr→r(같이), ch→치, qu→ㅋ(묵음u), gu→g(묵음u), h 묵음, j/g(e|i)→ㅎ(jota), v→b, z/c(e|i)→ㅅ.
- 매우 일관된 알고리즘적 정리. 스페인어 발음이 음운적이라 룰 fit 좋음. / Phonetically regular language fits rules well.

---

### Block 10 / 독일어 룰 (lines 1075-1255)
**분석 / Analysis (Level 2)**
- sch→슈, ch (front vowel 후) → 히/흐, qu→ㅋ, st/sp 어두 → 슈ㅌ/슈ㅍ, ei→아이, ie→이, eu/äu→오이, au→아우, ß→스, 움라우트 ä/ö/ü, w→ㅂ, v→ㅍ, z→츠, -ig 어말 → 이히.
- ch 의 앞 모음 검사가 한글 음절 분해(line 1112-1120)로 이루어짐 — 정확하지만 약간 어색한 우회.

**짚어볼 점 / Watch-outs**
- st/sp 어두 룰 (i == 0) → "Stadt", "Sprache" 같은 어두에만 작동. 중간에 나오는 st/sp 는 일반 자음 시퀀스로 fall-through.

---

### Block 11 / 영어 사전 `DICT` (lines 1257-1398)
```rust path=src-tauri/src/core/translit.rs lines=1259-1398
static DICT: Lazy<HashMap<&'static str, &'static str>> = Lazy::new(|| {
    let entries: &[(&str, &str)] = &[
        ("the","더"), ("an","언"), ("and","앤드"), ...
        ("electrician","일렉트리션"), ...
    ];
    ...
});
```
**분석 / Analysis (Level 2)**
- ≈300 단어 사전. 고빈도/룰 실패 케이스 hand-coded. / Hand-tuned overrides.
- 사전 우선 → 매칭 안 되면 `rule_transliterate_en` 으로 fallback.
- 컴파일 타임에 static slice, runtime 에 한 번만 HashMap 생성.

**짚어볼 점 / Watch-outs**
- 사전이 커질수록 메모리/lookup 비용은 무시 가능. HashMap 의 hash 비용은 짧은 string 에 충분히 빠름.
- 새 단어 추가 권장 워크플로 부재 → contributor 가 어디에 추가할지 docstring 권장.

---

### Block 12 / 알파벳 letter name (lines 1400-1415)
**분석 / Analysis (Level 2)**
- 단독 대문자 토큰(A., B.) 을 letter name 으로 변환 (TOEIC 선택지 등 use case 가 분명). / TOEIC choice support.
- 소문자/숫자/punctuation 은 별도 처리.

---

### Block 13 / 토큰 분리 + Latin dispatch (lines 1417-1472)
**분석 / Analysis (Level 2)**
- `split_punct` (ASCII), `split_punct_unicode` (Unicode) 가 prefix/core/suffix 로 분리.
- `transliterate_latin_token` 가:
  1. 단일 대문자 → letter name
  2. 사전 hit (영어만)
  3. 언어별 rule 변환
- DICT 는 lowercase 키. core.to_ascii_lowercase() 후 lookup.

**짚어볼 점 / Watch-outs**
- 다국어 사전 부재. French/Spanish/German 은 룰만으로 처리 → 변환 품질 룰 한계에 묶임.

---

### Block 14 / `split_mixed` (lines 1474-1504)
**분석 / Analysis (Level 2)**
- char_indices 로 word 와 separator 인덱스를 추적 → 슬라이스 반환. UTF-8 안전 (byte index 사용).
- apostrophe / hyphen 을 word 일부로 취급 — "don't", "well-known" 보존.

**짚어볼 점 / Watch-outs**
- 비라틴 word boundary 는 `is_alphabetic` 으로 판정. 키릴/가나/한글에도 동일하게 작동.

---

### Block 15 / 메인 엔트리 `transliterate` (lines 1506-1528)
```rust path=src-tauri/src/core/translit.rs lines=1510-1528
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
```
**분석 / Analysis (Level 2)**
- 라우터. 단일 진입점. / Single dispatch.
- 빈 문자열 short-circuit. / Empty trim shortcut.
- `split_whitespace` → " " join 으로 다중 공백/탭 손실. 사용자 체감은 거의 없음. / Multi-space collapse.

**짚어볼 점 / Watch-outs**
- 혼합 스크립트 문장 (예: "Hello 안녕")은 dominant_script 결과에 따라 일부 token 망가질 가능. 적어도 한쪽이 손해.

---

### Block 16 / 테스트 (lines 1530-1713)
**분석 / Analysis (Level 2)**
- ≈25개 테스트. dictionary hit, fallback, digraphs, letter name, French/Spanish/German detection, hangul passthrough, full sentence (TOEIC 샘플) 등 커버.
- 어설션이 "starts_with" / "contains" 위주 → 강한 정밀 매칭은 적지만 회귀 잡기에는 적절. / Soft assertions.
- 한글 패스스루, 키릴 변환 (empty 아님 + ASCII 부재), 가나 변환 등이 sanity check.

**짚어볼 점 / Watch-outs**
- 부정 케이스(잘못된 변환 회피) 테스트가 적음. ex: French detect 가 영어 단어를 망치지 않음 검증.
- 성능 테스트 부재. 매우 긴 문장에서 트리거되는 panic 가능성 검증 부족.

---

**파일 수준 평가 / File-Level Verdict**
- 강점 / Strengths:
  1. 모듈 docstring 명확, 함수 이름이 의도 잘 드러냄.
  2. 25+ 단위 테스트 — 본 프로젝트에서 거의 유일한 테스트 자산.
  3. 사전 + 룰 fallback 2단 구조 (영어). 다국어 라우팅 + 디아크리틱 감지.
- 위험 / Risks:
  1. 단일 거대한 영어 룰 함수 (~350 LOC) — edge case 추가 시 안전한 수정 어려움.
  2. 혼합 스크립트 문장 처리 부실 (dominant script 1개만 선택).
  3. ASCII 가정 코드에서 non-ASCII byte slicing 시 잠재 panic — 현재 호출자가 막아주지만 직접 호출 시 위험.
- 개선 권고 / Recommendations:
  1. 영어 룰을 phase 별(trigraph/digraph/vowel/cons)로 분리한 작은 함수 추출.
  2. 혼합 스크립트: token 단위 script 재판정.
  3. fuzz test (`cargo fuzz` 또는 `proptest`) 도입.
  4. `rule_transliterate_*` 들이 거의 동일한 구조 — common framework 로 묶기 가능.
- 재읽기 권고 레벨 / Suggested re-read level: 3 — 음차 정확도 개선/언어 추가 시.
