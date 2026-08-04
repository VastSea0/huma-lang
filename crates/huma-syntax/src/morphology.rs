//! Hüma kaynak kodundaki kesme işaretli Türkçe yüzey ekleri.
//!
//! Programlama dili, bir tanımlayıcının telaffuzunu güvenilir biçimde bilemez.
//! Bu nedenle sayı ile biten adlar, tek harfli adlar, ünlüsüz kısaltmalar ve
//! tamamı büyük harfli kısaltmalar denetim dışında tutulur. Diğer adlarda
//! kanonik biçim, son yazılı ünlüye dayanan büyük/küçük ünlü uyumu ile sert
//! ünsüz benzeşmesidir.

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SuffixHarmonyError {
    pub stem: String,
    pub suffix: String,
    pub expected: Vec<String>,
}

impl std::fmt::Display for SuffixHarmonyError {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            formatter,
            "{}'{} eki Türkçe ek uyumuna uymuyor; beklenen: {}",
            self.stem,
            self.suffix,
            self.expected
                .iter()
                .map(|suffix| format!("'{}", suffix))
                .collect::<Vec<_>>()
                .join(" veya ")
        )
    }
}

fn four_way_vowel(vowel: char) -> char {
    match vowel {
        'a' | 'ı' => 'ı',
        'o' | 'u' => 'u',
        'e' | 'i' => 'i',
        'ö' | 'ü' => 'ü',
        _ => vowel,
    }
}

fn two_way_vowel(vowel: char) -> char {
    if matches!(vowel, 'a' | 'ı' | 'o' | 'u') {
        'a'
    } else {
        'e'
    }
}

fn is_vowel(character: char) -> bool {
    matches!(character, 'a' | 'e' | 'ı' | 'i' | 'o' | 'ö' | 'u' | 'ü')
}

fn is_voiceless_consonant(character: char) -> bool {
    matches!(character, 'ç' | 'f' | 'h' | 'k' | 'p' | 's' | 'ş' | 't')
}

fn lower_turkish(character: char) -> char {
    match character {
        'I' => 'ı',
        'İ' => 'i',
        other => other.to_lowercase().next().unwrap_or(other),
    }
}

fn checkable_stem(stem: &str) -> Option<(char, char)> {
    let segment = stem.rsplit('_').next().unwrap_or(stem);
    let letters = segment
        .chars()
        .filter(|character| character.is_alphabetic())
        .collect::<Vec<_>>();
    if segment.chars().any(|character| character.is_numeric())
        || letters.len() < 2
        || (!letters.is_empty() && letters.iter().all(|character| character.is_uppercase()))
    {
        return None;
    }

    let lowered = segment.chars().map(lower_turkish).collect::<Vec<_>>();
    let last = lowered.last().copied()?;
    let last_vowel = lowered
        .iter()
        .rev()
        .copied()
        .find(|character| is_vowel(*character))?;
    Some((last, last_vowel))
}

fn accepted_suffixes(stem: &str, suffix: &str) -> Option<Vec<String>> {
    let (last, last_vowel) = checkable_stem(stem)?;
    let ends_in_vowel = is_vowel(last);
    let high = four_way_vowel(last_vowel);
    let low = two_way_vowel(last_vowel);
    let dental = if is_voiceless_consonant(last) {
        't'
    } else {
        'd'
    };
    let equative = if is_voiceless_consonant(last) {
        'ç'
    } else {
        'c'
    };

    let forms = match suffix {
        "i" | "ı" | "u" | "ü" | "yi" | "yı" | "yu" | "yü" | "ni" | "nı" | "nu" | "nü" => {
            if ends_in_vowel {
                // `y` genel kaynaştırmadır; `n` zamir/iyelik kökenli adlarda
                // kullanılır ve sözcüksel sınıf bilgisi olmadan ayırt edilemez.
                vec![format!("y{high}"), format!("n{high}")]
            } else {
                vec![high.to_string()]
            }
        }
        "a" | "e" | "ya" | "ye" => vec![if ends_in_vowel {
            format!("y{low}")
        } else {
            low.to_string()
        }],
        "da" | "de" | "ta" | "te" => vec![format!("{dental}{low}")],
        "dan" | "den" | "tan" | "ten" => vec![format!("{dental}{low}n")],
        "nin" | "nın" | "nun" | "nün" | "in" | "ın" | "un" | "ün" => {
            vec![if ends_in_vowel {
                format!("n{high}n")
            } else {
                format!("{high}n")
            }]
        }
        "si" | "sı" | "su" | "sü" => vec![if ends_in_vowel {
            format!("s{high}")
        } else {
            high.to_string()
        }],
        "lar" | "ler" => vec![format!("l{low}r")],
        "ca" | "ce" | "ça" | "çe" => vec![format!("{equative}{low}")],
        "daki" | "deki" | "taki" | "teki" => vec![format!("{dental}{low}ki")],
        "la" | "le" | "yla" | "yle" => vec![if ends_in_vowel {
            format!("yl{low}")
        } else {
            format!("l{low}")
        }],
        _ => return None,
    };
    Some(forms)
}

pub fn validate_suffix_harmony(stem: &str, suffix: &str) -> Result<(), SuffixHarmonyError> {
    let Some(expected) = accepted_suffixes(stem, suffix) else {
        return Ok(());
    };
    if expected.iter().any(|candidate| candidate == suffix) {
        Ok(())
    } else {
        Err(SuffixHarmonyError {
            stem: stem.to_string(),
            suffix: suffix.to_string(),
            expected,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::validate_suffix_harmony;

    #[test]
    fn unlu_uyumu_ve_unsuz_benzesmesini_dogrular() {
        for (stem, suffix) in [
            ("değer", "i"),
            ("sayı", "yı"),
            ("sayı", "dan"),
            ("kitap", "ta"),
            ("göz", "ün"),
            ("araba", "ya"),
            ("kendisi", "ni"),
        ] {
            assert!(
                validate_suffix_harmony(stem, suffix).is_ok(),
                "{stem}'{suffix} geçerli olmalı"
            );
        }
        assert!(validate_suffix_harmony("değer", "yi").is_err());
        assert!(validate_suffix_harmony("kitap", "da").is_err());
        assert!(validate_suffix_harmony("tokens", "ı").is_err());
    }

    #[test]
    fn telaffuzu_kaynaktan_cikarilamayan_adlari_reddetmez() {
        for (stem, suffix) in [("n", "i"), ("PI", "yi"), ("sonuç2", "yi"), ("fks", "ı")] {
            assert!(validate_suffix_harmony(stem, suffix).is_ok());
        }
    }
}
