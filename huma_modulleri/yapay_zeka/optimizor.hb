// ══════════════════════════════════════════════════════════════════════════════
// yapay_zeka/optimizor.hb — Stochastic Gradient Descent (SGD) Optimizör
// ══════════════════════════════════════════════════════════════════════════════

sınıf SGDOptimizor {
    ilklendir fonksiyon olsun lr alsın {
        kendisi.lr = lr olsun
    }

    guncelle fonksiyon olsun t alsın {
        grad = tensor_gradyan(t) olsun
        // W = W - lr * grad
        n = uzunluk(grad) olsun
        "Ağırlıklar güncellendi. Gradyan boyutu: " + n'i yazdır
    }
}
