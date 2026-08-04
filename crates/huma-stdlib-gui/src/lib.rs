//! Hüma'nın isteğe bağlı yerel GUI adaptörü.
//!
//! Bu paket, normatif çalışma zamanını pencereleme ve grafik
//! bağımlılıklarından ayırır. Bir ana makine GUI yerleşiklerini kullanmak
//! istiyorsa [`kayit_et`] işlevini açıkça çağırmalıdır.

mod gui;

pub use gui::*;
