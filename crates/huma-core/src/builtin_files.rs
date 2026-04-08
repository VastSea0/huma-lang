pub fn get_lib_files() -> Vec<(&'static str, &'static str)> {
    vec![
        ("birim_test.hb", include_str!("../../../lib/birim_test.hb")),
        ("dizgi.hb",      include_str!("../../../lib/dizgi.hb")),
        ("dosya.hb",      include_str!("../../../lib/dosya.hb")),
        ("istatistik.hb", include_str!("../../../lib/istatistik.hb")),
        ("liste.hb",      include_str!("../../../lib/liste.hb")),
        ("matematik.hb",  include_str!("../../../lib/matematik.hb")),
        ("nlp.hb",        include_str!("../../../lib/nlp.hb")),
        ("rastgele.hb",   include_str!("../../../lib/rastgele.hb")),
        ("renkler.hb",    include_str!("../../../lib/renkler.hb")),
        ("zaman.hb",      include_str!("../../../lib/zaman.hb")),
    ]
}

pub fn get_example_files() -> Vec<(&'static str, &'static str)> {
    vec![
        ("fibonacci.hb", include_str!("../../../examples/fibonacci.hb")),
        ("merhaba.hb", include_str!("../../../examples/merhaba.hb")),
        ("uygulama_gorevler.hb", include_str!("../../../examples/uygulama_gorevler.hb")),
        ("uygulama_log_analiz.hb", include_str!("../../../examples/uygulama_log_analiz.hb")),
        ("uygulama_tahmin.hb", include_str!("../../../examples/uygulama_tahmin.hb")),
    ]
}
