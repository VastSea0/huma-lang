//! Sürümlü backend destek sözleşmesi.

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Backend {
    Interpreter,
    BytecodeVm,
    Aot,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Stability {
    Normative,
    Experimental,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct BackendContract {
    pub backend: Backend,
    pub contract_version: u16,
    pub stability: Stability,
    pub coverage: &'static str,
}

pub const BACKEND_CONTRACTS: [BackendContract; 3] = [
    BackendContract {
        backend: Backend::Interpreter,
        contract_version: 1,
        stability: Stability::Normative,
        coverage: "tam-dil",
    },
    BackendContract {
        backend: Backend::BytecodeVm,
        contract_version: 1,
        stability: Stability::Experimental,
        coverage: "dogrulanan-bytecode-alt-kumesi",
    },
    BackendContract {
        backend: Backend::Aot,
        contract_version: 1,
        stability: Stability::Experimental,
        coverage: "sayisal-cranelift-alt-kumesi",
    },
];

pub fn contract(backend: Backend) -> &'static BackendContract {
    BACKEND_CONTRACTS
        .iter()
        .find(|contract| contract.backend == backend)
        .expect("bütün backend türlerinin sabit bir sözleşmesi olmalıdır")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn yalniz_yorumlayici_normatiftir() {
        let normative = BACKEND_CONTRACTS
            .iter()
            .filter(|contract| contract.stability == Stability::Normative)
            .collect::<Vec<_>>();
        assert_eq!(normative.len(), 1);
        assert_eq!(normative[0].backend, Backend::Interpreter);
    }
}
