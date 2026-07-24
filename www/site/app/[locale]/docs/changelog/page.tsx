import { getDictionary } from "@/dictionaries/dictionaries";
import Link from "next/link";
import type { Metadata } from "next";

export const metadata: Metadata = {
  title: "Changelog",
  description: "Official update history of the Hüma language and toolchain.",
};

export default async function ChangelogPage({
  params,
}: {
  params: Promise<{ locale: string }>;
}) {
  const { locale } = await params;
  const dict = await getDictionary(locale as "en" | "tr");
  const ch = dict.Docs.changelog;

  const versions = ["v0_6_0_alfa_3", "v0_6_0_alfa_2", "v0_6_0_alfa_1", "v0_5_2", "v0_5_1", "v0_5_0", "v0_4_0", "v0_3_1", "v0_2_0", "v0_1_0"];

  const getPath = (path: string) => `/${locale}${path}`;

  return (
    <>
      <main className="flex-1 px-8 md:px-16 py-12 max-w-4xl">
        <nav className="flex gap-2 text-[10px] uppercase tracking-widest text-on-surface-variant/60 mb-4">
          <Link href={getPath("/docs")} className="hover:text-primary transition-colors">
            {dict.Nav.docs}
          </Link>
          <span>/</span>
          <span className="text-on-surface-variant">{dict.Sidebar.community}</span>
          <span>/</span>
          <span className="text-primary">{ch.title}</span>
        </nav>

        <h1 className="text-5xl font-extrabold text-on-surface tracking-tighter mb-6">
          {ch.hero_title}
        </h1>
        <p className="text-lg text-on-surface-variant leading-relaxed mb-16">
          {ch.hero_desc}
        </p>

        <div className="relative space-y-24">
          {/* Vertical line for timeline */}
          <div className="absolute left-0 top-4 bottom-0 w-px bg-outline-variant/20 ml-[7px] hidden md:block" />

          {versions.map((vKey) => {
            const v = (ch as any)[vKey];
            if (!v) return null;

            return (
              <section key={vKey} className="relative md:pl-12" id={vKey}>
                {/* Timeline Dot */}
                <div className="absolute left-0 top-3 w-4 h-4 rounded-full border-2 border-primary bg-surface-container-lowest z-10 hidden md:block shadow-sm" />
                
                <div className="flex flex-col md:flex-row md:items-baseline justify-between mb-8">
                  <h2 className="text-3xl font-black text-on-surface tracking-tight">
                    {v.version}
                  </h2>
                  <span className="text-[10px] font-bold text-on-surface-variant/40 uppercase tracking-[0.2em] mt-2 md:mt-0 bg-surface-container-high px-3 py-1 rounded-full border border-outline-variant/10">
                    {v.date}
                  </span>
                </div>

                <div className="bg-surface-container-lowest rounded-2xl border border-outline-variant/10 p-8 shadow-sm hover:border-primary/20 transition-colors">
                  <ul className="space-y-5">
                    {v.changes.map((change: string, idx: number) => (
                      <li key={idx} className="flex gap-4 text-on-surface-variant leading-relaxed text-sm">
                        <span className="text-primary mt-1.5 shrink-0">
                          <div className="w-2 h-2 rounded-full bg-primary/40 group-hover:bg-primary transition-colors" />
                        </span>
                        <span>{change}</span>
                      </li>
                    ))}
                  </ul>
                </div>
              </section>
            );
          })}
        </div>

        {/* Raw commits link */}
        <div className="mt-32 p-10 rounded-3xl bg-surface-container-low border border-outline-variant/5 text-center relative overflow-hidden group">
          <div className="absolute inset-0 bg-primary/5 opacity-0 group-hover:opacity-100 transition-opacity"></div>
          <h3 className="text-xl font-bold text-on-surface mb-3 relative z-10">
            {locale === "tr" ? "Ham Veriye mi İhtiyacınız Var?" : "Need the Raw Data?"}
          </h3>
          <p className="text-sm text-on-surface-variant mb-8 max-w-md mx-auto relative z-10">
            {locale === "tr" 
              ? "Geliştirme sürecindeki her adımı görmek için son 100 git commit mesajını ham metin olarak inceleyebilirsiniz."
              : "You can examine the last 100 git commit messages as raw text to see every step in the development process."}
          </p>
          <a 
            href="/commits.txt" 
            target="_blank"
            className="relative z-10 inline-flex items-center gap-3 px-8 py-3.5 rounded-xl bg-primary text-on-primary font-bold text-sm shadow-lg shadow-primary/20 hover:scale-105 transition-all"
          >
            <span className="material-symbols-outlined text-base">history</span>
            {locale === "tr" ? "Commit Geçmişini Gör" : "View Commit History"}
          </a>
        </div>
      </main>

      {/* Right TOC */}
      <aside className="hidden xl:block w-64 sticky top-16 h-[calc(100vh-4rem)] py-12 px-8 overflow-y-auto border-l border-outline-variant/10 shrink-0">
        <h5 className="text-[10px] font-bold text-on-surface uppercase tracking-[0.2em] mb-6 opacity-40">
           {locale === "tr" ? "VERSİYONLAR" : "VERSIONS"}
        </h5>
        <ul className="space-y-4 text-[11px] font-bold uppercase tracking-widest">
          {versions.map((vKey) => {
             const v = (ch as any)[vKey];
             if (!v) return null;
             return (
               <li key={vKey}>
                 <a
                   href={`#${vKey}`}
                   className="text-on-surface-variant/60 hover:text-primary transition-all block"
                 >
                   {v.version.split(" ")[0]}
                 </a>
               </li>
             );
          })}
        </ul>
      </aside>
    </>
  );
}
