import Link from "next/link";
import { getDictionary } from "@/dictionaries/dictionaries";
import CodeBlock from "@/components/CodeBlock";
import type { Metadata } from "next";

export const metadata: Metadata = {
  title: "Hüma IDE",
  description: "Official Hüma desktop editor and development environment.",
};

const buildCode = `# 1. Enter the ide folder and install dependencies
cd ide
npm install

# 2. Run the Tauri dev server
npm run tauri dev

# 3. Build a standalone desktop release
npm run tauri build`;

export default async function IdePage({
  params,
}: {
  params: Promise<{ locale: string }>;
}) {
  const { locale } = await params;
  const dict = await getDictionary(locale as "en" | "tr");
  const i = dict.Docs.ide;

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
          <span className="text-on-surface-variant">{dict.Sidebar.tooling}</span>
          <span>/</span>
          <span className="text-primary">{i.title}</span>
        </nav>

        {/* Title & Description */}
        <h1 className="text-5xl font-extrabold text-on-surface tracking-tighter mb-6">
          {i.hero_title}
        </h1>
        <p className="text-lg text-on-surface-variant leading-relaxed mb-12">
          {i.hero_desc}
        </p>

        {/* Architecture */}
        <section className="mb-16" id="architecture">
          <h2 className="text-2xl font-bold text-on-surface mb-4">
            {i.architecture.title}
          </h2>
          <p className="text-on-surface-variant leading-relaxed mb-8">
            {i.architecture.desc}
          </p>

          <div className="overflow-x-auto bg-surface-container-low rounded-lg border border-outline-variant/10">
            <table className="w-full text-sm border-collapse">
              <thead>
                <tr className="border-b border-outline-variant/20">
                  <th className="text-left py-3 px-5 text-on-surface-variant/60 font-bold text-[10px] uppercase tracking-widest">
                    {locale === "tr" ? "Katman" : "Layer"}
                  </th>
                  <th className="text-left py-3 px-5 text-on-surface-variant/60 font-bold text-[10px] uppercase tracking-widest">
                    {locale === "tr" ? "Teknoloji" : "Technology"}
                  </th>
                  <th className="text-left py-3 px-5 text-on-surface-variant/60 font-bold text-[10px] uppercase tracking-widest">
                    {locale === "tr" ? "Açıklama" : "Description"}
                  </th>
                </tr>
              </thead>
              <tbody className="divide-y divide-outline-variant/10">
                <tr className="hover:bg-surface-container-lowest transition-colors">
                  <td className="py-4 px-5">
                    <code className="text-primary font-mono text-xs font-bold">Frontend</code>
                  </td>
                  <td className="py-4 px-5 text-on-surface-variant text-sm">Vite + React</td>
                  <td className="py-4 px-5 text-on-surface-variant text-sm leading-relaxed">
                    {locale === "tr"
                      ? "Monaco Editor ile güçlü kod düzenleme ve Xterm.js ile entegre terminal sunar."
                      : "Offers powerful code editing with Monaco Editor and integrated terminal with Xterm.js."}
                  </td>
                </tr>
                <tr className="hover:bg-surface-container-lowest transition-colors">
                  <td className="py-4 px-5">
                    <code className="text-primary font-mono text-xs font-bold">Backend</code>
                  </td>
                  <td className="py-4 px-5 text-on-surface-variant text-sm">Tauri + Rust</td>
                  <td className="py-4 px-5 text-on-surface-variant text-sm leading-relaxed">
                    {locale === "tr"
                      ? "Yerel dosya sistemine güvenli erişim ve işletim sistemi pencere yönetimi sağlar."
                      : "Provides secure access to the local filesystem and OS window management."}
                  </td>
                </tr>
              </tbody>
            </table>
          </div>
        </section>

        {/* Build from Source */}
        <section className="mb-16" id="build">
          <h2 className="text-2xl font-bold text-on-surface mb-4">
            {locale === "tr" ? "Kaynaktan Derleme" : "Build from Source"}
          </h2>
          <p className="text-on-surface-variant leading-relaxed mb-6">
            {locale === "tr"
              ? "IDE'yi kendi makinenizde çalıştırmak veya geliştirmek için Node.js ve Rust araçlarının kurulu olması gerekir."
              : "To run or develop the IDE on your own machine, Node.js and Rust tools must be installed."}
          </p>
          <CodeBlock code={buildCode} variant="terminal" />
        </section>

        {/* Requirements Note */}
        <div className="bg-tertiary/5 border-l-4 border-tertiary p-8 rounded-r-2xl">
          <div className="flex items-center gap-3 mb-4 text-tertiary">
            <span className="material-symbols-outlined text-2xl">info</span>
            <h3 className="text-lg font-bold">
               {locale === "tr" ? "Gereksinimler" : "Requirements"}
            </h3>
          </div>
          <p className="text-on-surface-variant text-sm leading-relaxed">
            {locale === "tr"
              ? "Tauri derlemesi için işletim sisteminize özel bazı kütüphaneler (Linux'ta webkit2gtk gibi) gerekebilir. Lütfen Tauri belgelerine göz atın."
              : "Some OS-specific libraries (like webkit2gtk on Linux) may be required for Tauri build. Please check the Tauri documentation."}
          </p>
        </div>

        {/* Navigation */}
        <div className="flex justify-between mt-24 pt-8 border-t border-outline-variant/10">
          <Link
            href={getPath("/docs/compiler")}
            className="flex items-center gap-2 text-on-surface-variant hover:text-primary transition-colors font-bold uppercase tracking-widest text-[10px]"
          >
            <span className="material-symbols-outlined text-base">arrow_back</span>
            {dict.Docs.compiler.title}
          </Link>
          <Link
            href={getPath("/docs/my-first-package")}
            className="flex items-center gap-2 text-on-surface-variant hover:text-primary transition-colors font-bold uppercase tracking-widest text-[10px]"
          >
            {dict.Sidebar.items.my_first_package}
            <span className="material-symbols-outlined text-base">arrow_forward</span>
          </Link>
        </div>
      </main>
    </>
  );
}
