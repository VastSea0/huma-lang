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
          <h2 className="text-3xl font-black text-on-surface mb-8 tracking-tighter flex items-center gap-4">
            <span className="w-10 h-10 rounded-2xl bg-primary/10 flex items-center justify-center text-lg font-mono text-primary rotate-3">
              02
            </span>
            {g?.suffixes?.title}
          </h2>
          <p className="mb-10 text-xl text-on-surface-variant/80 font-medium leading-relaxed max-w-2xl">
            {g?.suffixes?.desc}
          </p>

          <div className="grid grid-cols-1 md:grid-cols-3 gap-6 mb-12">
             {/* Accusative */}
             <div className="group relative bg-surface-container-lowest border border-outline-variant/10 rounded-3xl p-8 hover:bg-primary/[0.02] transition-all duration-500 overflow-hidden">
              <div className="absolute top-0 right-0 p-4 opacity-5 group-hover:opacity-10 transition-opacity">
                <span className="text-6xl font-black">i</span>
              </div>
              <div className="relative z-10">
                <h3 className="text-xl font-bold text-primary mb-3">Belirtme Durumu</h3>
                <div className="flex gap-1 mb-6">
                  {["'i", "'ı", "'u", "'ü"].map(s => (
                    <span key={s} className="px-2 py-1 bg-primary/10 text-primary font-mono text-xs rounded-lg">{s}</span>
                  ))}
                </div>
                <p className="text-on-surface-variant text-sm mb-6 leading-relaxed">Değişkeni veya nesneyi hedef alır.</p>
                <div className="p-4 bg-surface-container rounded-xl border border-outline-variant/20 font-mono text-[11px]">
                  <span className="opacity-50"># Örnek</span><br/>
                  sayı'yı <span className="text-primary">yazdır</span>
                </div>
              </div>
            </div>

            {/* Dative */}
            <div className="group relative bg-surface-container-lowest border border-outline-variant/10 rounded-3xl p-8 hover:bg-primary/[0.02] transition-all duration-500 overflow-hidden">
              <div className="absolute top-0 right-0 p-4 opacity-5 group-hover:opacity-10 transition-opacity">
                <span className="text-6xl font-black">e</span>
              </div>
              <div className="relative z-10">
                <h3 className="text-xl font-bold text-primary mb-3">Yönelme Durumu</h3>
                <div className="flex gap-1 mb-6">
                  {["'e", "'a", "'ye", "'ya"].map(s => (
                    <span key={s} className="px-2 py-1 bg-primary/10 text-primary font-mono text-xs rounded-lg">{s}</span>
                  ))}
                </div>
                <p className="text-on-surface-variant text-sm mb-6 leading-relaxed">İşlemin yapılacağı hedefi belirtir.</p>
                <div className="p-4 bg-surface-container rounded-xl border border-outline-variant/20 font-mono text-[11px]">
                  <span className="opacity-50"># Örnek</span><br/>
                  liste'ye <span className="text-primary">ekle</span>
                </div>
              </div>
            </div>

            {/* Genitive */}
            <div className="group relative bg-surface-container-lowest border border-outline-variant/10 rounded-3xl p-8 hover:bg-primary/[0.02] transition-all duration-500 overflow-hidden">
              <div className="absolute top-0 right-0 p-4 opacity-5 group-hover:opacity-10 transition-opacity">
                <span className="text-6xl font-black">in</span>
              </div>
              <div className="relative z-10">
                <h3 className="text-xl font-bold text-primary mb-3">İlgi/İyelik Durumu</h3>
                <div className="flex gap-1 mb-6">
                  {["'in", "'ın", "'un", "'ün"].map(s => (
                    <span key={s} className="px-2 py-1 bg-primary/10 text-primary font-mono text-xs rounded-lg">{s}</span>
                  ))}
                </div>
                <p className="text-on-surface-variant text-sm mb-6 leading-relaxed">Özelliklere ve alt öğelere erişir.</p>
                <div className="p-4 bg-surface-container rounded-xl border border-outline-variant/20 font-mono text-[11px]">
                  <span className="opacity-50"># Örnek</span><br/>
                  kişi'nin <span className="text-primary">ad'ı</span>
                </div>
              </div>
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
