import Link from "next/link";
import Navbar from "@/components/Navbar";
import type { Metadata } from "next";
import { getDictionary } from "@/dictionaries/dictionaries";

export const metadata: Metadata = {
  title: "Hüma IDE",
  description:
    "The official desktop environment for Hüma. Integrated terminal, Monaco editing, and seamless toolchain support built with Tauri & React.",
};

const GITHUB_URL = "https://github.com/VastSea0/huma-lang";

export default async function IDEPage({
  params,
}: {
  params: Promise<{ locale: string }>;
}) {
  const { locale } = await params;
  const dict = await getDictionary(locale as "en" | "tr");

  return (
    <>
      <Navbar dict={dict} locale={locale} />
      <main className="pt-32 pb-24 overflow-hidden">
        {/* ── Hero ── */}
        <section className="max-w-[1440px] mx-auto px-8 md:px-12 flex flex-col items-center text-center">
          <div className="inline-flex items-center gap-2 px-3 py-1 rounded-full bg-surface-container-high border border-outline-variant/20 mb-8">
            <span className="w-2 h-2 rounded-full bg-tertiary" />
            <span className="font-mono text-[10px] tracking-widest uppercase text-tertiary">
              {dict.IDE_Page.badge}
            </span>
          </div>

          <h1 className="text-[clamp(3rem,8vw,5.5rem)] font-black leading-[0.9] tracking-tighter text-on-surface mb-8">
            {dict.IDE_Page.headline} <br />
            <span className="bg-gradient-to-r from-secondary to-primary bg-clip-text text-transparent">
              {dict.IDE_Page.subheadline}
            </span>
          </h1>

          <p className="text-lg md:text-xl text-on-surface-variant max-w-2xl mb-12 leading-relaxed">
            {dict.IDE_Page.description}
          </p>

          <div className="flex flex-wrap gap-4 justify-center">
            <Link href={`${GITHUB_URL}/releases`} target="_blank">
              <button className="bg-gradient-to-tr from-primary to-primary-container text-on-primary px-8 py-4 font-bold rounded-sm shadow-xl active:scale-95 transition-all flex items-center gap-2">
                <span className="material-symbols-outlined text-[1.2em]">download</span>
                {dict.IDE_Page.cta_download}
              </button>
            </Link>
            <Link href={`/${locale}/docs/cli`}>
              <button className="px-8 py-4 font-bold rounded-sm border border-outline/20 text-on-surface hover:bg-surface-bright transition-all flex items-center gap-2">
                <span className="material-symbols-outlined text-[1.2em]">menu_book</span>
                {dict.IDE_Page.cta_docs}
              </button>
            </Link>
          </div>
        </section>

        {/* ── Visual Preview ── */}
        <section className="mt-20 max-w-[1200px] mx-auto px-8 md:px-12 relative">
          <div className="absolute top-1/2 left-1/2 -translate-x-1/2 -translate-y-1/2 w-full h-[150%] max-h-[800px] bg-primary/10 blur-[150px] -z-10 rounded-full" />
          <div className="glass-card rounded-md border border-outline-variant/20 overflow-hidden shadow-2xl ring-1 ring-white/5 bg-[#131313]/90">
            {/* Fake Mac Toolbar */}
            <div className="bg-surface-container-low/80 backdrop-blur-md px-4 py-3 flex items-center justify-between border-b border-outline-variant/20">
               <div className="flex gap-1.5">
                  <div className="w-2.5 h-2.5 rounded-full bg-red-500" />
                  <div className="w-2.5 h-2.5 rounded-full bg-yellow-500" />
                  <div className="w-2.5 h-2.5 rounded-full bg-green-500" />
               </div>
               <div className="font-mono text-xs text-on-surface-variant/40">huma-ide</div>
               <div className="w-8" />
            </div>
            {/* Editor Image representation */}
            <div className="relative p-8 h-[500px] flex items-center justify-center bg-surface-container-lowest">
               <div className="absolute inset-0 bg-[url('data:image/svg+xml;base64,PHN2ZyB4bWxucz0iaHR0cDovL3d3dy53My5vcmcvMjAwMC9zdmciIHdpZHRoPSI0IiBoZWlnaHQ9IjQiPjxyZWN0IHdpZHRoPSI0IiBoZWlnaHQ9IjQiIGZpbGw9IiMxMzEzMTMiLz48cmVjdCB3aWR0aD0iMSIgaGVpZ2h0PSIxIiBmaWxsPSIjMWYxZjFmIi8+PC9zdmc+')] opacity-50 z-0"></div>
               <div className="z-10 bg-surface-container p-6 rounded-md shadow-2xl flex items-center gap-4 border border-outline-variant/10 group cursor-pointer hover:border-primary/50 transition-colors">
                  <span className="material-symbols-outlined text-4xl text-primary animate-pulse">open_in_browser</span>
                  <div>
                    <h3 className="text-xl font-bold font-mono">React + Vite + Tauri = 🚀</h3>
                    <p className="text-on-surface-variant font-body">Launch the local IDE build to see the realtime Monaco editor & Terminal in action.</p>
                  </div>
               </div>
            </div>
          </div>
        </section>

        {/* ── Features ── */}
        <section className="mt-48 max-w-[1440px] mx-auto px-8 md:px-12">
          <div className="mb-16">
            <h2 className="text-3xl font-black tracking-tight mb-4 text-center">
              {dict.IDE_Page.features_headline}
            </h2>
            <div className="w-12 h-1 bg-primary mx-auto" />
          </div>

          <div className="grid grid-cols-1 md:grid-cols-3 gap-6">
            {[
              {
                icon: "data_object",
                title: dict.IDE_Page.features.monaco.title,
                desc: dict.IDE_Page.features.monaco.desc,
              },
              {
                icon: "terminal",
                title: dict.IDE_Page.features.terminal.title,
                desc: dict.IDE_Page.features.terminal.desc,
              },
              {
                icon: "deployed_code",
                title: dict.IDE_Page.features.tauri.title,
                desc: dict.IDE_Page.features.tauri.desc,
              },
            ].map((f) => (
              <div
                key={f.title}
                className="bg-surface-container-lowest p-10 hover:bg-surface-container-low transition-colors group rounded-xl border border-outline-variant/10"
              >
                <div className="w-12 h-12 flex items-center justify-center bg-surface-container rounded-sm mb-6 group-hover:bg-primary/10 transition-colors">
                  <span className="material-symbols-outlined text-primary">
                    {f.icon}
                  </span>
                </div>
                <h3 className="text-xl font-bold mb-3">{f.title}</h3>
                <p className="text-on-surface-variant leading-relaxed text-sm">
                  {f.desc}
                </p>
              </div>
            ))}
          </div>
        </section>
      </main>

      <footer className="bg-[#0E0E0E] border-t border-[#5A413A]/20">
        <div className="flex justify-between items-center px-12 py-12 max-w-[1440px] mx-auto">
          <p className="text-xs text-on-surface-variant/60 uppercase tracking-widest font-mono">
             {dict.IDE_Page.badge} • hüma-ide 
          </p>
          <div className="flex gap-8 font-body text-xs uppercase tracking-[0.1em]">
            <Link href={GITHUB_URL} target="_blank" rel="noopener noreferrer"
              className="text-on-surface-variant/60 hover:text-primary transition-colors">{dict.Footer.github}</Link>
            <Link href={`${GITHUB_URL}/issues`} target="_blank" rel="noopener noreferrer"
              className="text-on-surface-variant/60 hover:text-primary transition-colors">{dict.Footer.issues}</Link>
            <Link href={`${GITHUB_URL}/releases`} target="_blank" rel="noopener noreferrer"
              className="text-on-surface-variant/60 hover:text-primary transition-colors">{dict.Footer.releases}</Link>
            <Link href={`${GITHUB_URL}/blob/main/LICENSE`} target="_blank" rel="noopener noreferrer"
              className="text-on-surface-variant/60 hover:text-primary transition-colors">{dict.Footer.license_link}</Link>
          </div>
        </div>
      </footer>
    </>
  );
}
