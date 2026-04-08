import Link from "next/link";
import { getDictionary } from "@/dictionaries/dictionaries";
import CodeBlock from "@/components/CodeBlock";
import type { Metadata } from "next";

export const metadata: Metadata = {
  title: "Hüma Package Manager",
  description: "Manage, update, and download Hüma libraries from the community.",
};

export default async function PackageManagerPage({
  params,
}: {
  params: Promise<{ locale: string }>;
}) {
  const { locale } = await params;
  const dict = await getDictionary(locale as "en" | "tr");
  const p = dict.Docs.package_manager;

  const getPath = (path: string) => `/${locale}${path}`;

  const commands = [
    {
      cmd: "huma paket kur",
      desc: locale === "tr" ? "huma.json'daki tüm bağımlılıkları yükler." : "Installs all dependencies in huma.json.",
    },
    {
      cmd: "huma paket kur <isim>",
      desc: locale === "tr" ? "Belirtilen paketi yükler ve huma.json'a ekler." : "Installs specified package and adds to huma.json.",
    },
    {
      cmd: "huma paket yeni <isim>",
      desc: locale === "tr" ? "Yeni bir kütüphane şablonu oluşturur." : "Creates a new library template.",
    },
    {
      cmd: "huma paket liste",
      desc: locale === "tr" ? "Yüklü paketleri ve sürümlerini listeler." : "Lists installed packages and versions.",
    },
    {
      cmd: "huma paket güncelle",
      desc: locale === "tr" ? "Paketleri en son sürümlere yükseltir." : "Updates packages to latest versions.",
    },
  ];

  return (
    <>
      <main className="flex-1 px-8 md:px-16 py-12 max-w-4xl">
        <nav className="flex gap-2 text-[10px] uppercase tracking-widest text-on-surface-variant/60 mb-4">
          <Link href={getPath("/docs")} className="hover:text-primary transition-colors">
            {dict.Nav.docs}
          </Link>
          <span>/</span>
          <span className="text-on-surface-variant">{dict.Sidebar.ecosystem}</span>
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
        <section className="mb-24">
          <h2 className="text-2xl font-bold text-on-surface mb-8 flex items-center gap-3">
             <span className="w-8 h-8 rounded-full bg-surface-container-high flex items-center justify-center text-sm font-mono text-primary">
              01
            </span>
            {locale === "tr" ? "Komut Başvurusu" : "Command Reference"}
          </h2>
          <div className="grid grid-cols-1 md:grid-cols-2 gap-4">
            {[
              {
                cmd: "huma paket ilkle / init",
                desc: locale === "tr" ? "Mevcut dizini bir Hüma projesi olarak başlatır." : "Initializes the current directory as a Hüma project.",
              },
              {
                cmd: "huma paket yeni / new <ad>",
                desc: locale === "tr" ? "Yeni bir klasör açıp proje şablonu oluşturur." : "Creates a new folder and project template.",
              },
              {
                cmd: "huma paket kur / install [ad]",
                desc: locale === "tr" ? "Tüm bağımlılıkları veya belirtilen paketi kurar." : "Installs all dependencies or a specific package.",
              },
              {
                cmd: "huma paket liste / list",
                desc: locale === "tr" ? "Kilitli sürümleriyle yüklü paketleri listeler." : "Lists installed packages with locked versions.",
              },
              {
                cmd: "huma paket sil / remove <ad>",
                desc: locale === "tr" ? "Belirtilen paketi ve dosyalarını projeden kaldırır." : "Removes a specific package and its files.",
              },
              {
                cmd: "huma paket run <betik>",
                desc: locale === "tr" ? "huma.json içinde tanımlı bir otomasyon betiğini çalıştırır." : "Executes an automation script defined in huma.json.",
              }
            ].map((c) => (
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
        <section className="mb-24">
          <h2 className="text-2xl font-bold text-on-surface mb-8 flex items-center gap-3">
            <span className="w-8 h-8 rounded-full bg-surface-container-high flex items-center justify-center text-sm font-mono text-primary">
              02
            </span>
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

            <div className="bg-secondary/5 p-8 rounded-2xl border border-secondary/20">
              <h3 className="text-xl font-bold text-secondary mb-4 flex items-center gap-2">
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
        <section className="mb-24">
          <h2 className="text-2xl font-bold text-on-surface mb-8 flex items-center gap-3">
            <span className="w-8 h-8 rounded-full bg-surface-container-high flex items-center justify-center text-sm font-mono text-primary">
              03
            </span>
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
        <section className="mb-24">
          <h2 className="text-2xl font-bold text-on-surface mb-8 flex items-center gap-3">
            <span className="w-8 h-8 rounded-full bg-surface-container-high flex items-center justify-center text-sm font-mono text-primary">
              04
            </span>
            {p.publishing_title}
          </h2>
          <p className="text-on-surface-variant mb-8 leading-relaxed">
            {p.publishing_desc}
          </p>
          <div className="bg-surface-container-lowest p-8 border border-outline-variant/10 rounded-2xl">
            <ul className="space-y-4 text-sm text-on-surface-variant">
              <li className="flex gap-4">
                 <span className="w-6 h-6 rounded-full bg-primary/10 text-primary flex items-center justify-center text-[10px] font-bold shrink-0">1</span>
                 {locale === "tr" ? "Kodunuzu GitHub reposuna yükleyin (huma.json projeniz olmalı)." : "Upload your code to GitHub (ensure huma.json is present)."}
              </li>
              <li className="flex gap-4">
                 <span className="w-6 h-6 rounded-full bg-primary/10 text-primary flex items-center justify-center text-[10px] font-bold shrink-0">2</span>
                 {locale === "tr" ? "Geçerli bir sürüm numarası belirleyin (x.y.z)." : "Specify a valid version number (x.y.z)."}
              </li>
              <li className="flex gap-4">
                 <span className="w-6 h-6 rounded-full bg-primary/10 text-primary flex items-center justify-center text-[10px] font-bold shrink-0">3</span>
                 {locale === "tr" ? "GitHub Release oluşturup v1.0.0 gibi bir etiket ekleyin." : "Create a GitHub Release with a tag like v1.0.0."}
              </li>
            </ul>
          </div>
        </section>

        {/* Native Packages */}
        <section className="mb-24">
          <h2 className="text-2xl font-bold text-on-surface mb-8 flex items-center gap-3">
            <span className="w-8 h-8 rounded-full bg-surface-container-high flex items-center justify-center text-sm font-mono text-primary">
              05
            </span>
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
              href="https://huma-lang.org/packages" 
              target="_blank"
              className="relative z-10 bg-primary text-on-primary px-10 py-5 rounded-xl font-bold text-sm hover:scale-105 transition-all shadow-xl shadow-primary/25 shrink-0"
            >
              {locale === "tr" ? "Kütüphaneleri Keşfet" : "Explore Libraries"}
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
    </>
  );
}
