import Navbar from "@/components/Navbar";
import Sidebar from "@/components/Sidebar";
import Link from "next/link";
import { getDictionary } from "@/dictionaries/dictionaries";

export default async function DocsLayout({
  children,
  params,
}: {
  children: React.ReactNode;
  params: Promise<{ locale: string }>;
}) {
  const { locale } = await params;
  const dict = await getDictionary(locale as "en" | "tr");

  return (
    <>
      <Navbar dict={dict} locale={locale} />
      <div className="max-w-[1440px] mx-auto flex pt-16 min-h-screen">
        <Sidebar dict={dict} locale={locale} />
        <div className="flex-1 flex min-w-0 pb-24 lg:pb-0">
          {children}
        </div>
      </div>
      {/* Footer */}
      <footer className="bg-[#0E0E0E] w-full py-12 border-t border-[#5A413A]/20">
        <div className="flex flex-col md:flex-row justify-between items-center px-12 gap-6 max-w-[1440px] mx-auto">
          <p className="font-body text-xs uppercase tracking-[0.1em] text-on-surface-variant/60">
            {dict.Footer.copy}
          </p>
          <div className="flex gap-8 font-body text-xs uppercase tracking-[0.1em]">
            <Link href="https://github.com/VastSea0/huma-lang" target="_blank" rel="noopener noreferrer"
              className="text-on-surface-variant/60 hover:text-primary transition-colors">{dict.Footer.github}</Link>
            <Link href="https://github.com/VastSea0/huma-lang/discussions" target="_blank" rel="noopener noreferrer"
              className="text-on-surface-variant/60 hover:text-primary transition-colors">{dict.Footer.discord}</Link>
            <Link href="https://github.com/VastSea0/huma-lang/releases" target="_blank" rel="noopener noreferrer"
              className="text-on-surface-variant/60 hover:text-primary transition-colors">{dict.Footer.releases}</Link>
            <Link href="https://github.com/VastSea0/huma-lang/issues" target="_blank" rel="noopener noreferrer"
              className="text-on-surface-variant/60 hover:text-primary transition-colors">{dict.Footer.issues}</Link>
            <Link href="https://github.com/VastSea0/huma-lang/blob/main/LICENSE" target="_blank" rel="noopener noreferrer"
              className="text-on-surface-variant/60 hover:text-primary transition-colors">{dict.Footer.license_link}</Link>
          </div>
        </div>
      </footer>
    </>
  );
}
