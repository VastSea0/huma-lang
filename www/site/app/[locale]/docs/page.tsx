import Link from "next/link";
import CodeBlock from "@/components/CodeBlock";
import type { Metadata } from "next";
import { getDictionary } from "@/dictionaries/dictionaries";

export const metadata: Metadata = {
  title: "Getting Started",
  description:
    "Build Hüma from source and follow the verified getting-started guide.",
};

const installCode = `curl -fsSL https://raw.githubusercontent.com/VastSea0/huma-lang/main/install.sh | sh`;

const versionCode = `$ huma --version
Hüma 0.6.0`;

export default async function DocsPage({
  params,
}: {
  params: Promise<{ locale: string }>;
}) {
  const { locale } = await params;
  const dict = await getDictionary(locale as "en" | "tr");
  const d = dict.Docs.getting_started;

  const getPath = (path: string) => `/${locale}${path}`;

  return (
    <>
      {/* Main content */}
      <main className="flex-1 px-8 md:px-16 py-12 max-w-4xl">
        {/* Breadcrumb */}
        <nav className="flex gap-2 text-[10px] uppercase tracking-widest text-on-surface-variant/60 mb-4">
          <Link href={getPath("/docs")} className="hover:text-primary transition-colors">
            {dict.Nav.docs}
          </Link>
          <span>/</span>
          <span className="text-on-surface-variant">{dict.Sidebar.core}</span>
          <span>/</span>
          <span className="text-primary">{d.title}</span>
        </nav>

        <h1 className="text-5xl font-extrabold text-on-surface tracking-tighter mb-6">
          {d.hero_title}
        </h1>
        <p className="text-lg text-on-surface-variant leading-relaxed mb-12">
          {d.hero_desc}
        </p>

        {/* 01 Installation */}
        <section className="mb-16" id="installation">
          <h2 className="text-2xl font-bold text-on-surface mb-6 flex items-center gap-3">
            <span className="w-8 h-8 rounded-full bg-surface-container-high flex items-center justify-center text-sm font-mono text-primary">
              01
            </span>
            {d.installation.title}
          </h2>
          <p className="mb-6 text-on-surface-variant">
            {d.installation.desc}
          </p>
          <CodeBlock code={installCode} variant="terminal" />
          <CodeBlock
            code={`$ huma --version\nHüma 0.6.0`}
            variant="terminal"
          />

          {/* Info callout */}
          <div className="bg-primary/5 border-l-4 border-primary p-6 rounded-r-lg mb-8">
            <div className="flex items-center gap-3 mb-2 text-primary">
              <span className="material-symbols-outlined text-lg">info</span>
              <span className="text-xs font-bold uppercase tracking-widest">
                {d.installation.arch_title}
              </span>
            </div>
            <p className="text-sm text-on-surface-variant leading-relaxed">
              {d.installation.arch_desc}
            </p>
          </div>
        </section>

        {/* 02 Quick Start & REPL */}
        <section className="mb-16" id="quick-start">
          <h2 className="text-2xl font-bold text-on-surface mb-6 flex items-center gap-3">
            <span className="w-8 h-8 rounded-full bg-surface-container-high flex items-center justify-center text-sm font-mono text-primary">
              02
            </span>
            {d.quick_start.title}
          </h2>
          <p className="mb-6 text-on-surface-variant">
            {d.quick_start.desc}
          </p>
          <div className="space-y-4">
            <CodeBlock code={versionCode} variant="terminal" />
            <CodeBlock 
              code={`$ huma repl\n\nhüma ❯ "merhaba world"'u yazdır\nmerhaba world\nhüma ❯ çıkış`} 
              variant="terminal" 
            />
          </div>
        </section>

        {/* 03 First Project */}
        <section className="mb-16" id="first-project">
          <h2 className="text-2xl font-bold text-on-surface mb-6 flex items-center gap-3">
            <span className="w-8 h-8 rounded-full bg-surface-container-high flex items-center justify-center text-sm font-mono text-primary">
              03
            </span>
            {d.first_project.title}
          </h2>
          <p className="mb-6 text-on-surface-variant">
            {d.first_project.desc}
          </p>
          
          <div className="space-y-6">
            <div>
              <h4 className="text-xs font-bold uppercase tracking-widest text-primary mb-3">1. {locale === "tr" ? "Projeyi Oluştur" : "Create Project"}</h4>
              <CodeBlock code={`$ huma paket yeni merhaba_projesi\n\n[Başarılı!] 'merhaba_projesi' projesi oluşturuldu.`} variant="terminal" />
            </div>

            <div>
              <h4 className="text-xs font-bold uppercase tracking-widest text-primary mb-3">2. {locale === "tr" ? "Kodu Düzenle" : "Edit Code"}</h4>
              <CodeBlock 
                filename="merhaba_projesi/merhaba_projesi.hb"
                code={`// Değişken tanımlayıp koşullu mantık\npuan = 85 olsun\n\npuan > 50 ise {\n    "Başarılı!"'yı yazdır;\n} yoksa {\n    "Tekrar dene."'i yazdır;\n}`} 
              />
            </div>

            <div>
              <h4 className="text-xs font-bold uppercase tracking-widest text-primary mb-3">3. {locale === "tr" ? "Çalıştır" : "Run Project"}</h4>
              <CodeBlock code={`$ cd merhaba_projesi\n$ huma paket run baslat\n\n[Hüma] baslat betiği çalıştırılıyor...\nBaşarılı!`} variant="terminal" />
            </div>
          </div>

          {/* Suffix Callout */}
          <div className="bg-tertiary/5 border-l-4 border-tertiary p-6 rounded-r-lg mt-8">
            <div className="flex items-center gap-3 mb-2 text-tertiary">
              <span className="material-symbols-outlined text-lg">package_2</span>
              <span className="text-xs font-bold uppercase tracking-widest">
                {d.first_project.suffix_title}
              </span>
            </div>
            <p className="text-sm text-on-surface-variant leading-relaxed">
              {d.first_project.suffix_desc}
            </p>
          </div>
        </section>

        {/* Next steps */}
        <section id="next-steps">
          <h2 className="text-2xl font-bold text-on-surface mb-6 flex items-center gap-3">
            <span className="w-8 h-8 rounded-full bg-surface-container-high flex items-center justify-center text-sm font-mono text-primary">
              04
            </span>
            {d.next_steps.title}
          </h2>
          <div className="grid grid-cols-1 sm:grid-cols-2 gap-4">
            {[
              {
                href: "/docs/syntax",
                icon: "code",
                title: d.next_steps.syntax.title,
                desc: d.next_steps.syntax.desc,
              },
              {
                href: "/docs/functions-classes",
                icon: "data_object",
                title: d.next_steps.functions.title,
                desc: d.next_steps.functions.desc,
              },
              {
                href: "/docs/lists-errors",
                icon: "list",
                title: d.next_steps.lists.title,
                desc: d.next_steps.lists.desc,
              },
              {
                href: "/docs/stdlib",
                icon: "menu_book",
                title: d.next_steps.stdlib.title,
                desc: d.next_steps.stdlib.desc,
              },
            ].map((card) => (
              <Link
                key={card.href}
                href={getPath(card.href)}
                className="flex items-start gap-4 p-5 bg-surface-container-low rounded-lg border border-outline-variant/10 hover:border-primary/30 hover:bg-surface-container transition-all group"
              >
                <span className="material-symbols-outlined text-primary mt-0.5">
                  {card.icon}
                </span>
                <div>
                  <div className="font-bold text-sm text-on-surface group-hover:text-primary transition-colors">
                    {card.title}
                  </div>
                  <div className="text-xs text-on-surface-variant mt-1">
                    {card.desc}
                  </div>
                </div>
              </Link>
            ))}
          </div>
        </section>
      </main>

      {/* Right TOC */}
      <aside className="hidden xl:block w-64 sticky top-16 h-[calc(100vh-4rem)] py-12 px-8 overflow-y-auto border-l border-outline-variant/10 shrink-0">
        <h5 className="text-[10px] font-bold text-on-surface uppercase tracking-[0.2em] mb-6 opacity-40">
          {d.toc.title}
        </h5>
        <ul className="space-y-4 text-[11px] font-bold uppercase tracking-widest">
          {[
            { href: "#installation", label: d.toc.installation },
            { href: "#quick-start", label: d.toc.quick_start },
            { href: "#first-project", label: d.toc.first_project },
            { href: "#next-steps", label: d.toc.next_steps },
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
          <div className="bg-surface-container-high/30 p-4 rounded-lg border border-outline-variant/5">
            <p className="text-[10px] font-bold uppercase tracking-widest text-on-surface-variant/40 mb-3">
              {d.resources.title}
            </p>
            <div className="flex flex-col gap-3 text-[11px] font-bold uppercase tracking-widest">
              <a href="https://github.com/VastSea0/huma-lang/discussions" className="flex items-center gap-2 text-primary hover:translate-x-1 transition-transform">
                <span className="material-symbols-outlined text-[14px]">chat</span>
                {d.resources.discord}
              </a>
              <a href="https://github.com/VastSea0/huma-lang/issues" className="flex items-center gap-2 text-primary hover:translate-x-1 transition-transform">
                <span className="material-symbols-outlined text-[14px]">bug_report</span>
                {d.resources.issue}
              </a>
            </div>
          </div>
        </div>
      </aside>
    </>
  );
}
