import Link from "next/link";
import CodeBlock from "@/components/CodeBlock";
import type { Metadata } from "next";
import { getDictionary } from "@/dictionaries/dictionaries";

export const metadata: Metadata = {
  title: "Functions & Classes",
  description:
    "Define reusable logic with Hüma functions and model data with classes.",
};

const functionCode = `// Fonksiyon Tanımlama
topla fonksiyon olsun a, b alsın {
    a + b'yi döndür
}

// Fonksiyon çağırma
sonuç = topla(5, 10) olsun
"Sonuç: " + sonuç'u yazdır;`;

const classCode = `// Sınıf Tanımlama
araç sınıf olsun {
    hız = 0 olsun

    hızlan fonksiyon olsun miktar alsın {
        kendisi'nin hız'ı = kendisi'nin hız'ı + miktar olsun
    }

    hız_göster fonksiyon olsun {
        "Mevcut hız: " + kendisi'nin hız'ı yazdır;
    }
}

// Nesne oluşturma
araba = araç() olsun
araba.hızlan(10)
araba.hız_göster()`;

export default async function FunctionsClassesPage({
  params,
}: {
  params: Promise<{ locale: string }>;
}) {
  const { locale } = await params;
  const dict = await getDictionary(locale as "en" | "tr");
  const f = dict.Docs.functions_classes;

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
          <span className="text-primary">{f.title}</span>
        </nav>

        <h1 className="text-5xl font-extrabold text-on-surface tracking-tighter mb-6">
          {f.hero_title}
        </h1>
        <p className="text-lg text-on-surface-variant leading-relaxed mb-12">
          {f.hero_desc}
        </p>

        {/* Functions */}
        <section className="mb-16" id="functions">
          <h2 className="text-2xl font-bold text-on-surface mb-6 flex items-center gap-3">
            <span className="w-8 h-8 rounded-full bg-surface-container-high flex items-center justify-center text-sm font-mono text-primary">
              01
            </span>
            {f.functions.title}
          </h2>
          <p className="mb-6 text-on-surface-variant leading-relaxed">
            {f.functions.desc}
          </p>
          <CodeBlock code={functionCode} filename="fonksiyonlar.hb" />

          {/* Keyword table */}
          <div className="overflow-x-auto mt-8 bg-surface-container-low rounded-lg border border-outline-variant/10 p-6">
            <table className="w-full text-sm border-collapse">
              <thead>
                <tr className="border-b border-outline-variant/20">
                  <th className="text-left py-3 pr-8 text-on-surface-variant/60 font-bold text-[10px] uppercase tracking-widest">
                    Token
                  </th>
                  <th className="text-left py-3 pr-8 text-on-surface-variant/60 font-bold text-[10px] uppercase tracking-widest">
                    Meaning
                  </th>
                  <th className="text-left py-3 text-on-surface-variant/60 font-bold text-[10px] uppercase tracking-widest">
                    Role
                  </th>
                </tr>
              </thead>
              <tbody className="font-mono text-xs divide-y divide-outline-variant/10">
                {[
                  ["fonksiyon olsun", "function", locale === "tr" ? "Fonksiyon tanımlar" : "Declares a function"],
                  ["alsın", "takes / params", locale === "tr" ? "Parametre listesini başlatır" : "Introduces parameter list"],
                  ["döndür", "return", locale === "tr" ? "Değer döndürür" : "Returns a value"],
                ].map(([token, meaning, role]) => (
                  <tr key={token}>
                    <td className="py-4 pr-8 text-primary font-bold">{token}</td>
                    <td className="py-4 pr-8 text-tertiary">{meaning}</td>
                    <td className="py-4 text-on-surface-variant font-body">
                      {role}
                    </td>
                  </tr>
                ))}
              </tbody>
            </table>
          </div>
        </section>

        {/* Classes */}
        <section className="mb-16" id="classes">
          <h2 className="text-2xl font-bold text-on-surface mb-6 flex items-center gap-3">
            <span className="w-8 h-8 rounded-full bg-surface-container-high flex items-center justify-center text-sm font-mono text-primary">
              02
            </span>
            {f.classes.title}
          </h2>
          <p className="mb-6 text-on-surface-variant leading-relaxed">
            {f.classes.desc}
          </p>
          <CodeBlock code={classCode} filename="siniflar.hb" />

          {/* Info callout */}
          <div className="bg-tertiary/5 border-l-4 border-tertiary p-6 rounded-r-lg mt-8">
            <div className="flex items-center gap-3 mb-2 text-tertiary">
              <span className="material-symbols-outlined text-lg">info</span>
              <span className="text-xs font-bold uppercase tracking-widest">
                {locale === "tr" ? "Metot ve Kendisi" : "Methods & Self"}
              </span>
            </div>
            <p className="text-sm text-on-surface-variant leading-relaxed">
              {locale === "tr" 
                ? "Sınıf içindeki fonksiyonlar o sınıfın metotlarıdır. 'kendisi' (self) anahtar kelimesi ile nesne özelliklerine erişebilirler."
                : "Functions inside a class are its methods. They can access instance properties using the 'kendisi' (self) keyword."}
            </p>
          </div>
        </section>

        {/* Navigation */}
        <div className="flex justify-between mt-16 pt-8 border-t border-outline-variant/10">
          <Link
            href={getPath("/docs/syntax")}
            className="flex items-center gap-2 text-sm text-on-surface-variant hover:text-primary transition-colors font-bold uppercase tracking-widest text-[10px]"
          >
            <span className="material-symbols-outlined text-base">
              arrow_back
            </span>
            {dict.Docs.syntax.title}
          </Link>
          <Link
            href={getPath("/docs/lists-errors")}
            className="flex items-center gap-2 text-sm text-on-surface-variant hover:text-primary transition-colors font-bold uppercase tracking-widest text-[10px]"
          >
            {dict.Sidebar.items.lists}
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
            { href: "#functions", label: f.functions.title },
            { href: "#classes", label: f.classes.title },
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
