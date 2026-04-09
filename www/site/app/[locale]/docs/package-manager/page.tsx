import Link from "next/link";
import { getDictionary } from "@/dictionaries/dictionaries";
import CodeBlock from "@/components/CodeBlock";
import type { Metadata } from "next";

export async function generateMetadata({
  params,
}: {
  params: Promise<{ locale: string }>;
}): Promise<Metadata> {
  const { locale } = await params;
  const dict = await getDictionary(locale as "en" | "tr");
  return {
    title: dict.Docs.package_manager.title,
    description: dict.Docs.package_manager.description,
  };
}

export default async function PackageManagerPage({
  params,
}: {
  params: Promise<{ locale: string }>;
}) {
  const { locale } = await params;
  const dict = await getDictionary(locale as "en" | "tr");
  const p = dict.Docs.package_manager as any;

  const getPath = (path: string) => `/${locale}${path}`;

  return (
    <>
      <main className="flex-1 px-8 md:px-16 py-12 max-w-4xl">
        <nav className="flex gap-2 text-[10px] uppercase tracking-widest text-on-surface-variant/60 mb-4">
          <Link href={getPath("/docs")} className="hover:text-primary transition-colors">
            {dict.Nav.docs}
          </Link>
          <span>/</span>
          <span className="text-on-surface-variant">{dict.Sidebar.tooling}</span>
          <span>/</span>
          <span className="text-primary">{p.title}</span>
        </nav>

        <h1 className="text-5xl font-extrabold text-on-surface tracking-tighter mb-6">
          {p.hero_title}
        </h1>
        <p className="text-lg text-on-surface-variant leading-relaxed mb-12">
          {p.hero_desc}
        </p>

        {/* Command Reference */}
        <section className="mb-24" id="commands">
          <h2 className="text-2xl font-bold text-on-surface mb-8">
            {locale === "tr" ? "Komut Başvurusu" : "Command Reference"}
          </h2>
          <div className="grid grid-cols-1 md:grid-cols-2 gap-4">
            {p.commands.map((c: any) => (
              <div key={c.cmd} className="bg-surface-container-low/50 rounded-xl border border-outline-variant/10 overflow-hidden hover:border-primary/20 transition-all group">
                <div className="px-5 py-4 border-b border-outline-variant/5">
                  <code className="font-mono text-[11px] text-primary font-bold group-hover:scale-105 transition-transform inline-block lowercase">
                    {c.cmd}
                  </code>
                </div>
                <div className="px-5 py-4 text-xs text-on-surface-variant leading-relaxed">
                  {c.desc}
                </div>
              </div>
            ))}
          </div>
        </section>

        {/* Manifest (huma.json) */}
        <section className="mb-24" id="manifest">
          <h2 className="text-2xl font-bold text-on-surface mb-8">
            {p.manifest_title}
          </h2>
          <p className="text-on-surface-variant mb-8 leading-relaxed">
            {p.manifest_desc}
          </p>
          <CodeBlock 
            filename="huma.json"
            code={`{
  "ad": "merhaba_dunya",
  "surum": "1.0.0",
  "yazar": "Egehan KAHRAMAN",
  "lisans": "MIT",
  "github": "KullaniciAdi/merhaba_dunya",
  "giris": "ana.hb",
  "huma_surum": ">=0.5.0",
  "bagimliliklar": {
    "sunucu": "VastSea0/huma-sunucu@v1.2.0"
  },
  "betikler": {
    "baslat": "huma run ana.hb",
    "test": "huma run tests/test.hb"
  }
}`} 
          />
        </section>

        {/* Scripts & Automation & Security */}
        <section className="grid grid-cols-1 lg:grid-cols-2 gap-8 mb-24">
          <div className="bg-surface-container-low p-8 rounded-2xl border border-outline-variant/10">
            <h3 className="text-xl font-bold text-on-surface mb-4">
              {p.scripts_title}
            </h3>
            <p className="text-sm text-on-surface-variant leading-relaxed mb-6">
              {p.scripts_desc}
            </p>
            <CodeBlock code={`$ huma paket run baslat`} variant="terminal" />
          </div>
          
          <div className="flex flex-col gap-8">
            <div className="bg-surface-container-low p-8 rounded-2xl border border-outline-variant/10">
              <h3 className="text-xl font-bold text-on-surface mb-4">
                {p.verify_title}
              </h3>
              <p className="text-sm text-on-surface-variant leading-relaxed mb-6">
                {p.verify_desc}
              </p>
              <CodeBlock code={`$ huma paket doğrula`} variant="terminal" />
            </div>

            <div className="bg-primary/5 p-8 rounded-2xl border border-primary/20">
              <h3 className="text-xl font-bold text-primary mb-4 flex items-center gap-2">
                <span className="material-symbols-outlined text-xl">security</span>
                {p.security_title}
              </h3>
              <p className="text-sm text-on-surface-variant leading-relaxed">
                {p.security_desc}
              </p>
            </div>
          </div>
        </section>

        {/* Lock System */}
        <section className="mb-24" id="dependencies">
          <h2 className="text-2xl font-bold text-on-surface mb-8">
            {p.dependencies_title}
          </h2>
          <div className="grid grid-cols-1 md:grid-cols-2 gap-6">
            <div className="bg-surface-container-low p-8 rounded-2xl border border-outline-variant/10">
              <h4 className="font-mono text-[10px] font-bold text-primary uppercase tracking-widest mb-4">huma.lock</h4>
              <p className="text-sm text-on-surface-variant leading-relaxed">
                {locale === "tr" 
                  ? "Bağımlılıkların tam ve özgün sürümlerini (SHA-256 hash dahil) kilitler. Bu sayede projeniz her makinede aynı ikili bütünlükle çalışır."
                  : "Locks exact versions and SHA-256 hashes of dependencies. Ensures bit-perfect reproducibility across all developer machines."}
              </p>
            </div>
            <div className="bg-surface-container-low p-8 rounded-2xl border border-outline-variant/10">
              <h4 className="font-mono text-[10px] font-bold text-primary uppercase tracking-widest mb-4">SemVer</h4>
              <p className="text-sm text-on-surface-variant leading-relaxed">
                {locale === "tr" 
                  ? "Hüma, anlamsal sürümleme kurallarına (major.minor.patch) tam uyumludur. Sürüm çakışmalarını derleme öncesi tespit eder."
                  : "Fully compliant with Semantic Versioning (major.minor.patch). Detects version conflicts before the build process starts."}
              </p>
            </div>
          </div>
        </section>

        {/* Publishing */}
        <section className="mb-24" id="publishing">
          <h2 className="text-2xl font-bold text-on-surface mb-8">
            {p.publishing_title}
          </h2>
          <p className="text-on-surface-variant mb-8 leading-relaxed">
            {p.publishing_desc}
          </p>
          <div className="bg-surface-container-lowest p-8 border border-outline-variant/10 rounded-2xl">
            <ul className="space-y-4 text-sm text-on-surface-variant">
              {p.pub_steps.map((step: string, idx: number) => (
                <li key={idx} className="flex gap-4">
                  <span className="w-6 h-6 rounded-full bg-primary/10 text-primary flex items-center justify-center text-[10px] font-bold shrink-0">
                    {idx + 1}
                  </span>
                  {step}
                </li>
              ))}
            </ul>
          </div>
        </section>

        {/* Native Packages */}
        <section className="mb-24" id="native">
          <h2 className="text-2xl font-bold text-on-surface mb-8">
            {p.native_title}
          </h2>
          <p className="text-on-surface-variant mb-8 leading-relaxed">
            {p.native_desc}
          </p>
          <div className="bg-surface-container-low p-8 rounded-2xl border border-outline-variant/10">
            <h4 className="font-mono text-[10px] font-bold text-primary uppercase tracking-widest mb-4">
              {locale === "tr" ? "Native Konfigürasyon Örneği" : "Native Configuration Example"}
            </h4>
            <CodeBlock 
              filename="huma.json"
              code={`{
  "ad": "huma_sqlite",
  "crate_bagimliliklari": {
    "rusqlite": "0.31"
  },
  "yerleşik_rust": "use rusqlite::Connection; ..."
}`} 
            />
          </div>
        </section>

        {/* Platform Callout */}
        <section className="mb-24">
          <div className="bg-primary/5 border border-primary/20 rounded-3xl p-10 flex flex-col md:flex-row items-center justify-between gap-8 relative overflow-hidden">
            <div className="absolute top-0 right-0 w-64 h-64 bg-primary/5 rounded-full -mr-32 -mt-32 blur-3xl"></div>
            <div className="relative z-10 text-center md:text-left">
              <h3 className="text-2xl font-extrabold text-on-surface mb-3 tracking-tight">
                {p.platform.title}
              </h3>
              <p className="text-on-surface-variant text-sm max-w-sm leading-relaxed">
                {p.platform.desc}
              </p>
            </div>
            <Link 
              href="https://github.com/VastSea0/huma-lang/discussions" 
              target="_blank"
              className="relative z-10 bg-primary text-on-primary px-10 py-5 rounded-xl font-bold text-sm hover:scale-105 transition-all shadow-xl shadow-primary/25 shrink-0"
            >
              {p.platform.cta}
            </Link>
          </div>
        </section>

        {/* Navigation */}
        <div className="flex justify-between mt-24 pt-8 border-t border-outline-variant/10">
          <Link
            href={getPath("/docs/gui")}
            className="flex items-center gap-2 text-sm text-on-surface-variant hover:text-primary transition-colors font-bold uppercase tracking-widest text-[10px]"
          >
            <span className="material-symbols-outlined text-base">arrow_back</span>
            {dict.Docs.gui.title}
          </Link>
          <Link
            href={getPath("/docs/my-first-package")}
            className="flex items-center gap-2 text-sm text-on-surface-variant hover:text-primary transition-colors font-bold uppercase tracking-widest text-[10px]"
          >
            {dict.Sidebar.items.my_first_package}
            <span className="material-symbols-outlined text-base">arrow_forward</span>
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
            { href: "#commands", label: locale === "tr" ? "Komutlar" : "Commands" },
            { href: "#manifest", label: "Manifest" },
            { href: "#dependencies", label: locale === "tr" ? "Bağımlılıklar" : "Dependencies" },
            { href: "#publishing", label: locale === "tr" ? "Yayınlama" : "Publishing" },
            { href: "#native", label: "Native" },
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
