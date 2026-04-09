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

  const methods = [
    { name: "kur(yol)", desc: p.methods.kur, example: `vt'nin kur'u("test.db")` },
    { name: "yürüt(sql)", desc: p.methods.yürüt, example: `vt'nin yürüt'ü("INSERT INTO x VALUES (1)")` },
    { name: "sorgula(sql)", desc: p.methods.sorgula, example: `liste = vt'nin sorgula'sı("SELECT * FROM x")` },
  ];

  return (
    <>
      <main className="flex-1 px-8 md:px-16 py-12 max-w-4xl">
        {/* Breadcrumb */}
        <nav className="flex gap-2 text-[10px] uppercase tracking-widest text-on-surface-variant/60 mb-4">
          <Link href={getPath("/docs")} className="hover:text-primary transition-colors">
            {dict.Nav.docs}
          </Link>
          <span>/</span>
          <span className="text-on-surface-variant">{dict.Sidebar.core_libraries}</span>
          <span>/</span>
          <span className="text-primary">{p.title}</span>
        </nav>

        {/* Title & Description */}
        <h1 className="text-5xl font-extrabold text-on-surface tracking-tighter mb-6">
          {p.hero_title}
        </h1>
        <p className="text-lg text-on-surface-variant leading-relaxed mb-12">
          {p.hero_desc}
        </p>

        {/* Quick Start */}
        <section className="mb-16" id="quick-start">
          <h2 className="text-2xl font-bold text-on-surface mb-4">
            {p.quick_start.title}
          </h2>
          <p className="text-on-surface-variant mb-6 leading-relaxed">
            {p.quick_start.desc}
          </p>
          <CodeBlock
            code={`yükle "huma_sqlite"

vt = Veritabanı()
vt'nin kur'u("veriler.db")

// Tablo oluştur
vt'nin yürüt'ü("CREATE TABLE IF NOT EXISTS notlar (id INTEGER PRIMARY KEY, icerik TEXT)")`}
          />
        </section>

        {/* API Reference */}
        <section className="mb-16" id="api-reference">
          <h2 className="text-2xl font-bold text-on-surface mb-6">
            {locale === "tr" ? "API Referansı" : "API Reference"}
          </h2>
          <div className="overflow-x-auto bg-surface-container-low rounded-lg border border-outline-variant/10 mb-8">
            <table className="w-full text-sm border-collapse">
              <thead>
                <tr className="border-b border-outline-variant/20">
                  <th className="text-left py-3 px-5 text-on-surface-variant/60 font-bold text-[10px] uppercase tracking-widest">
                    {locale === "tr" ? "Metot" : "Method"}
                  </th>
                  <th className="text-left py-3 px-5 text-on-surface-variant/60 font-bold text-[10px] uppercase tracking-widest">
                    {locale === "tr" ? "Açıklama" : "Description"}
                  </th>
                </tr>
              </thead>
              <tbody className="divide-y divide-outline-variant/10">
                {methods.map((m) => (
                  <tr key={m.name} className="hover:bg-surface-container-lowest transition-colors">
                    <td className="py-4 px-5">
                      <code className="text-primary font-mono text-xs font-bold">{m.name}</code>
                    </td>
                    <td className="py-4 px-5 text-on-surface-variant text-sm leading-relaxed">
                      {m.desc}
                    </td>
                  </tr>
                ))}
              </tbody>
            </table>
          </div>

          {/* Method usage examples */}
          <div className="space-y-4">
            {methods.map((m) => (
              <CodeBlock key={m.name} code={m.example} />
            ))}
          </div>
        </section>

        {/* Real-world Example */}
        <section className="mb-16" id="example">
          <h2 className="text-2xl font-bold text-on-surface mb-4">
            {locale === "tr" ? "Örnek: Verileri Listeleme" : "Example: Listing Data"}
          </h2>
          <CodeBlock
            code={`sonuclar = vt'nin sorgula'sı("SELECT * FROM notlar")
i = 0 olsun
u = sonuclar'ın uzunluğu

i < u olduğu sürece {
    satır = sonuclar[i]
    "Not ID: " + (satır'ın id'i) + ", İçerik: " + (satır'ın icerik'i)'i yazdır
    i = i + 1 olsun
}`}
          />
          <div className="mt-6 bg-primary/5 border-l-4 border-primary p-6 rounded-r-2xl text-sm text-on-surface-variant leading-relaxed">
            {locale === "tr"
              ? "Not: Sütun isimlerine 'ın, 'in gibi iyelik ekleriyle doğrudan nesne özelliği olarak erişebilirsiniz."
              : "Note: You can access column names directly as object properties using possessive suffixes like 'ın, 'in."}
          </div>
        </section>

        {/* Navigation */}
        <div className="flex justify-between mt-24 pt-8 border-t border-outline-variant/10">
          <Link
            href={getPath("/docs/stdlib")}
            className="flex items-center gap-2 text-on-surface-variant hover:text-primary transition-colors font-bold uppercase tracking-widest text-[10px]"
          >
            <span className="material-symbols-outlined text-base">arrow_back</span>
            {dict.Docs.stdlib.title}
          </Link>
          <Link
            href={getPath("/docs/ag_istekleri")}
            className="flex items-center gap-2 text-on-surface-variant hover:text-primary transition-colors font-bold uppercase tracking-widest text-[10px]"
          >
            {dict.Sidebar.items.ag_istekleri}
            <span className="material-symbols-outlined text-base">arrow_forward</span>
          </Link>
        </div>
      </main>
    </>
  );
}
