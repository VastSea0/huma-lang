use std::collections::{BTreeMap, HashMap};
use std::sync::{Arc, Mutex};

pub struct BPETokenizer {
    vocab: Vec<Vec<u8>>,
    token_to_id: HashMap<Vec<u8>, usize>,
    merges: Vec<(Vec<u8>, Vec<u8>)>,
}

impl Default for BPETokenizer {
    fn default() -> Self {
        Self::new()
    }
}

impl BPETokenizer {
    pub fn new() -> Self {
        let mut vocab = Vec::with_capacity(256);
        let mut token_to_id = HashMap::with_capacity(256);

        // Bayt tabanı bütün geçerli UTF-8 metinleri kayıpsız temsil eder.
        for byte in 0..=u8::MAX {
            let token = vec![byte];
            token_to_id.insert(token.clone(), vocab.len());
            vocab.push(token);
        }

        Self {
            vocab,
            token_to_id,
            merges: Vec::new(),
        }
    }

    pub fn egit(&mut self, metin: &str, hedef_vocab_boyutu: usize) {
        // Her eğitim çağrısı aynı girdiden aynı modeli üretmelidir.
        *self = Self::new();
        let hedef_vocab_boyutu = hedef_vocab_boyutu.clamp(256, 65_536);
        let mut tokens = metin
            .as_bytes()
            .iter()
            .map(|byte| vec![*byte])
            .collect::<Vec<_>>();

        while self.vocab.len() < hedef_vocab_boyutu && tokens.len() >= 2 {
            let mut pair_counts: BTreeMap<(Vec<u8>, Vec<u8>), usize> = BTreeMap::new();

            for pair in tokens.windows(2) {
                *pair_counts
                    .entry((pair[0].clone(), pair[1].clone()))
                    .or_insert(0) += 1;
            }

            // BTreeMap eşit frekanslarda leksikografik ve tekrarlanabilir seçim sağlar.
            let best_pair = pair_counts
                .into_iter()
                .max_by(|(pair_a, count_a), (pair_b, count_b)| {
                    count_a.cmp(count_b).then_with(|| pair_b.cmp(pair_a))
                })
                .map(|(pair, _)| pair);

            if let Some((first, second)) = best_pair {
                let mut merged = first.clone();
                merged.extend_from_slice(&second);
                if !self.token_to_id.contains_key(&merged) {
                    self.token_to_id.insert(merged.clone(), self.vocab.len());
                    self.vocab.push(merged.clone());
                    self.merges.push((first.clone(), second.clone()));
                }

                let mut yeni = Vec::with_capacity(tokens.len());
                let mut i = 0;
                while i < tokens.len() {
                    if i + 1 < tokens.len() && tokens[i] == first && tokens[i + 1] == second {
                        yeni.push(merged.clone());
                        i += 2;
                    } else {
                        yeni.push(tokens[i].clone());
                        i += 1;
                    }
                }
                tokens = yeni;
            } else {
                break;
            }
        }
    }

    pub fn kodla(&self, metin: &str) -> Vec<usize> {
        let mut tokens = metin
            .as_bytes()
            .iter()
            .map(|byte| vec![*byte])
            .collect::<Vec<_>>();

        for (first, second) in &self.merges {
            let mut merged = first.clone();
            merged.extend_from_slice(second);
            let mut yeni = Vec::with_capacity(tokens.len());
            let mut i = 0;
            while i < tokens.len() {
                if i + 1 < tokens.len() && tokens[i] == *first && tokens[i + 1] == *second {
                    yeni.push(merged.clone());
                    i += 2;
                } else {
                    yeni.push(tokens[i].clone());
                    i += 1;
                }
            }
            tokens = yeni;
        }

        tokens
            .iter()
            .filter_map(|token| self.token_to_id.get(token).copied())
            .collect()
    }

    pub fn coz(&self, token_ids: &[usize]) -> Result<String, String> {
        let mut bytes = Vec::new();
        for id in token_ids {
            let token = self
                .vocab
                .get(*id)
                .ok_or_else(|| format!("BPE sözlüğünde {} kimlikli token yok", id))?;
            bytes.extend_from_slice(token);
        }
        String::from_utf8(bytes).map_err(|_| "Token dizisi geçerli UTF-8 üretmedi".to_string())
    }
}

pub static BPE_TOKENIZER: once_cell::sync::Lazy<Arc<Mutex<BPETokenizer>>> =
    once_cell::sync::Lazy::new(|| Arc::new(Mutex::new(BPETokenizer::new())));

#[cfg(test)]
mod tests {
    use super::BPETokenizer;

    #[test]
    fn turkce_utf8_ve_bosluklar_kayipsiz_doner() {
        let metin = "İyi günler, şeker ölçümü!";
        let mut tokenizer = BPETokenizer::new();
        tokenizer.egit(metin, 280);
        let tokenler = tokenizer.kodla(metin);

        assert_eq!(tokenizer.coz(&tokenler).unwrap(), metin);
    }

    #[test]
    fn egitim_tekrarlanan_bayt_dizisini_sikistirir() {
        let metin = "merhaba merhaba merhaba";
        let mut tokenizer = BPETokenizer::new();
        tokenizer.egit(metin, 270);

        assert!(tokenizer.kodla(metin).len() < metin.len());
    }

    #[test]
    fn bilinmeyen_token_kimligi_hata_verir() {
        let tokenizer = BPETokenizer::new();
        assert!(tokenizer.coz(&[999]).is_err());
    }
}
