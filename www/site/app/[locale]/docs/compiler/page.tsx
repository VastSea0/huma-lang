import Link from "next/link";
import CodeBlock from "@/components/CodeBlock";
import type { Metadata } from "next";
import { getDictionary } from "@/dictionaries/dictionaries";

export const metadata: Metadata = {
  title: "Compiler Internals",
  description:
    "Deep-dive into the Hüma compiler pipeline: lexer, suffix system, AST, and interpreter.",
};

const lexerExampleCode = `// Kullanıcının yazdığı kod:
isim'i yazdır;

// Hüma Lexer'ı eki temizledikten sonra:
// Token akışı:  IDENT("isim")  BUILTIN("yazdır")  SEMI`;

const astCode = `// Kaynak:
topla fonksiyon olsun a, b alsın {
    a + b'yi döndür
}

// Basitleştirilmiş AST Yapısı:
FunctionDecl {
  name: "topla",
  params: ["a", "b"],
  body: [
    ReturnStmt {
      value: BinaryExpr {
        op: "+",
        left:  Ident("a"),
        right: Ident("b")
      }
    }
  ]
}`;

const cargoCode = `# Hüma derleyicisini kaynaktan derleyin
cargo build --release

# Çalıştırılabilir dosya buradadır:
./target/release/huma

# Bir dosyayı çalıştırın:
./target/release/huma calistir ornek.hb`;

export default async function CompilerPage({
  params,
}: {
  params: Promise<{ locale: string }>;
}) {
  const { locale } = await params;
  const dict = await getDictionary(locale as "en" | "tr");
  const c = dict.Docs.compiler;

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
          <span className="text-on-surface-variant">{dict.Sidebar.tooling}</span>
          <span>/</span>
          <span className="text-primary">{c.title}</span>
        </nav>

        <h1 className="text-5xl font-extrabold text-on-surface tracking-tighter mb-6">
          {c.hero_title}
        </h1>
        <p className="text-lg text-on-surface-variant leading-relaxed mb-12">
          {c.hero_desc}
        </p>

        {/* Pipeline overview */}
        <section className="mb-16" id="pipeline">
          <h2 className="text-2xl font-bold text-on-surface mb-6">
            {c.pipeline.title}
          </h2>

          <div className="grid grid-cols-1 sm:grid-cols-4 gap-px bg-outline-variant/10 rounded-xl overflow-hidden mb-8 border border-outline-variant/10 shadow-sm">
            {[
              { step: "01", name: c.pipeline.steps.source, icon: "draft", desc: ".hb (UTF-8)" },
              { step: "02", name: c.pipeline.steps.lexer, icon: "text_fields", desc: "Tokens & Suffixes" },
              { step: "03", name: c.pipeline.steps.parser, icon: "account_tree", desc: "AST Builder" },
              { step: "04", name: c.pipeline.steps.interpreter, icon: "play_arrow", desc: "Tree-walk" },
            ].map((p) => (
              <div
                key={p.step}
                className="bg-surface-container-lowest p-6 flex flex-col items-center text-center gap-3 hover:bg-surface-container-low transition-colors"
              >
                <span className="material-symbols-outlined text-primary text-2xl">
                  {p.icon}
                </span>
                <div>
                  <div className="font-mono text-[9px] font-bold text-on-surface-variant/40 mb-1">
                    STEP {p.step}
                  </div>
                  <div className="font-bold text-xs text-on-surface uppercase tracking-wider">{p.name}</div>
                  <div className="text-[10px] text-on-surface-variant mt-1 font-medium">{p.desc}</div>
                </div>
              </div>
            ))}
          </div>

          <p className="text-on-surface-variant leading-relaxed mb-6">
            {c.pipeline.desc}
          </p>
        </section>

        {/* Lexer + Suffix System */}
        <section className="mb-16" id="lexer">
          <h2 className="text-2xl font-bold text-on-surface mb-6">
            {c.lexer.title}
          </h2>
          <p className="mb-6 text-on-surface-variant leading-relaxed">
            {c.lexer.desc}
          </p>
          <CodeBlock code={lexerExampleCode} filename="lexer_analizi.txt" />

          {/* Token types table */}
          <div className="overflow-x-auto mt-8 bg-surface-container-low rounded-lg border border-outline-variant/10 p-6">
            <table className="w-full text-sm border-collapse">
              <thead>
                <tr className="border-b border-outline-variant/20">
                  <th className="text-left py-3 pr-8 text-on-surface-variant/60 font-bold text-[10px] uppercase tracking-widest">Token</th>
                  <th className="text-left py-3 pr-8 text-on-surface-variant/60 font-bold text-[10px] uppercase tracking-widest">Examples</th>
                  <th className="text-left py-3 text-on-surface-variant/60 font-bold text-[10px] uppercase tracking-widest">Notes</th>
                </tr>
              </thead>
              <tbody className="font-mono text-xs divide-y divide-outline-variant/10">
                {[
                  ["KEYWORD", "olsun, ise, döngü", locale === "tr" ? "Rezerve kelimeler" : "Reserved words"],
                  ["IDENT", "isim, sayi, araç", locale === "tr" ? "Eki silinmiş isimler" : "Suffix-stripped names"],
                  ["NUMBER", "42, 3.14", locale === "tr" ? "Sayısal değerler" : "Numeric literals"],
                  ["STRING", '"Hüma"', locale === "tr" ? "Metin dizileri" : "String literals"],
                  ["BUILTIN", "yazdır, ekle", locale === "tr" ? "Dahili fonksiyonlar" : "Built-in functions"],
                  ["PUNCT", "{ } ( ) ; ,", locale === "tr" ? "Yapısal ayraçlar" : "Delimiters"],
                ].map(([tok, ex, note]) => (
                  <tr key={tok}>
                    <td className="py-4 pr-8 text-primary font-bold">{tok}</td>
                    <td className="py-4 pr-8 text-tertiary">{ex}</td>
                    <td className="py-4 text-on-surface-variant font-body">{note}</td>
                  </tr>
                ))}
              </tbody>
            </table>
          </div>
        </section>

        {/* AST */}
        <section className="mb-16" id="ast">
          <h2 className="text-2xl font-bold text-on-surface mb-6">
            {c.ast.title}
          </h2>
          <p className="mb-6 text-on-surface-variant leading-relaxed">
            {c.ast.desc}
          </p>
          <CodeBlock code={astCode} filename="ast_yapisi.txt" />
          
          <div className="bg-tertiary/5 border-l-4 border-tertiary p-6 rounded-r-lg mt-8">
            <div className="flex items-center gap-3 mb-2 text-tertiary">
              <span className="material-symbols-outlined text-lg">code</span>
              <span className="text-xs font-bold uppercase tracking-widest">Source: src/ast.rs</span>
            </div>
            <p className="text-sm text-on-surface-variant leading-relaxed">
              {locale === "tr" 
                ? "Tüm AST düğüm tanımları 'src/ast.rs' içinde yer alır. Yinelemeli inişli (recursive descent) parser ise 'src/parser.rs' içindedir."
                : "Full AST node definitions live in 'src/ast.rs'. The recursive descent parser is in 'src/parser.rs'."}
            </p>
          </div>
        </section>

        {/* Build from source */}
        <section className="mb-16" id="build">
          <h2 className="text-2xl font-bold text-on-surface mb-6">
            {c.build.title}
          </h2>
          <p className="mb-6 text-on-surface-variant leading-relaxed">
            {c.build.desc}
          </p>
          <CodeBlock code={cargoCode} variant="terminal" />

          <div className="grid grid-cols-1 sm:grid-cols-3 gap-4 mt-8">
            {[
              { icon: "memory", label: locale === "tr" ? "DİL" : "LANGUAGE", value: "Rust (stable)" },
              { icon: "scale", label: locale === "tr" ? "LİSANS" : "LICENSE", value: "MIT" },
              { icon: "link", label: locale === "tr" ? "REPO" : "REPOSITORY", value: "huma-lang" },
            ].map((stat) => (
              <div
                key={stat.label}
                className="bg-surface-container-low rounded-lg border border-outline-variant/10 p-5 flex items-center gap-4 hover:border-primary/20 transition-colors"
              >
                <span className="material-symbols-outlined text-primary text-xl">{stat.icon}</span>
                <div>
                  <div className="text-[9px] uppercase tracking-widest text-on-surface-variant font-bold opacity-60">
                    {stat.label}
                  </div>
                  <div className="font-mono text-xs text-on-surface font-bold">{stat.value}</div>
                </div>
              </div>
            ))}
          </div>
        </section>

        {/* Navigation */}
        <div className="flex justify-between mt-24 pt-8 border-t border-outline-variant/10">
          <Link
            href={getPath("/docs/stdlib")}
            className="flex items-center gap-2 text-sm text-on-surface-variant hover:text-primary transition-colors font-bold uppercase tracking-widest text-[10px]"
          >
            <span className="material-symbols-outlined text-base">arrow_back</span>
            {dict.Docs.stdlib.title}
          </Link>
          <a
            href="https://github.com/VastSea0/huma-lang"
            target="_blank"
            rel="noopener noreferrer"
            className="flex items-center gap-2 text-sm text-on-surface-variant hover:text-primary transition-colors font-bold uppercase tracking-widest text-[10px]"
          >
            {locale === "tr" ? "GitHub'da İncele" : "View on GitHub"}
            <span className="material-symbols-outlined text-base">open_in_new</span>
          </a>
        </div>
      </main>

      {/* Right TOC */}
      <aside className="hidden xl:block w-64 sticky top-16 h-[calc(100vh-4rem)] py-12 px-8 overflow-y-auto border-l border-outline-variant/10 shrink-0">
        <h5 className="text-[10px] font-bold text-on-surface uppercase tracking-[0.2em] mb-6 opacity-40">
          {locale === "tr" ? "BU SAYFADA" : "ON THIS PAGE"}
        </h5>
        <ul className="space-y-4 text-[11px] font-bold uppercase tracking-widest">
          {[
            { href: "#pipeline", label: c.pipeline.title },
            { href: "#lexer", label: c.lexer.title },
            { href: "#ast", label: c.ast.title },
            { href: "#build", label: c.build.title },
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

        <div className="mt-12 pt-12 border-t border-outline-variant/10">
          <div className="bg-surface-container-high p-5 rounded-xl border border-outline-variant/5 shadow-sm">
            <p className="text-[9px] font-bold uppercase tracking-[0.2em] text-on-surface-variant/40 mb-4">
              {locale === "tr" ? "KAYNAK KOD" : "SOURCE CODE"}
            </p>
            <div className="flex flex-col gap-4 text-[11px] font-bold uppercase tracking-widest">
              <a
                href="https://github.com/VastSea0/huma-lang/tree/main/src"
                target="_blank"
                rel="noopener noreferrer"
                className="flex items-center gap-3 text-primary hover:text-primary-fixed transition-colors"
              >
                <span className="material-symbols-outlined text-base">folder_open</span>
                src/ GitHub
              </a>
              <a
                href="https://github.com/VastSea0/huma-lang/issues"
                target="_blank"
                rel="noopener noreferrer"
                className="flex items-center gap-3 text-primary hover:text-primary-fixed transition-colors"
              >
                <span className="material-symbols-outlined text-base">bug_report</span>
                {locale === "tr" ? "Hata Bildir" : "Report Issue"}
              </a>
            </div>
          </div>
        </div>
      </aside>
    </>
  );
}
