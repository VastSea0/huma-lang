//! AST üzerinde özyinelemeli çalışan katmanlar için ev sahibi yığını koruması.
//!
//! Ayrıştırıcı, yorumlayıcı ve bytecode derleyicisi ağacı özyinelemeli
//! dolaşır. `parser::MAKS_YUVALAMA_DERINLIGI` ağacın derinliğini sınırlar,
//! fakat bir düzeyin çerçeve boyutu derleme profiline göre çok değişir ve
//! çağıran iş parçacığının yığını küçük olabilir (test iş parçacıkları ve LSP
//! işçileri 2 MiB). Kalan yığın `KIRMIZI_BOLGE`nin altına inerse iş, yığından
//! ayrılan yeni bir segmentte sürer; böylece süreç yığın taşmasıyla düşmez.

/// İki denetim noktası arasındaki en büyük özyinelemeli işi karşılamalıdır:
/// yorumlayıcı fonksiyon gövdesini çağrıda `clone` eder ve sınır derinliğindeki
/// bir gövdenin debug derlemede kopyalanması ~1,1 MiB yığın kullanır.
const KIRMIZI_BOLGE: usize = 2 * 1024 * 1024;
const YENI_SEGMENT: usize = 16 * 1024 * 1024;

/// `f`'yi en az `KIRMIZI_BOLGE` kadar boş yığınla çalıştırır.
pub fn yigini_koru<R>(f: impl FnOnce() -> R) -> R {
    stacker::maybe_grow(KIRMIZI_BOLGE, YENI_SEGMENT, f)
}
