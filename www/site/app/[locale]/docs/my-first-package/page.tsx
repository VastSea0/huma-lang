import Link from "next/link";
import { getDictionary } from "@/dictionaries/dictionaries";
import CodeBlock from "@/components/CodeBlock";
import type { Metadata } from "next";

export const metadata: Metadata = {
  title: "My First Hüma Package",
  description:
    "A step-by-step guide to creating a Hüma library and publishing it via GitHub.",
};

const packageJsonCode = `{
  "ad": "benim_kutuphanem",
  "surum": "1.0.0",
  "aciklama": "Hüma için ilk kütüphanem",
  "yazar": "Adınız",
  "giris": "benim_kutuphanem.hb"
}`;

const myCode = `// benim_kutuphanem.hb

merhaba_de fonksiyon olsun {
    "Merhaba Hüma Dünyası!"'ı yazdır;
}

topla fonksiyon olsun a, b alsın {
    (a + b)'yi döndür;
}`;

export default async function MyFirstPackagePage({
  params,
}: {
  params: Promise<{ locale: string }>;
}) {
  const { locale } = await params;
  const dict = await getDictionary(locale as "en" | "tr");
  const g = dict.Docs.guides.my_first_package;

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
          <span className="text-primary">{g.title}</span>
        </nav>

        <h1 className="text-5xl font-extrabold text-on-surface tracking-tighter mb-6">
          {g.hero_title}
        </h1>
        <p className="text-lg text-on-surface-variant leading-relaxed mb-12">
          {g.hero_desc}
        </p>

        <div className="space-y-24">
          {/* Step 1 */}
          <section id="step-1" className="scroll-mt-24">
            <h2 className="text-2xl font-bold text-on-surface mb-6">
              {locale === "tr" ? "Projeyi Başlatma" : "Initialization"}
            </h2>
            <p className="text-on-surface-variant mb-6 leading-relaxed">
              {locale === "tr"
                ? "Hüma CLI, kütüphane projeleri için hazır bir şablon sunar. Terminalde projeniz için bir dizin oluşturmakla başlayın:"
                : "The Hüma CLI offers a ready-made template for library projects. Start by creating a directory for your project in the terminal:"}
            </p>
            <CodeBlock code="huma paket yeni benim_kutuphanem" variant="terminal" />
          </section>

          {/* Step 2 */}
          <section id="step-2" className="scroll-mt-24">
            <h2 className="text-2xl font-bold text-on-surface mb-6">
              Metadata (paket.json)
            </h2>
            <p className="text-on-surface-variant mb-6 leading-relaxed">
              {locale === "tr"
                ? "Oluşturulan dizine girdiğinizde bir 'paket.json' dosyası göreceksiniz. Bu dosya, kütüphanenizin kimliği ve bağımlılıklarını tutar."
                : "When you enter the created directory, you will see a 'paket.json' file. This file holds the identity and dependencies of your library."}
            </p>
            <CodeBlock code={packageJsonCode} filename="paket.json" />
          </section>

          {/* Step 3 */}
          <section id="step-3" className="scroll-mt-24">
            <h2 className="text-2xl font-bold text-on-surface mb-6">
              {locale === "tr" ? "Mantık Yazımı" : "Writing Logic"}
            </h2>
            <p className="text-on-surface-variant mb-6 leading-relaxed">
              {locale === "tr"
                ? "Modül dosyanızı açın ve dışarıya açmak istediğiniz fonksiyonları tanımlayın. Hüma'nın temiz ek sistemiyle kodunuzu yazın."
                : "Open your module file and define the functions you want to export. Write your code with Hüma's clean suffix system."}
            </p>
            <CodeBlock code={myCode} filename="benim_kutuphanem.hb" />
          </section>

          {/* Step 4 */}
          <section id="step-4" className="scroll-mt-24">
            <h2 className="text-2xl font-bold text-on-surface mb-6">
              {locale === "tr" ? "Yayınlama (GitHub)" : "Publishing (GitHub)"}
            </h2>
            <p className="text-on-surface-variant mb-6 leading-relaxed">
              {locale === "tr"
                ? "Paketinizi GitHub'a yükleyin. 'huma paket kur' komutu doğrudan GitHub URL'lerini destekler, böylece dünyayla paylaşabilirsiniz."
                : "Upload your package to GitHub. The 'huma paket kur' command directly supports GitHub URLs, allowing you to share it with the world."}
            </p>
            <div className="bg-surface-container-low p-6 rounded-2xl border border-outline-variant/10 text-center">
               <p className="text-xs text-on-surface-variant uppercase tracking-widest font-bold mb-4">{locale === "tr" ? "KURULUM KOMUTU" : "INSTALLATION COMMAND"}</p>
               <code className="text-primary font-mono text-sm font-bold">huma paket kur github.com/kullanici/repo</code>
            </div>
          </section>
        </div>

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
            {dict.Docs.changelog.title}
            <span className="material-symbols-outlined text-base">arrow_forward</span>
          </Link>
        </div>
      </main>

      {/* Right TOC */}
      <aside className="hidden xl:block w-64 sticky top-16 h-[calc(100vh-4rem)] py-12 px-8 overflow-y-auto border-l border-outline-variant/10 shrink-0">
        <h5 className="text-[10px] font-bold text-on-surface uppercase tracking-[0.2em] mb-6 opacity-40">
          {locale === "tr" ? "ADIMLAR" : "STEPS"}
        </h5>
        <ul className="space-y-4 text-[11px] font-bold uppercase tracking-widest">
          {[
            { href: "#step-1", label: locale === "tr" ? "Başlatma" : "Initialization" },
            { href: "#step-2", label: "Metadata" },
            { href: "#step-3", label: locale === "tr" ? "Kod Yazımı" : "Writing Code" },
            { href: "#step-4", label: locale === "tr" ? "Yayınlama" : "Publishing" },
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
