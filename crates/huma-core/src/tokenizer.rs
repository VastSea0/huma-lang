use std::collections::HashMap;
use std::sync::{Arc, Mutex};

pub struct BPETokenizer {
    vocab: Vec<String>,
    token_to_id: HashMap<String, usize>,
    merges: Vec<(String, String)>,
}

impl BPETokenizer {
    pub fn new() -> Self {
        let mut vocab = Vec::new();
        let mut token_to_id = HashMap::new();

        // 256 ASCII / UTF-8 temel karakterleri ile başla
        for i in 0..256 {
            let ch = (i as u8 as char).to_string();
            token_to_id.insert(ch.clone(), vocab.len());
            vocab.push(ch);
        }

        Self {
            vocab,
            token_to_id,
            merges: Vec::new(),
        }
    }

    pub fn egit(&mut self, metin: &str, hedef_vocab_boyutu: usize) {
        let words: Vec<Vec<String>> = metin
            .split_whitespace()
            .map(|w| w.chars().map(|c| c.to_string()).collect())
            .collect();

        let mut current_words = words;

        while self.vocab.len() < hedef_vocab_boyutu {
            let mut pair_counts: HashMap<(String, String), usize> = HashMap::new();

            for word in &current_words {
                if word.len() < 2 { continue; }
                for i in 0..(word.len() - 1) {
                    let pair = (word[i].clone(), word[i + 1].clone());
                    *pair_counts.entry(pair).or_insert(0) += 1;
                }
            }

            if pair_counts.is_empty() { break; }

            // En çok tekrar eden çifti bul
            let best_pair = pair_counts
                .into_iter()
                .max_by_key(|&(_, count)| count)
                .map(|(pair, _)| pair);

            if let Some((first, second)) = best_pair {
                let merged = format!("{}{}", first, second);
                if !self.token_to_id.contains_key(&merged) {
                    self.token_to_id.insert(merged.clone(), self.vocab.len());
                    self.vocab.push(merged.clone());
                    self.merges.push((first.clone(), second.clone()));
                }

                // Kelimeleri güncelle (birleştir)
                for word in &mut current_words {
                    let mut i = 0;
                    while i < word.len().saturating_sub(1) {
                        if word[i] == first && word[i + 1] == second {
                            word[i] = merged.clone();
                            word.remove(i + 1);
                        } else {
                            i += 1;
                        }
                    }
                }
            } else {
                break;
            }
        }
    }

    pub fn kodla(&self, metin: &str) -> Vec<usize> {
        let mut result = Vec::new();
        for word in metin.split_whitespace() {
            let mut subwords: Vec<String> = word.chars().map(|c| c.to_string()).collect();

            for (first, second) in &self.merges {
                let merged = format!("{}{}", first, second);
                let mut i = 0;
                while i < subwords.len().saturating_sub(1) {
                    if &subwords[i] == first && &subwords[i + 1] == second {
                        subwords[i] = merged.clone();
                        subwords.remove(i + 1);
                    } else {
                        i += 1;
                    }
                }
            }

            for sw in subwords {
                if let Some(&id) = self.token_to_id.get(&sw) {
                    result.push(id);
                } else {
                    for ch in sw.chars() {
                        let ch_str = ch.to_string();
                        if let Some(&id) = self.token_to_id.get(&ch_str) {
                            result.push(id);
                        }
                    }
                }
            }
        }
        result
    }

    pub fn coz(&self, token_ids: &[usize]) -> String {
        let mut words = Vec::new();
        for &id in token_ids {
            if id < self.vocab.len() {
                words.push(self.vocab[id].clone());
            }
        }
        words.join("")
    }
}

pub static BPE_TOKENIZER: once_cell::sync::Lazy<Arc<Mutex<BPETokenizer>>> =
    once_cell::sync::Lazy::new(|| Arc::new(Mutex::new(BPETokenizer::new())));
