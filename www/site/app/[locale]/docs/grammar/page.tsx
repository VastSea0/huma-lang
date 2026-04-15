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
    title: dict.Docs?.grammar?.title || "Grammar",
    description: dict.Docs?.grammar?.description || "Grammar description",
  };
}

const compilerCode = `// Geliştiricinin yazdığı doğal kod:
isim = "Hüma" olsun
isim'i yazdır;

/* 
 * Derleyici (Lexer) aşamasında kod anında saflaştırılır.
 * "isim'i" token'ı alınır, kesme işaretinden sonrası (i) atılır.
 * VM'ye giden nihai kod şuna denktir:
 * isim yazdır
 */`;

const usageCode = `// Nesne erişimleri (İlgi Durumu: 'nin, 'ın)
ayarlar = { "tema": "koyu" } olsun
yazdır ayarlar'ın tema'sı;

// Listeye ekleme (Yönelme Durumu: 'e, 'a, 'ye, 'ya)
sayılar = [1, 2] olsun
sayılar'a [3]'ü ekle;

// Edatlı Çağrı ve Postfix Yükle
"matematik.hb"'yi yükle
10 ile 20'yi topla'yı yazdır;

// Nesneyi hedef alma (Belirtme Durumu: 'ni, 'nı, 'nu, 'nü)
dosya_adı = "huma.txt" olsun
dosya_adı'nı yazdır;`;

export default async function GrammarPage({
  params,
}: {
  params: Promise<{ locale: string }>;
}) {
  const { locale } = await params;
  const dict = await getDictionary(locale as "en" | "tr");
  const g = dict.Docs?.grammar || {};

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
          <span className="text-primary">{g?.title || ""}</span>
        </nav>

        <h1 className="text-5xl font-extrabold text-on-surface tracking-tighter mb-6">
          {g?.hero_title}
        </h1>
        <p className="text-lg text-on-surface-variant leading-relaxed mb-12">
          {g?.hero_desc}
        </p>

        {/* Mechanics */}
        <section className="mb-16" id="mechanics">
          <h2 className="text-2xl font-bold text-on-surface mb-6 flex items-center gap-3">
            <span className="w-8 h-8 rounded-full bg-surface-container-high flex items-center justify-center text-sm font-mono text-primary">
              01
            </span>
            {g?.mechanics?.title}
          </h2>
          <p className="mb-6 text-on-surface-variant leading-relaxed">
            {g?.mechanics?.desc}
          </p>
          <CodeBlock code={compilerCode} filename="mimari_ornek.hb" />
        </section>

        {/* Suffixes Library */}
        <section className="mb-16" id="suffixes">
          <h2 className="text-2xl font-bold text-on-surface mb-6 flex items-center gap-3">
            <span className="w-8 h-8 rounded-full bg-surface-container-high flex items-center justify-center text-sm font-mono text-primary">
              02
            </span>
            {g?.suffixes?.title}
          </h2>
          <p className="mb-8 text-on-surface-variant leading-relaxed">
            {g?.suffixes?.desc}
          </p>

          <div className="grid grid-cols-1 md:grid-cols-2 gap-6 mb-8">
            {/* Accusative */}
            <div className="bg-surface-container-lowest border border-outline-variant/10 rounded-2xl p-6 hover:border-primary/30 transition-colors">
              <h3 className="text-lg font-bold text-primary mb-2">{g?.suffixes?.accusative?.name}</h3>
              <p className="text-xs font-mono text-tertiary mb-4 bg-tertiary/10 inline-block px-2 py-1 rounded">
                {g?.suffixes?.accusative?.suffixes}
              </p>
              <p className="text-sm text-on-surface-variant mb-4">{g?.suffixes?.accusative?.usage}</p>
              <p className="text-xs font-mono text-on-surface/80">» {g?.suffixes?.accusative?.example}</p>
            </div>

            {/* Dative */}
            <div className="bg-surface-container-lowest border border-outline-variant/10 rounded-2xl p-6 hover:border-primary/30 transition-colors">
              <h3 className="text-lg font-bold text-primary mb-2">{g?.suffixes?.dative?.name}</h3>
              <p className="text-xs font-mono text-tertiary mb-4 bg-tertiary/10 inline-block px-2 py-1 rounded">
                {g?.suffixes?.dative?.suffixes}
              </p>
              <p className="text-sm text-on-surface-variant mb-4">{g?.suffixes?.dative?.usage}</p>
              <p className="text-xs font-mono text-on-surface/80">» {g?.suffixes?.dative?.example}</p>
            </div>

            {/* Ablative */}
            <div className="bg-surface-container-lowest border border-outline-variant/10 rounded-2xl p-6 hover:border-primary/30 transition-colors">
              <h3 className="text-lg font-bold text-primary mb-2">{g?.suffixes?.ablative?.name}</h3>
              <p className="text-xs font-mono text-tertiary mb-4 bg-tertiary/10 inline-block px-2 py-1 rounded">
                {g?.suffixes?.ablative?.suffixes}
              </p>
              <p className="text-sm text-on-surface-variant mb-4">{g?.suffixes?.ablative?.usage}</p>
              <p className="text-xs font-mono text-on-surface/80">» {g?.suffixes?.ablative?.example}</p>
            </div>

            {/* Genitive */}
            <div className="bg-surface-container-lowest border border-outline-variant/10 rounded-2xl p-6 hover:border-primary/30 transition-colors">
              <h3 className="text-lg font-bold text-primary mb-2">{g?.suffixes?.genitive?.name}</h3>
              <p className="text-xs font-mono text-tertiary mb-4 bg-tertiary/10 inline-block px-2 py-1 rounded">
                {g?.suffixes?.genitive?.suffixes}
              </p>
              <p className="text-sm text-on-surface-variant mb-4">{g?.suffixes?.genitive?.usage}</p>
              <p className="text-xs font-mono text-on-surface/80">» {g?.suffixes?.genitive?.example}</p>
            </div>

            {/* Possessive */}
            <div className="bg-surface-container-lowest border border-outline-variant/10 rounded-2xl p-6 hover:border-primary/30 transition-colors">
              <h3 className="text-lg font-bold text-primary mb-2">{g?.suffixes?.possessive?.name}</h3>
              <p className="text-xs font-mono text-tertiary mb-4 bg-tertiary/10 inline-block px-2 py-1 rounded">
                {g?.suffixes?.possessive?.suffixes}
              </p>
              <p className="text-sm text-on-surface-variant mb-4">{g?.suffixes?.possessive?.usage}</p>
              <p className="text-xs font-mono text-on-surface/80">» {g?.suffixes?.possessive?.example}</p>
            </div>

            {/* Plural */}
            <div className="bg-surface-container-lowest border border-outline-variant/10 rounded-2xl p-6 hover:border-primary/30 transition-colors">
              <h3 className="text-lg font-bold text-primary mb-2">{g?.suffixes?.plural?.name}</h3>
              <p className="text-xs font-mono text-tertiary mb-4 bg-tertiary/10 inline-block px-2 py-1 rounded">
                {g?.suffixes?.plural?.suffixes}
              </p>
              <p className="text-sm text-on-surface-variant mb-4">{g?.suffixes?.plural?.usage}</p>
              <p className="text-xs font-mono text-on-surface/80">» {g?.suffixes?.plural?.example}</p>
            </div>

            {/* Equality */}
            <div className="bg-surface-container-lowest border border-outline-variant/10 rounded-2xl p-6 hover:border-primary/30 transition-colors">
              <h3 className="text-lg font-bold text-primary mb-2">{g?.suffixes?.equality?.name}</h3>
              <p className="text-xs font-mono text-tertiary mb-4 bg-tertiary/10 inline-block px-2 py-1 rounded">
                {g?.suffixes?.equality?.suffixes}
              </p>
              <p className="text-sm text-on-surface-variant mb-4">{g?.suffixes?.equality?.usage}</p>
              <p className="text-xs font-mono text-on-surface/80">» {g?.suffixes?.equality?.example}</p>
            </div>
          </div>

          <CodeBlock code={usageCode} filename="ekler.hb" />
        </section>

        {/* Best Practices */}
        <section className="mb-16" id="practices">
          <h2 className="text-2xl font-bold text-on-surface mb-6 flex items-center gap-3">
            <span className="w-8 h-8 rounded-full bg-surface-container-high flex items-center justify-center text-sm font-mono text-primary">
              03
            </span>
            {g?.best_practices?.title}
          </h2>
          <div className="bg-primary/5 border-l-4 border-primary p-6 rounded-r-lg">
            <div className="flex items-center gap-3 mb-2 text-primary">
              <span className="material-symbols-outlined text-lg">lightbulb</span>
              <span className="text-xs font-bold uppercase tracking-widest">{locale === "tr" ? "İpucu" : "Tip"}</span>
            </div>
            <p className="text-sm text-on-surface-variant leading-relaxed">
              {g?.best_practices?.desc}
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
            {dict.Docs?.syntax?.title}
          </Link>
          <Link
            href={getPath("/docs/cli")}
            className="flex items-center gap-2 text-sm text-on-surface-variant hover:text-primary transition-colors font-bold uppercase tracking-widest text-[10px]"
          >
            {dict.Docs?.cli?.title}
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
            { href: "#mechanics", label: g?.mechanics?.title },
            { href: "#suffixes", label: g?.suffixes?.title },
            { href: "#practices", label: g?.best_practices?.title },
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
