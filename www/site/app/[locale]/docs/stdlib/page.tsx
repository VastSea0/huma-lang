import Link from "next/link";
import { getDictionary } from "@/dictionaries/dictionaries";
import type { Metadata } from "next";

export const metadata: Metadata = {
  title: "Standard Library",
  description:
    "Complete API reference for Hüma's built-in standard library modules.",
};

interface StdlibFunction {
  name: string;
  signature: string;
  description: string;
}

interface StdlibModule {
  id: string;
  file: string;
  icon: string;
  title: string;
  descriptionKey: string;
  constants?: { name: string; value: string; description: string }[];
  functions: StdlibFunction[];
  colorAccent: string;
}

export default async function StdlibPage({
  params,
}: {
  params: Promise<{ locale: string }>;
}) {
  const { locale } = await params;
  const dict = await getDictionary(locale as "en" | "tr");
  const s = dict.Docs.stdlib;

  const modules: StdlibModule[] = [
    {
      id: "matematik",
      file: "matematik.hb",
      icon: "calculate",
      title: "matematik",
      descriptionKey: "matematik",
      colorAccent: "text-tertiary",
      constants: [
        { name: "PI", value: "3.14159265…", description: locale === "tr" ? "Çember sabiti π" : "Circle constant π" },
        {
          name: "E",
          value: "2.71828182…",
          description: locale === "tr" ? "Euler sayısı e" : "Euler's number e",
        },
      ],
      functions: [
        { name: "karesi(n)", signature: "karesi(n: Sayı) → Sayı", description: locale === "tr" ? "n² değerini döndürür" : "Returns n²" },
        { name: "küpü(n)", signature: "küpü(n: Sayı) → Sayı", description: locale === "tr" ? "n³ değerini döndürür" : "Returns n³" },
        { name: "mutlak(n)", signature: "mutlak(n: Sayı) → Sayı", description: locale === "tr" ? "Mutlak değer |n|" : "Absolute value |n|" },
        { name: "kuvvet(a, b)", signature: "kuvvet(a: Sayı, b: Sayı) → Sayı", description: locale === "tr" ? "aᵇ değerini döndürür" : "Returns aᵇ" },
        { name: "yuvarla(n)", signature: "yuvarla(n: Sayı) → Sayı", description: locale === "tr" ? "En yakın tam sayıya yuvarlar" : "Rounds to nearest integer" },
        { name: "faktöriyel(n)", signature: "faktöriyel(n: Sayı) → Sayı", description: locale === "tr" ? "n! (faktöriyel) değerini döndürür" : "Returns n! (factorial)" },
      ],
    },
    {
      id: "renkler",
      file: "renkler.hb",
      icon: "palette",
      title: "renkler",
      descriptionKey: "renkler",
      colorAccent: "text-primary",
      constants: [
        { name: "KIRMIZI", value: "\\x1b[31m", description: locale === "tr" ? "Kırmızı ANSI renk kodu" : "Red ANSI color code" },
        { name: "YEŞİL", value: "\\x1b[32m", description: locale === "tr" ? "Yeşil ANSI renk kodu" : "Green ANSI color code" },
        { name: "SARI", value: "\\x1b[33m", description: locale === "tr" ? "Sarı ANSI renk kodu" : "Yellow ANSI color code" },
        { name: "MAVI", value: "\\x1b[34m", description: locale === "tr" ? "Mavi ANSI renk kodu" : "Blue ANSI color code" },
      ],
      functions: [
        {
          name: "renkli_yaz(metin, renk)",
          signature: "renkli_yaz(metin: Yazı, renk: Yazı)",
          description: locale === "tr" ? "Metni belirtilen ANSI renginde yazdırır" : "Prints text in the specified ANSI color",
        },
        {
          name: "başarı_yaz(m)",
          signature: "başarı_yaz(m: Yazı)",
          description: locale === "tr" ? "Metni yeşil (başarı stili) yazdırır" : "Prints text in green (success style)",
        },
        {
          name: "hata_yaz(m)",
          signature: "hata_yaz(m: Yazı)",
          description: locale === "tr" ? "Metni kırmızı (hata stili) yazdırır" : "Prints text in red (error style)",
        },
        {
          name: "uyarı_yaz(m)",
          signature: "uyarı_yaz(m: Yazı)",
          description: locale === "tr" ? "Metni sarı (uyarı stili) yazdırır" : "Prints text in yellow (warning style)",
        },
      ],
    },
    {
      id: "zaman",
      file: "zaman.hb",
      icon: "schedule",
      title: "zaman",
      descriptionKey: "zaman",
      colorAccent: "text-secondary",
      functions: [
        {
          name: "beklet(saniye)",
          signature: "beklet(saniye: Sayı)",
          description: locale === "tr" ? "Çalışmayı verilen saniye kadar duraklatır" : "Pauses execution for given seconds",
        },
        {
          name: "kronometre_başlat()",
          signature: "kronometre_başlat() → Sayı",
          description: locale === "tr" ? "Mevcut zaman damgasını döndürür (ms)" : "Returns current timestamp (milliseconds)",
        },
        {
          name: "kronometre_bitir(başlangıç)",
          signature: "kronometre_bitir(başlangıç: Sayı) → Sayı",
          description: locale === "tr" ? "Başlangıçtan itibaren geçen ms'yi döndürür" : "Returns elapsed ms since başlangıç",
        },
      ],
    },
    {
      id: "liste",
      file: "liste.hb",
      icon: "filter_list",
      title: "liste",
      descriptionKey: "liste",
      colorAccent: "text-tertiary",
      functions: [
        {
          name: "eşle(liste, f)",
          signature: "eşle(liste: Liste, f: Fonksiyon) → Liste",
          description: locale === "tr" ? "Map — her elemana f uygular, yeni bir liste döndürür" : "Map — applies f to every element, returns new list",
        },
        {
          name: "filtrele(liste, f)",
          signature: "filtrele(liste: Liste, f: Fonksiyon) → Liste",
          description: locale === "tr" ? "Filter — f'in doğru döndüğü elemanları tutar" : "Filter — keeps elements where f returns truthy",
        },
        {
          name: "indirge(liste, f, baş)",
          signature: "indirge(liste: Liste, f: Fonksiyon, baş: Değer) → Değer",
          description: locale === "tr" ? "Reduce — listeyi tek bir değere indirger" : "Reduce — folds list into single value starting from baş",
        },
        {
          name: "ters_cevir(liste)",
          signature: "ters_cevir(liste: Liste) → Liste",
          description: locale === "tr" ? "Listenin ters çevrilmiş kopyasını döndürür" : "Returns a reversed copy of the list",
        },
        {
          name: "içeriyor_mu(liste, eleman)",
          signature: "içeriyor_mu(liste: Liste, eleman: Değer) → Mantıksal",
          description: locale === "tr" ? "Eleman listede mevcutsa 1 döndürür" : "Returns 1 if element exists in the list",
        },
        {
          name: "dilimle(liste, baş, son)",
          signature: "dilimle(liste: Liste, baş: Sayı, son: Sayı) → Liste",
          description: locale === "tr" ? "Listeden alt liste alır" : "Returns a sub-list from start to end index",
        },
      ],
    },
    {
      id: "dosya",
      file: "dosya.hb",
      icon: "folder_open",
      title: "dosya",
      descriptionKey: "dosya",
      colorAccent: "text-primary",
      functions: [
        {
          name: "güvenli_oku(yol)",
          signature: "güvenli_oku(yol: Yazı) → Yazı | hata",
          description: locale === "tr" ? "Dosyayı okur; içerik döndürür veya hata fırlatır" : "Reads file at yol; returns contents or raises an error on failure",
        },
        {
          name: "satırlara_ayır(metin)",
          signature: "satırlara_ayır(metin: Yazı) → Liste",
          description: locale === "tr" ? "Metni satır satır listeye çevirir" : "Splits text into a list of lines",
        },
        {
          name: "dosya_oku_bayt(yol)",
          signature: "dosya_oku_bayt(yol: Yazı) → Bayt",
          description: locale === "tr" ? "Dosyayı ikili (binary) veri olarak okur (Rust built-in)" : "Reads file as binary data — Rust built-in",
        },
      ],
    },
    {
      id: "sistem",
      file: "built-in",
      icon: "settings",
      title: "Sistem",
      descriptionKey: "sistem",
      colorAccent: "text-secondary",
      functions: [
        {
          name: "ortam_değişkeni(anahtar)",
          signature: "ortam_değişkeni(anahtar: Yazı) → Yazı | Boş",
          description: locale === "tr" ? "Ortam değişkeninin değerini döndürür" : "Returns the value of the environment variable",
        },
        {
          name: "sistem(komut)",
          signature: "sistem(komut: Yazı) → Yazı",
          description: locale === "tr" ? "Kabuk komutu çalıştırır ve çıktıyı döndürür" : "Executes a shell command and returns its output",
        },
      ],
    },
    {
      id: "istatistik",
      file: "istatistik.hb",
      icon: "analytics",
      title: "istatistik",
      descriptionKey: "istatistik",
      colorAccent: "text-tertiary",
      functions: [
        {
          name: "ortalama(liste)",
          signature: "ortalama(liste: Liste) → Sayı",
          description: locale === "tr" ? "Aritmetik ortalama hesaplar" : "Calculates arithmetic mean",
        },
        {
          name: "en_büyük(liste)",
          signature: "en_büyük(liste: Liste) → Sayı",
          description: locale === "tr" ? "Listedeki en büyük değeri döndürür" : "Returns the maximum value in the list",
        },
        {
          name: "en_küçük(liste)",
          signature: "en_küçük(liste: Liste) → Sayı",
          description: locale === "tr" ? "Listedeki en küçük değeri döndürür" : "Returns the minimum value in the list",
        },
        {
          name: "varyans(liste)",
          signature: "varyans(liste: Liste) → Sayı",
          description: locale === "tr" ? "Popülasyon varyansını hesaplar" : "Calculates population variance",
        },
        {
          name: "standart_sapma(liste)",
          signature: "standart_sapma(liste: Liste) → Sayı",
          description: locale === "tr" ? "Standart sapmayı hesaplar" : "Calculates standard deviation",
        },
      ],
    },
    {
      id: "dizgi",
      file: "dizgi.hb",
      icon: "text_fields",
      title: "dizgi",
      descriptionKey: "dizgi",
      colorAccent: "text-primary",
      functions: [
        {
          name: "büyük_mü(karakter)",
          signature: "büyük_mü(karakter: Yazı) → Mantıksal",
          description: locale === "tr" ? "Karakterin büyük harf olup olmadığını kontrol eder" : "Checks if character is uppercase",
        },
        {
          name: "küçük_mü(karakter)",
          signature: "küçük_mü(karakter: Yazı) → Mantıksal",
          description: locale === "tr" ? "Karakterin küçük harf olup olmadığını kontrol eder" : "Checks if character is lowercase",
        },
        {
          name: "boşluk_mu(karakter)",
          signature: "boşluk_mu(karakter: Yazı) → Mantıksal",
          description: locale === "tr" ? "Karakterin boşluk, tab veya satır sonu olup olmadığını kontrol eder" : "Checks if character is whitespace",
        },
      ],
    },
    {
      id: "rastgele",
      file: "rastgele.hb",
      icon: "casino",
      title: "rastgele",
      descriptionKey: "rastgele",
      colorAccent: "text-secondary",
      functions: [
        {
          name: "r_tamsayı(min, max)",
          signature: "r_tamsayı(min: Sayı, max: Sayı) → Sayı",
          description: locale === "tr" ? "Belirtilen aralıkta rastgele tam sayı üretir" : "Generates random integer in range [min, max]",
        },
        {
          name: "r_seç(liste)",
          signature: "r_seç(liste: Liste) → Değer",
          description: locale === "tr" ? "Listeden rastgele eleman seçer" : "Picks a random element from the list",
        },
        {
          name: "r_karıştır(liste)",
          signature: "r_karıştır(liste: Liste) → Liste",
          description: locale === "tr" ? "Listeyi rastgele sırayla karıştırır" : "Shuffles the list randomly",
        },
      ],
    },
    {
      id: "birim_test",
      file: "birim_test.hb",
      icon: "bug_report",
      title: "birim_test",
      descriptionKey: "birim_test",
      colorAccent: "text-tertiary",
      functions: [
        {
          name: "test_et(ad, f)",
          signature: "test_et(ad: Yazı, f: Fonksiyon) → Boş",
          description: locale === "tr" ? "Test fonksiyonunu çalıştırır ve sonucu raporlar" : "Runs test function and reports result",
        },
        {
          name: "iddia_et(beklenen, gelen, mesaj)",
          signature: "iddia_et(beklenen: Değer, gelen: Değer, mesaj: Yazı) → Mantıksal",
          description: locale === "tr" ? "İki değeri karşılaştırır, eşitse 1 döndürür" : "Asserts two values are equal, returns 1 if true",
        },
        {
          name: "test_raporu()",
          signature: "test_raporu() → Boş",
          description: locale === "tr" ? "Toplam test sonuçlarının özetini yazdırır" : "Prints summary of all test results",
        },
      ],
    },
    {
      id: "yapay_zeka",
      file: "yapay_zeka_temel.hb",
      icon: "psychology",
      title: "yapay_zeka",
      descriptionKey: "yapay_zeka",
      colorAccent: "text-primary",
      functions: [
        {
          name: "bpe_tokenizer_oluştur(sözlük)",
          signature: "bpe_tokenizer_oluştur(sözlük: Sözlük) → Tokenizer",
          description: locale === "tr" ? "Rust yerel BPE Tokenizer motorunu başlatır" : "Initializes native Rust BPE Tokenizer engine",
        },
        {
          name: "otomatik_türev_grafı()",
          signature: "otomatik_türev_grafı() → AutogradGraph",
          description: locale === "tr" ? "Otomatik türev hesaplama grafı (Autograd) oluşturur" : "Creates automatic differentiation computation graph (Autograd)",
        },
        {
          name: "sinir_ağı_katmanı(giriş, çıkış)",
          signature: "sinir_ağı_katmanı(giriş: Sayı, çıkış: Sayı) → Katman",
          description: locale === "tr" ? "Ağırlıkları rastgele ilklendirilmiş katman oluşturur" : "Creates dense neural network layer with initialized weights",
        },
      ],
    },
    {
      id: "ffi",
      file: "ffi.rs",
      icon: "terminal",
      title: "FFI (C/C++/CUDA)",
      descriptionKey: "ffi",
      colorAccent: "text-secondary",
      functions: [
        {
          name: "ffi_yükle(kütüphane_yolu)",
          signature: "ffi_yükle(kütüphane_yolu: Yazı) → FFIDomain",
          description: locale === "tr" ? "Dinamik C/C++/CUDA kütüphanesini (.so/.dylib/.dll) yükler" : "Loads dynamic C/C++/CUDA shared library (.so/.dylib/.dll)",
        },
        {
          name: "ffi_çağır(domain, sembol, argümanlar)",
          signature: "ffi_çağır(domain: FFIDomain, sembol: Yazı, args: Liste) → Değer",
          description: locale === "tr" ? "Yerel C/CUDA sembolünü doğrudan çağırır" : "Calls native C/CUDA symbol directly at native speeds",
        },
      ],
    },
  ];

  const getPath = (path: string) => `/${locale}${path}`;

  return (
    <>
      <main className="flex-1 px-8 md:px-16 py-12 max-w-4xl">
        <nav className="flex gap-2 text-[10px] uppercase tracking-widest text-on-surface-variant/60 mb-4">
          <Link href={getPath("/docs")} className="hover:text-primary transition-colors">
            {dict.Nav.docs}
          </Link>
          <span>/</span>
          <span className="text-on-surface-variant">{dict.Sidebar.core_libraries}</span>
          <span>/</span>
          <span className="text-primary">{s.title}</span>
        </nav>

        <h1 className="text-5xl font-extrabold text-on-surface tracking-tighter mb-6">
          {s.hero_title}
        </h1>
        <p className="text-lg text-on-surface-variant leading-relaxed mb-4">
          {s.hero_desc}
        </p>
        
        <div className="bg-primary/5 border border-primary/10 rounded-lg p-6 mb-12">
          <p className="text-sm text-on-surface-variant mb-4 font-medium">
            {s.import_info}
          </p>
          <div className="bg-surface-container-lowest rounded border border-outline-variant/10 p-4 font-mono text-xs text-primary">
            yükle &quot;matematik.hb&quot;
          </div>
        </div>

        {/* Modules */}
        <div className="space-y-24">
          {modules.map((mod) => (
            <section key={mod.id} id={mod.id} className="scroll-mt-24">
              {/* Module header */}
              <div className="flex items-center gap-6 mb-8 pb-6 border-b border-outline-variant/10">
                <div className="w-12 h-12 flex items-center justify-center bg-surface-container-high rounded-lg shadow-sm">
                  <span className={`material-symbols-outlined text-2xl ${mod.colorAccent}`}>
                    {mod.icon}
                  </span>
                </div>
                <div>
                  <h2 className="text-3xl font-extrabold text-on-surface tracking-tight font-mono">
                    {mod.file}
                  </h2>
                  <p className="text-sm text-on-surface-variant mt-1 font-medium">
                    {(s.modules as any)[mod.descriptionKey]}
                  </p>
                </div>
              </div>

              {/* Constants */}
              {mod.constants && mod.constants.length > 0 && (
                <div className="mb-10">
                  <h3 className="text-[10px] font-bold text-on-surface-variant/60 uppercase tracking-[0.2em] mb-4">
                    {locale === "tr" ? "SABİTLER" : "CONSTANTS"}
                  </h3>
                  <div className="grid grid-cols-1 sm:grid-cols-2 gap-4">
                    {mod.constants.map((c) => (
                      <div
                        key={c.name}
                        className="bg-surface-container-low rounded-lg p-5 border border-outline-variant/10 hover:border-primary/20 transition-colors"
                      >
                        <div className="font-mono text-sm text-primary font-bold mb-1">
                          {c.name}
                        </div>
                        <div className="font-mono text-[11px] text-tertiary mb-3 opacity-80 uppercase tracking-tighter">
                          = {c.value}
                        </div>
                        <div className="text-xs text-on-surface-variant leading-relaxed">
                          {c.description}
                        </div>
                      </div>
                    ))}
                  </div>
                </div>
              )}

              {/* Functions */}
              <h3 className="text-[10px] font-bold text-on-surface-variant/60 uppercase tracking-[0.2em] mb-4">
                {locale === "tr" ? "FONKSİYONLAR" : "FUNCTIONS"}
              </h3>
              <div className="space-y-4">
                {mod.functions.map((fn) => (
                  <div
                    key={fn.name}
                    className="bg-surface-container-lowest rounded-lg border border-outline-variant/10 overflow-hidden hover:border-primary/20 transition-colors group"
                  >
                    <div className="bg-surface-container-low px-5 py-3 border-b border-outline-variant/5">
                      <code className="font-mono text-sm text-primary-fixed group-hover:text-primary transition-colors font-bold">
                        {fn.signature}
                      </code>
                    </div>
                    <div className="px-5 py-4 text-sm text-on-surface-variant leading-relaxed">
                      {fn.description}
                    </div>
                  </div>
                ))}
              </div>
            </section>
          ))}
        </div>

        {/* Navigation */}
        <div className="flex justify-between mt-24 pt-8 border-t border-outline-variant/10">
          <Link
            href={getPath("/docs/lists-errors")}
            className="flex items-center gap-2 text-sm text-on-surface-variant hover:text-primary transition-colors font-bold uppercase tracking-widest text-[10px]"
          >
            <span className="material-symbols-outlined text-base">
              arrow_back
            </span>
            {dict.Docs.lists_errors.title}
          </Link>
          <Link
            href={getPath("/docs/changelog")}
            className="flex items-center gap-2 text-sm text-on-surface-variant hover:text-primary transition-colors font-bold uppercase tracking-widest text-[10px]"
          >
            {dict.Sidebar.items.changelog}
            <span className="material-symbols-outlined text-base">
              arrow_forward
            </span>
          </Link>
        </div>
      </main>

      {/* Right TOC */}
      <aside className="hidden xl:block w-64 sticky top-16 h-[calc(100vh-4rem)] py-12 px-8 overflow-y-auto border-l border-outline-variant/10 shrink-0">
        <h5 className="text-[10px] font-bold text-on-surface uppercase tracking-[0.2em] mb-6 opacity-40">
          {locale === "tr" ? "MODÜLLER" : "MODULES"}
        </h5>
        <ul className="space-y-3 text-[11px] font-bold uppercase tracking-widest">
          {modules.map((mod) => (
            <li key={mod.id}>
              <a
                href={`#${mod.id}`}
                className="flex items-center gap-3 text-on-surface-variant/60 hover:text-primary transition-all group"
              >
                <span className="material-symbols-outlined text-xs group-hover:scale-110 transition-transform">
                  {mod.icon}
                </span>
                <span className="font-mono">{mod.file}</span>
              </a>
            </li>
          ))}
        </ul>
      </aside>
    </>
  );
}
