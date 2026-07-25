import Link from "next/link";
import type { Metadata } from "next";
import CodeBlock from "@/components/CodeBlock";
import { getDictionary } from "@/dictionaries/dictionaries";

export const metadata: Metadata = {
  title: "Hüma Package Manager",
  description:
    "Verified commands, manifest fields, lock file behavior, and security limits of the Hüma package manager.",
};

const manifestExample = `{
  "ad": "ornek_paket",
  "surum": "1.0.0",
  "aciklama": "Örnek Hüma paketi",
  "yazar": "Geliştirici",
  "giris": "ana.hb",
  "huma_surum": ">=0.6.0",
  "bagimliliklar": {},
  "betikler": {
    "test": "huma test tests"
  }
}`;

export default async function PackageManagerPage({
  params,
}: {
  params: Promise<{ locale: string }>;
}) {
  const { locale } = await params;
  const dict = await getDictionary(locale as "en" | "tr");
  const tr = locale === "tr";
  const getPath = (path: string) => `/${locale}${path}`;

  const commands = [
    ["huma ilkle", tr ? "Geçerli dizinde proje dosyalarını oluşturur." : "Initializes project files in the current directory."],
    ["huma yeni <ad>", tr ? "Yeni bir proje dizini oluşturur." : "Creates a new project directory."],
    ["huma kur [paket]", tr ? "Kaynak ağacında dağıtılan yerel bağımlılıkları kurar." : "Installs local dependencies distributed in the source tree."],
    ["huma listele", tr ? "Kurulu paketleri listeler." : "Lists installed packages."],
    ["huma paket sil <ad>", tr ? "Kurulu paketi kaldırır." : "Removes an installed package."],
    ["huma paket doğrula", tr ? "Manifest ve kilit dosyası tutarlılığını denetler." : "Checks manifest and lock-file consistency."],
    ["huma paket run <betik>", tr ? "Manifestteki güvenlik denetiminden geçen betiği çalıştırır." : "Runs a manifest script after its safety checks pass."],
  ];

  return (
    <main className="flex-1 px-8 py-12 md:px-16 max-w-4xl">
      <nav className="flex gap-2 text-[10px] uppercase tracking-widest text-on-surface-variant/60 mb-4">
        <Link href={getPath("/docs")} className="hover:text-primary">
          {dict.Nav.docs}
        </Link>
        <span>/</span>
        <span>{dict.Sidebar.tooling}</span>
        <span>/</span>
        <span className="text-primary">{dict.Sidebar.items.package_manager}</span>
      </nav>

      <h1 className="text-5xl font-extrabold tracking-tighter mb-6">
        {dict.Sidebar.items.package_manager}
      </h1>
      <p className="text-lg text-on-surface-variant leading-relaxed mb-14">
        {tr
          ? "0.6.0 paket yöneticisi yalnızca kaynak ağacında dağıtılan yerel paketleri kurar. İmzalı ve çok dosyalı bir kayıt protokolü hazır olmadığı için uzak kurulum kapalıdır."
          : "The 0.6.0 package manager installs only local packages distributed in the source tree. Remote installation is disabled until a signed, multi-file registry protocol exists."}
      </p>

      <section className="mb-20">
        <h2 className="text-2xl font-bold mb-7">
          {tr ? "Doğrulanmış komutlar" : "Verified commands"}
        </h2>
        <div className="grid gap-4 md:grid-cols-2">
          {commands.map(([command, description]) => (
            <div
              key={command}
              className="rounded-xl border border-outline-variant/10 bg-surface-container-low p-5"
            >
              <code className="text-primary text-xs font-bold">{command}</code>
              <p className="mt-3 text-xs text-on-surface-variant leading-relaxed">
                {description}
              </p>
            </div>
          ))}
        </div>
      </section>

      <section className="mb-20">
        <h2 className="text-2xl font-bold mb-5">huma.json</h2>
        <p className="text-on-surface-variant mb-7">
          {tr
            ? "ad, sürüm, açıklama, yazar ve giriş zorunlu alanlardır. Bağımlılıklar, betikler, Rust crate bağımlılıkları ve gömülü Rust kodu isteğe bağlıdır."
            : "Name, version, description, author, and entry point are required. Dependencies, scripts, Rust crates, and embedded Rust code are optional."}
        </p>
        <CodeBlock filename="huma.json" code={manifestExample} />
      </section>

      <section className="mb-20 rounded-2xl border border-primary/20 bg-primary/5 p-8">
        <h2 className="text-2xl font-bold mb-5">
          {tr ? "Güvenlik sınırları" : "Security boundaries"}
        </h2>
        <ul className="list-disc pl-5 space-y-3 text-sm text-on-surface-variant">
          <li>
            {tr
              ? "Paket adları ve hedef yolları dizin geçişi kalıplarına karşı doğrulanır."
              : "Package names and destination paths are checked against traversal patterns."}
          </li>
          <li>
            {tr
              ? "Gömülü Rust veya crate bağımlılığı içeren paketler açık güven onayı gerektirir."
              : "Packages with embedded Rust or crate dependencies require explicit trust."}
          </li>
          <li>
            {tr
              ? "huma.lock sürüm, kaynak ve metadata ile tüm paket dosyalarının SHA-256 özetini saklar; bu tek başına paketin güvenilir olduğunu kanıtlamaz."
              : "huma.lock stores versions, sources, and a SHA-256 digest of metadata plus every package file; this alone does not prove a package is trustworthy."}
          </li>
          <li>
            {tr
              ? "Tehlikeli kabuk meta karakterleri içeren betikler etkileşimli onay ister; yalnızca güvendiğiniz manifestleri çalıştırın."
              : "Scripts containing dangerous shell metacharacters require interactive confirmation; run only manifests you trust."}
          </li>
          <li>
            {tr
              ? "Uzak URL'den paket kurma ve otomatik paket güncelleme 0.6.0'da desteklenmez."
              : "Installing packages from remote URLs and automatic package updates are unsupported in 0.6.0."}
          </li>
        </ul>
      </section>

      <div className="flex justify-between border-t border-outline-variant/10 pt-8">
        <Link href={getPath("/docs/cli")} className="text-sm font-bold hover:text-primary">
          ← {dict.Sidebar.items.cli}
        </Link>
        <Link href={getPath("/docs/stdlib")} className="text-sm font-bold hover:text-primary">
          {dict.Sidebar.items.stdlib} →
        </Link>
      </div>
    </main>
  );
}
