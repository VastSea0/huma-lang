import Link from "next/link";
import { getDictionary } from "@/dictionaries/dictionaries";
import CodeBlock from "@/components/CodeBlock";
import type { Metadata } from "next";

export const metadata: Metadata = {
  title: "Hüma SQLite Module",
  description: "Native SQLite database integration for Hüma applications.",
};

export default async function SqliteDocsPage({
  params,
}: {
  params: Promise<{ locale: string }>;
}) {
  const { locale } = await params;
  const dict = await getDictionary(locale as "en" | "tr");
  const p = (dict.Docs as any).sqlite;

  const getPath = (path: string) => `/${locale}${path}`;

  return (
    <>
      <main className="flex-1 px-8 md:px-16 py-12 max-w-4xl">
        <nav className="flex gap-2 text-[10px] uppercase tracking-widest text-on-surface-variant/60 mb-4">
          <Link href={getPath("/docs")} className="hover:text-primary transition-colors">
            {dict.Nav.docs}
          </Link>
          <span>/</span>
          <span className="text-on-surface-variant">{dict.Sidebar.items.stdlib}</span>
          <span>/</span>
          <span className="text-primary">{p.title}</span>
        </nav>

        <h1 className="text-5xl font-extrabold text-on-surface tracking-tighter mb-6">
          {p.hero_title}
        </h1>
        <p className="text-lg text-on-surface-variant leading-relaxed mb-12">
          {p.hero_desc}
        </p>

        {/* Quick Start */}
        <section className="mb-24">
          <h2 className="text-2xl font-bold text-on-surface mb-8 flex items-center gap-3">
            <span className="w-8 h-8 rounded-full bg-surface-container-high flex items-center justify-center text-sm font-mono text-primary">
              01
            </span>
            {p.quick_start.title}
          </h2>
          <p className="text-on-surface-variant mb-8 leading-relaxed">
            {p.quick_start.desc}
          </p>
          <CodeBlock 
            code={`yükle "huma_sqlite"

vt = Veritabanı()
vt'nin kur("veriler.db")

// Tablo oluştur
vt'nin yürüt("CREATE TABLE IF NOT EXISTS notlar (id INTEGER PRIMARY KEY, icerik TEXT)")`} 
          />
        </section>

        {/* Methods Reference */}
        <section className="mb-24">
          <h2 className="text-2xl font-bold text-on-surface mb-8 flex items-center gap-3">
            <span className="w-8 h-8 rounded-full bg-surface-container-high flex items-center justify-center text-sm font-mono text-primary">
              02
            </span>
            {locale === "tr" ? "Metot Başvurusu" : "Methods Reference"}
          </h2>
          <div className="space-y-6">
            {[
              { name: "kur(yol)", desc: p.methods.kur, example: `vt'nin kur("test.db")` },
              { name: "yürüt(sql)", desc: p.methods.yürüt, example: `vt'nin yürüt("INSERT INTO x VALUES (1)")` },
              { name: "sorgula(sql)", desc: p.methods.sorgula, example: `liste = vt'nin sorgula("SELECT * FROM x")` },
            ].map((m) => (
              <div key={m.name} className="bg-surface-container-low/50 rounded-2xl border border-outline-variant/10 p-8">
                <div className="flex flex-col md:flex-row md:items-center justify-between gap-4 mb-4">
                  <code className="text-primary font-bold font-mono text-lg">{m.name}</code>
                  <span className="text-xs text-on-surface-variant bg-surface-container-high px-3 py-1 rounded-full border border-outline-variant/10 italic">
                    {m.desc}
                  </span>
                </div>
                <CodeBlock code={m.example} />
              </div>
            ))}
          </div>
        </section>

        {/* Example: Full Loop */}
        <section className="mb-24">
          <h2 className="text-2xl font-bold text-on-surface mb-8">
            {locale === "tr" ? "Örnek: Verileri Listeleme" : "Example: Listing Data"}
          </h2>
          <CodeBlock 
            code={`sonuclar = vt'nin sorgula("SELECT * FROM notlar")
i = 0 olsun
u = sonuclar'ın uzunluğu

i < u olduğu sürece {
    satır = sonuclar[i]
    "Not ID: " + (satır'ın id) + ", İçerik: " + (satır'ın icerik)'i yazdır
    i = i + 1 olsun
}`} 
          />
           <div className="mt-8 bg-primary/5 border border-primary/20 rounded-2xl p-6 text-sm text-on-surface-variant leading-relaxed italic">
            {locale === "tr" 
              ? "Not: Sütun isimlerine 'ın, 'in gibi iyelik ekleriyle doğrudan nesne özelliği olarak erişebilirsiniz."
              : "Note: You can access column names directly as object properties using possessive suffixes like 'ın, 'in."}
          </div>
        </section>

        {/* Navigation */}
        <div className="flex justify-between mt-24 pt-8 border-t border-outline-variant/10">
          <Link
            href={getPath("/docs/package-manager")}
            className="flex items-center gap-2 text-sm text-on-surface-variant hover:text-primary transition-colors font-bold uppercase tracking-widest text-[10px]"
          >
            <span className="material-symbols-outlined text-base">arrow_back</span>
            {dict.Docs.package_manager.title}
          </Link>
          <Link
            href={getPath("/docs/changelog")}
            className="flex items-center gap-2 text-sm text-on-surface-variant hover:text-primary transition-colors font-bold uppercase tracking-widest text-[10px]"
          >
            {dict.Sidebar.items.changelog}
            <span className="material-symbols-outlined text-base">arrow_forward</span>
          </Link>
        </div>
      </main>
    </>
  );
}
