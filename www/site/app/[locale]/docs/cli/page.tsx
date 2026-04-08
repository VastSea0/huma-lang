import Link from "next/link";
import CodeBlock from "@/components/CodeBlock";
import type { Metadata } from "next";
import { getDictionary } from "@/dictionaries/dictionaries";

export const metadata: Metadata = {
  title: "CLI Command Reference",
  description: "Comprehensive guide for all Hüma command-line interface tools and subcommands.",
};

export default async function CLIPage({
  params,
}: {
  params: Promise<{ locale: string }>;
}) {
  const { locale } = await params;
  const dict = await getDictionary(locale as "en" | "tr");
  const cli = dict.Docs.cli;

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
          <span className="text-primary">{cli.title}</span>
        </nav>

        <h1 className="text-5xl font-extrabold text-on-surface tracking-tighter mb-6">
          {cli.hero_title}
        </h1>
        <p className="text-lg text-on-surface-variant leading-relaxed mb-12">
          {cli.hero_desc}
        </p>

        {/* Execution */}
        <section className="mb-16" id="execution">
          <h2 className="text-2xl font-bold text-on-surface mb-6 flex items-center gap-3">
            <span className="w-8 h-8 rounded-full bg-surface-container-high flex items-center justify-center text-sm font-mono text-primary">
              01
            </span>
            {cli.execution.title}
          </h2>
          <div className="space-y-4 mb-6">
            <p className="text-on-surface-variant text-sm font-medium">{cli.execution.run}</p>
            <CodeBlock variant="terminal" code="$ huma run index.hb" />
            
            <p className="text-on-surface-variant text-sm font-medium">{cli.execution.repl}</p>
            <CodeBlock variant="terminal" code="$ huma repl" />

            <p className="text-on-surface-variant text-sm font-medium">{cli.execution.ide}</p>
            <CodeBlock variant="terminal" code="$ huma ide" />
          </div>
        </section>

        {/* Compilation */}
        <section className="mb-16" id="compilation">
          <h2 className="text-2xl font-bold text-on-surface mb-6 flex items-center gap-3">
            <span className="w-8 h-8 rounded-full bg-surface-container-high flex items-center justify-center text-sm font-mono text-primary">
              02
            </span>
            {cli.compilation.title}
          </h2>
          <div className="space-y-4 mb-6">
            <p className="text-on-surface-variant text-sm font-medium">{cli.compilation.build}</p>
            <CodeBlock variant="terminal" code="$ huma build main.hb" />
            
            <p className="text-on-surface-variant text-sm font-medium">{cli.compilation.exec}</p>
            <CodeBlock variant="terminal" code="$ huma exec main.hbc" />

            <p className="text-on-surface-variant text-sm font-medium">{cli.compilation.gen}</p>
            <CodeBlock variant="terminal" code="$ huma gen main.hb" />
          </div>
        </section>

        {/* Package Management */}
        <section className="mb-16" id="package">
          <h2 className="text-2xl font-bold text-on-surface mb-6 flex items-center gap-3">
            <span className="w-8 h-8 rounded-full bg-surface-container-high flex items-center justify-center text-sm font-mono text-primary">
              03
            </span>
            {cli.package.title}
          </h2>
          <div className="space-y-4 mb-6">
            <p className="text-on-surface-variant text-sm font-medium">{cli.package.init}</p>
            <CodeBlock variant="terminal" code="$ huma paket ilkle" />
            
            <p className="text-on-surface-variant text-sm font-medium">{cli.package.install}</p>
            <CodeBlock variant="terminal" code="$ huma paket kur" />

            <p className="text-on-surface-variant text-sm font-medium">{cli.package.list}</p>
            <CodeBlock variant="terminal" code="$ huma paket liste" />
          </div>
        </section>

        {/* Maintenance */}
        <section className="mb-16" id="maintenance">
          <h2 className="text-2xl font-bold text-on-surface mb-6 flex items-center gap-3">
            <span className="w-8 h-8 rounded-full bg-surface-container-high flex items-center justify-center text-sm font-mono text-primary">
              04
            </span>
            {cli.maintenance.title}
          </h2>
          <div className="space-y-4 mb-6">
            <p className="text-on-surface-variant text-sm font-medium">{cli.maintenance.update}</p>
            <CodeBlock variant="terminal" code="$ huma güncelle" />
            
            <p className="text-on-surface-variant text-sm font-medium">{cli.maintenance.version}</p>
            <CodeBlock variant="terminal" code="$ huma --version" />
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
            {dict.Sidebar.items.syntax}
          </Link>
          <Link
            href={getPath("/docs/functions-classes")}
            className="flex items-center gap-2 text-sm text-on-surface-variant hover:text-primary transition-colors font-bold uppercase tracking-widest text-[10px]"
          >
            {dict.Sidebar.items.functions}
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
            { href: "#execution", label: cli.execution.title },
            { href: "#compilation", label: cli.compilation.title },
            { href: "#package", label: cli.package.title },
            { href: "#maintenance", label: cli.maintenance.title },
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
