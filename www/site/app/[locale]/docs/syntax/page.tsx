import Link from "next/link";
import CodeBlock from "@/components/CodeBlock";
import type { Metadata } from "next";
import { getDictionary } from "@/dictionaries/dictionaries";

export async function generateMetadata({
  params,
}: {
  params: Promise<{ locale: string }>;
}): Promise<Metadata> {
  const { locale } = await params;
  const dict = await getDictionary(locale as "en" | "tr");
  return {
    title: dict.Docs.syntax.title,
    description: dict.Docs.syntax.description,
  };
}

const variablesCode = `// Değişken Tanımlama (Variable Declaration)
isim = "Hüma" olsun // 'olsun' is an assignment operator
sayı = 10 olsun     // Type determined by value

// Ek sistemi kullanımı (Suffix System)
// Apostrophe (') is used to separate grammatical suffixes
isim'i yazdır;  // The compiler strips 'i
sayı'yı yazdır; // The compiler strips 'yı`;

const conditionCode = `// Koşul Blokları (Conditionals)
puan = 85 olsun

puan > 50 ise {
    "Başarılı!"'yı yazdır;
} yoksa {
    "Tekrar dene."'i yazdır;
}`;

const loopCode = `// Döngüler (olduğu sürece — while)
sayaç = 0 olsun

sayaç < 5 olduğu sürece {
    "Sıra: " + sayaç'ı yazdır;
    sayaç = sayaç + 1 olsun
}

// Aralık Döngüleri (Range Loops)
i = 0'dan 5'e kadar {
    "Adım: " + i'yi yazdır;
}`;
export default async function SyntaxPage({
  params,
}: {
  params: Promise<{ locale: string }>;
}) {
  const { locale } = await params;
  const dict = await getDictionary(locale as "en" | "tr");
  const s = dict.Docs.syntax;

  const getPath = (path: string) => `/${locale}${path}`;

  return (
    <>
      <main className="flex-1 px-8 md:px-16 py-12 max-w-4xl">
        {/* Breadcrumb */}
        <nav className="flex gap-2 text-[10px] uppercase tracking-widest text-on-surface-variant/60 mb-4">
          <Link href={getPath("/docs")} className="hover:text-primary transition-colors">
            {dict.Nav.docs}
          </Link>
          <span>/</span>
          <span className="text-on-surface-variant">{dict.Sidebar.language}</span>
          <span>/</span>
          <span className="text-primary">{s.title}</span>
        </nav>

        <h1 className="text-5xl font-extrabold text-on-surface tracking-tighter mb-6">
          {s.hero_title}
        </h1>
        <p className="text-lg text-on-surface-variant leading-relaxed mb-12">
          {s.hero_desc}
        </p>

        {/* Suffix System callout */}
        <div className="bg-tertiary/5 border-l-4 border-tertiary p-6 rounded-r-lg mb-12">
          <div className="flex items-center gap-3 mb-2 text-tertiary">
            <span className="material-symbols-outlined text-lg">auto_fix</span>
            <span className="text-xs font-bold uppercase tracking-widest">
              {s.suffix_system.title}
            </span>
          </div>
          <p className="text-sm text-on-surface-variant leading-relaxed">
            {s.suffix_system.desc}
          </p>
        </div>
        <div className="bg-primary/5 border-l-4 border-primary p-6 rounded-r-lg mb-12">
          <div className="flex items-center gap-3 mb-2 text-primary">
            <span className="material-symbols-outlined text-lg">rule</span>
            <span className="text-xs font-bold uppercase tracking-widest">
              {locale === "tr" ? "Sözdizimi Kapsam Notu" : "Syntax Scope Note"}
            </span>
          </div>
          <p className="text-sm text-on-surface-variant leading-relaxed">
            {locale === "tr"
              ? "Ekli yazımların temel amacı okunabilirliktir. Çekirdek anlam, lexer sonrası normalize edilen kök tokenlar üzerinden korunur. Bu nedenle örneklerdeki doğal yazım ile normalize biçim eşdeğer kabul edilir."
              : "Suffix-aware forms primarily improve readability. Core semantics are preserved on normalized root tokens after lexing. Natural and normalized forms in examples are therefore treated as equivalent."}
          </p>
        </div>

        {/* Variables */}
        <section className="mb-16" id="variables">
          <h2 className="text-2xl font-bold text-on-surface mb-6 flex items-center gap-3">
            <span className="w-8 h-8 rounded-full bg-surface-container-high flex items-center justify-center text-sm font-mono text-primary">
              01
            </span>
            {s.variables.title}
          </h2>
          <p className="mb-6 text-on-surface-variant leading-relaxed">
            {s.variables.desc}
          </p>
          <CodeBlock code={variablesCode} filename="degiskenler.hb" />

          {/* Keyword table */}
          <div className="overflow-x-auto mt-8 bg-surface-container-low rounded-lg border border-outline-variant/10 p-6">
            <table className="w-full text-sm border-collapse">
              <thead>
                <tr className="border-b border-outline-variant/20">
                  <th className="text-left py-3 pr-8 text-on-surface-variant/60 font-bold text-[10px] uppercase tracking-widest">
                    Hüma
                  </th>
                  <th className="text-left py-3 pr-8 text-on-surface-variant/60 font-bold text-[10px] uppercase tracking-widest">
                    Equivalent
                  </th>
                  <th className="text-left py-3 text-on-surface-variant/60 font-bold text-[10px] uppercase tracking-widest">
                    Purpose
                  </th>
                </tr>
              </thead>
              <tbody className="font-mono text-xs divide-y divide-outline-variant/10">
                {[
                  ["olsun", "let / var", locale === "tr" ? "Değişken tanımla veya güncelle" : "Declare or update a variable"],
                  ["yazdır", "print()", locale === "tr" ? "Değeri standart çıktıya bas" : "Output a value to stdout"],
                  ["döndür", "return", locale === "tr" ? "Fonksiyondan değer döndür" : "Return from a function"],
                  ["ile", "with", locale === "tr" ? "Edatlı fonksiyon çağrısı (infix-call)" : "Particle-based function call"],
                ].map(([huma, eq, desc]) => (
                  <tr key={huma} className="group">
                    <td className="py-4 pr-8 text-primary font-bold">{huma}</td>
                    <td className="py-4 pr-8 text-tertiary">{eq}</td>
                    <td className="py-4 text-on-surface-variant font-body">
                      {desc}
                    </td>
                  </tr>
                ))}
              </tbody>
            </table>
          </div>
        </section>

        {/* Conditionals */}
        <section className="mb-16" id="conditionals">
          <h2 className="text-2xl font-bold text-on-surface mb-6 flex items-center gap-3">
            <span className="w-8 h-8 rounded-full bg-surface-container-high flex items-center justify-center text-sm font-mono text-primary">
              02
            </span>
            {s.conditionals.title}
          </h2>
          <p className="mb-6 text-on-surface-variant leading-relaxed">
            {s.conditionals.desc}
          </p>
          <CodeBlock code={conditionCode} filename="kosul.hb" />
        </section>

        {/* Loops */}
        <section className="mb-16" id="loops">
          <h2 className="text-2xl font-bold text-on-surface mb-6 flex items-center gap-3">
            <span className="w-8 h-8 rounded-full bg-surface-container-high flex items-center justify-center text-sm font-mono text-primary">
              03
            </span>
            {s.loops.title}
          </h2>
          <p className="mb-6 text-on-surface-variant leading-relaxed">
            {s.loops.desc}
          </p>
          <CodeBlock code={loopCode} filename="dongu.hb" />

          {/* Warning callout */}
          <div className="bg-primary/5 border-l-4 border-primary p-6 rounded-r-lg mt-8">
            <div className="flex items-center gap-3 mb-2 text-primary">
              <span className="material-symbols-outlined text-lg">warning</span>
              <span className="text-xs font-bold uppercase tracking-widest">
                {locale === "tr" ? "Döngü Güvenliği" : "Loop Guard"}
              </span>
            </div>
            <p className="text-sm text-on-surface-variant leading-relaxed">
              {locale === "tr" 
                ? "Döngü değişkeninin gövde içinde güncellendiğinden emin olun. Hüma'nın çalışma zamanı, katı modda bazı sonsuz döngüleri tespit edebilir."
                : "Always ensure the loop variable is mutated inside the body. Hüma's runtime can detect some infinite loops in strict mode."}
            </p>
          </div>
        </section>

        {/* Particle / Postfix Calls */}
        <section className="mb-16" id="postfix">
          <h2 className="text-2xl font-bold text-on-surface mb-6 flex items-center gap-3">
            <span className="w-8 h-8 rounded-full bg-surface-container-high flex items-center justify-center text-sm font-mono text-primary">
              04
            </span>
            {s.postfix_calls.title}
          </h2>
          <p className="mb-6 text-on-surface-variant leading-relaxed">
            {s.postfix_calls.desc}
          </p>
          <div className="grid grid-cols-1 md:grid-cols-2 gap-4">
            <div className="bg-surface-container-low p-4 rounded-xl border border-outline-variant/10">
              <p className="text-[10px] font-bold text-primary uppercase tracking-widest mb-2">Edatlı Çağrı (ile)</p>
              <code className="text-xs font-mono bg-on-surface/5 p-2 rounded block">{"10 ile 20'yi topla"}</code>
            </div>
            <div className="bg-surface-container-low p-4 rounded-xl border border-outline-variant/10">
              <p className="text-[10px] font-bold text-primary uppercase tracking-widest mb-2">Postfix Yükle</p>
              <code className="text-xs font-mono bg-on-surface/5 p-2 rounded block">{`"mat.hb"'yi yükle`}</code>
            </div>
          </div>
        </section>

        {/* Navigation */}
        <div className="flex justify-between mt-16 pt-8 border-t border-outline-variant/10">
          <Link
            href={getPath("/docs")}
            className="flex items-center gap-2 text-sm text-on-surface-variant hover:text-primary transition-colors font-bold uppercase tracking-widest text-[10px]"
          >
            <span className="material-symbols-outlined text-base">
              arrow_back
            </span>
            {dict.Docs.getting_started.title}
          </Link>
          <Link
            href={getPath("/docs/functions-classes")}
            className="flex items-center gap-2 text-sm text-on-surface-variant hover:text-primary transition-colors font-bold uppercase tracking-widest text-[10px]"
          >
            {dict.Sidebar.items.functions}
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
            { href: "#variables", label: s.variables.title },
            { href: "#conditionals", label: s.conditionals.title },
            { href: "#loops", label: s.loops.title },
            { href: "#postfix", label: s.postfix_calls.title },
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
