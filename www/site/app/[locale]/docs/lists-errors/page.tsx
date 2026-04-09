import Link from "next/link";
import CodeBlock from "@/components/CodeBlock";
import type { Metadata } from "next";
import { getDictionary } from "@/dictionaries/dictionaries";

export const metadata: Metadata = {
  title: "Data Structures & Error Handling",
  description:
    "Working with lists, dictionaries and handling runtime errors in Hüma using dene/yakala.",
};

const listsCode = `// Liste Oluşturma
sayılar = [1, 2, 3] olsun

// Eleman ekleme
sayılar'a [5]'i ekle;

// Elemana erişim (indeks ile)
// Removing element at index 0
sayılar'dan [0]'ı çıkar;

// Listeyi yazdır
sayılar'ı yazdır;`;

const errorCode = `// Hata Yönetimi (dene / yakala)
dene {
    sonuç = 10 / 0 olsun
} yakala hata_mesajı {
    "Hata yakalandı: " + hata_mesajı'nı yazdır;
}`;

const mapCode = `// Sözlük (Dictionary) Oluşturma
ayarlar = {
    "tema": "koyu",
    "dil": "tr",
    "sürüm": 1.0
} olsun

// Değere erişim (Ek sistemi ile)
yazdır ayarlar'ın tema'sı;

// Değere erişim (Metot ile)
yazdır ayarlar.getir("dil");

// Değer güncelleme
ayarlar.ayarla("sürüm", 2.0);`;

const combinedCode = `// Listeler ve Hata Yönetimi Birlikte
sayılar = [10, 20, 30] olsun

dene {
    // İndeks sınır dışı olabilir
    sayılar'dan [10]'u çıkar;
} yakala hata {
    "İşlem başarısız: " + hata'yı yazdır;
}

sayılar'a [99]'u ekle;
sayılar'ı yazdır;`;

export default async function ListsErrorsPage({
  params,
}: {
  params: Promise<{ locale: string }>;
}) {
  const { locale } = await params;
  const dict = await getDictionary(locale as "en" | "tr");
  const l = dict.Docs.lists_errors;

  const getPath = (path: string) => `/${locale}${path}`;

  return (
    <>
      <main className="flex-1 px-8 md:px-16 py-12 max-w-4xl">
        <nav className="flex gap-2 text-[10px] uppercase tracking-widest text-on-surface-variant/60 mb-4">
          <Link href={getPath("/docs")} className="hover:text-primary transition-colors">
            {dict.Nav.docs}
          </Link>
          <span>/</span>
          <span className="text-on-surface-variant">{dict.Sidebar.language}</span>
          <span>/</span>
          <span className="text-primary">{l.title}</span>
        </nav>

        <h1 className="text-5xl font-extrabold text-on-surface tracking-tighter mb-6">
          {l.hero_title}
        </h1>
        <p className="text-lg text-on-surface-variant leading-relaxed mb-12">
          {l.hero_desc}
        </p>

        {/* Lists */}
        <section className="mb-16" id="lists">
          <h2 className="text-2xl font-bold text-on-surface mb-6 flex items-center gap-3">
            <span className="w-8 h-8 rounded-full bg-surface-container-high flex items-center justify-center text-sm font-mono text-primary">
              01
            </span>
            {l.lists.title}
          </h2>
          <p className="mb-6 text-on-surface-variant leading-relaxed">
            {l.lists.desc}
          </p>
          <CodeBlock code={listsCode} filename="listeler.hb" />

          <h3 className="text-lg font-bold text-on-surface mb-4 mt-8">Sözlükler (Dictionaries)</h3>
          <p className="mb-6 text-on-surface-variant leading-relaxed text-sm">
            {locale === "tr" 
              ? "Sözlükler, anahtar-değer çiftlerini saklamak için kullanılır. Süslü parantez içine 'anahtar': değer formatında yazılırlar."
              : "Dictionaries are used to store key-value pairs. They are written in 'key': value format inside curly braces."}
          </p>
          <CodeBlock code={mapCode} filename="sozlukler.hb" />

          {/* Operation table */}
          <div className="overflow-x-auto mt-8 bg-surface-container-low rounded-lg border border-outline-variant/10 p-6">
            <table className="w-full text-sm border-collapse">
              <thead>
                <tr className="border-b border-outline-variant/20">
                  <th className="text-left py-3 pr-8 text-on-surface-variant/60 font-bold text-[10px] uppercase tracking-widest">
                    {locale === "tr" ? "Veri Yapısı" : "Data Structure"}
                  </th>
                  <th className="text-left py-3 pr-8 text-on-surface-variant/60 font-bold text-[10px] uppercase tracking-widest">
                    Syntax
                  </th>
                  <th className="text-left py-3 text-on-surface-variant/60 font-bold text-[10px] uppercase tracking-widest">
                    Description
                  </th>
                </tr>
              </thead>
              <tbody className="font-mono text-xs divide-y divide-outline-variant/10">
                {[
                  ["Liste", "L = [1, 2] olsun", locale === "tr" ? "İndeks tabanlı sıralı dizi" : "Index-based ordered array"],
                  ["Sözlük", "S = { 'a': 1 } olsun", locale === "tr" ? "Anahtar-değer tabanlı harita" : "Key-value based map"],
                  ["Erişim (Sözlük)", "S'nin anahtar'ı", locale === "tr" ? "Ek sistemi ile değer okuma" : "Read value via suffix"],
                  ["Metot (Sözlük)", "S.getir('a')", locale === "tr" ? "Fonksiyon ile değer okuma" : "Read value via method"],
                ].map(([op, syntax, desc]) => (
                  <tr key={op}>
                    <td className="py-4 pr-8 text-tertiary font-bold">{op}</td>
                    <td className="py-4 pr-8 text-primary font-bold">{syntax}</td>
                    <td className="py-4 text-on-surface-variant font-body">
                      {desc}
                    </td>
                  </tr>
                ))}
              </tbody>
            </table>
          </div>
        </section>

        {/* Error Handling */}
        <section className="mb-16" id="errors">
          <h2 className="text-2xl font-bold text-on-surface mb-6 flex items-center gap-3">
            <span className="w-8 h-8 rounded-full bg-surface-container-high flex items-center justify-center text-sm font-mono text-primary">
              02
            </span>
            {l.errors.title}
          </h2>
          <p className="mb-6 text-on-surface-variant leading-relaxed">
            {l.errors.desc}
          </p>
          <CodeBlock code={errorCode} filename="hata_yonetimi.hb" />

          <div className="bg-primary/5 border-l-4 border-primary p-6 rounded-r-lg mt-8">
            <div className="flex items-center gap-3 mb-2 text-primary">
              <span className="material-symbols-outlined text-lg">info</span>
              <span className="text-xs font-bold uppercase tracking-widest">
                {locale === "tr" ? "Güvenli Çalışma" : "Safe Execution"}
              </span>
            </div>
            <p className="text-sm text-on-surface-variant leading-relaxed">
              {locale === "tr" 
                ? "Sıfıra bölme veya sınır dışı indeks erişimi gibi kritik hatalar 'dene' bloğu içinde yakalanarak programın çökmesini engeller."
                : "Critical errors like division by zero or index out of bounds are caught within the 'dene' block, preventing program crashes."}
            </p>
          </div>
        </section>

        {/* Navigation */}
        <div className="flex justify-between mt-16 pt-8 border-t border-outline-variant/10">
          <Link
            href={getPath("/docs/functions-classes")}
            className="flex items-center gap-2 text-sm text-on-surface-variant hover:text-primary transition-colors font-bold uppercase tracking-widest text-[10px]"
          >
            <span className="material-symbols-outlined text-base">
              arrow_back
            </span>
            {dict.Docs.functions_classes.title}
          </Link>
          <Link
            href={getPath("/docs/stdlib")}
            className="flex items-center gap-2 text-sm text-on-surface-variant hover:text-primary transition-colors font-bold uppercase tracking-widest text-[10px]"
          >
            {dict.Sidebar.items.stdlib}
            <span className="material-symbols-outlined text-base">
              arrow_forward
            </span>
          </Link>
        </div>
      </main>

      {/* Right TOC */}
      <aside className="hidden xl:block w-64 sticky top-16 h-[calc(100vh-4rem)] py-12 px-8 overflow-y-auto border-l border-outline-variant/10 shrink-0">
        <h5 className="text-[10px] font-bold text-on-surface uppercase tracking-[0.2em] mb-6 opacity-40">
          {locale === "tr" ? "BU SAYFADA" : "ON THIS PAGE"}
        </h5>
        <ul className="space-y-4 text-[11px] font-bold uppercase tracking-widest">
          {[
            { href: "#lists", label: l.lists.title },
            { href: "#errors", label: l.errors.title },
          ].map((item) => (
            <li key={item.href}>
              <a
                href={item.href}
                className="text-on-surface-variant/60 hover:text-primary transition-all block"
              >
                {item.label}
              </a>
            </li>
          ))}
        </ul>
      </aside>
    </>
  );
}
