/// Deterministic resource limits applied by both execution backends.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ExecutionLimits {
    /// Maximum executed statements/instructions before execution is stopped.
    pub max_steps: u64,
    /// Maximum nested user-function calls.
    pub max_call_depth: usize,
    /// Maximum bytes written through the language output buffer.
    pub max_output_bytes: usize,
    /// Maximum item count accepted for one collection literal.
    pub max_collection_items: usize,
}

impl Default for ExecutionLimits {
    fn default() -> Self {
        Self {
            max_steps: 10_000_000,
            max_call_depth: 32,
            max_output_bytes: 16 * 1024 * 1024,
            max_collection_items: 1_000_000,
        }
    }
}

impl ExecutionLimits {
    pub fn validate(self) -> Result<Self, String> {
        if self.max_steps == 0 {
            return Err("Çalıştırma adım sınırı sıfır olamaz".to_string());
        }
        if self.max_call_depth == 0 {
            return Err("Çağrı derinliği sınırı sıfır olamaz".to_string());
        }
        if self.max_output_bytes == 0 {
            return Err("Çıktı sınırı sıfır olamaz".to_string());
        }
        if self.max_collection_items == 0 {
            return Err("Koleksiyon sınırı sıfır olamaz".to_string());
        }
        Ok(self)
    }
}
