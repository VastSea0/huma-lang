"use client";

import Link from "next/link";
import { usePathname } from "next/navigation";
import { useState } from "react";
import LanguageSwitcher from "./LanguageSwitcher";
import { useDocs } from "@/context/DocsContext";
import type { Dictionary } from "@/dictionaries/dictionaries";

export default function Navbar({
  dict,
  locale,
}: {
  dict: Dictionary;
  locale: string;
}) {
  const pathname = usePathname();
  const [copied, setCopied] = useState(false);
  const { toggleSidebar, isSidebarOpen } = useDocs();

  // Helper to get localized path
  const getPath = (path: string) => {
    if (path.startsWith("http")) return path;
    return `/${locale}${path}`;
  };

  // Check if current page is docs
  const isDocsPage = pathname.includes("/docs");

  const handleCopy = () => {
    navigator.clipboard.writeText(
      "curl -fsSL https://raw.githubusercontent.com/VastSea0/huma-lang/main/install.sh | sh"
    );
    setCopied(true);
    setTimeout(() => setCopied(false), 2000);
  };

  return (
    <header className="fixed top-0 w-full z-50 bg-[#131313]/80 backdrop-blur-xl shadow-[0_20px_50px_rgba(0,0,0,0.5)]">
      <nav className="flex justify-between items-center h-16 px-8 max-w-[1440px] mx-auto">
        {/* Mobile Menu Button */}
        {isDocsPage && (
          <button
            onClick={toggleSidebar}
            className="lg:hidden text-on-surface-variant p-2 mr-2"
            aria-label="Toggle Sidebar"
          >
            <span className="material-symbols-outlined">
              {isSidebarOpen ? "close" : "menu"}
            </span>
          </button>
        )}
        {/* Logo + Nav Links */}
        <div className="flex items-center gap-8">
          <Link
            href={getPath("/")}
            className="text-xl font-black text-on-surface tracking-tighter hover:text-primary transition-colors"
          >
            Hüma
          </Link>
          <div className="hidden md:flex items-center gap-6 font-body font-medium tracking-tight text-sm">
            <Link
              href={getPath("/docs")}
              className={`transition-colors pb-0.5 ${
                pathname.includes("/docs")
                  ? "text-primary border-b-2 border-primary"
                  : "text-on-surface-variant hover:text-on-surface"
              }`}
            >
              {dict.Nav.docs}
            </Link>
            <Link
              href={getPath("/playground")}
              className={`transition-colors pb-0.5 ${
                pathname.includes("/playground")
                  ? "text-primary border-b-2 border-primary"
                  : "text-on-surface-variant hover:text-on-surface"
              }`}
            >
              {dict.Nav.playground}
            </Link>
            <Link
              href={getPath("/community")}
              className={`transition-colors pb-0.5 ${
                pathname.includes("/community")
                  ? "text-primary border-b-2 border-primary"
                  : "text-on-surface-variant hover:text-on-surface"
              }`}
            >
              {dict.Nav.community}
            </Link>
            <Link
              href={getPath("https://github.com/VastSea0/huma-lang")}
              target="_blank"
              rel="noopener noreferrer"
              className="text-on-surface-variant hover:text-on-surface transition-colors"
            >
              {dict.Nav.github}
            </Link>
          </div>
        </div>

        {/* Right Side */}
        <div className="flex items-center gap-4">
          {isDocsPage && (
            <div className="relative hidden sm:block">
              <span className="material-symbols-outlined absolute left-3 top-1/2 -translate-y-1/2 text-on-surface-variant/50 text-base">
                search
              </span>
              <input
                type="text"
                placeholder={dict.Nav.search_placeholder}
                className="bg-surface-container-highest border border-outline-variant/10 rounded-sm pl-9 pr-4 py-1.5 text-xs w-56 focus:outline-none focus:ring-1 focus:ring-tertiary/40 transition-all placeholder:text-on-surface-variant/40 text-on-surface"
              />
            </div>
          )}
          
          <LanguageSwitcher currentLocale={locale} />

          <button
            onClick={handleCopy}
            className="bg-primary text-on-primary px-5 py-2 font-body text-sm font-semibold rounded-sm active:scale-95 transition-all hover:bg-primary-fixed ml-2"
          >
            {copied ? dict.Nav.copied : dict.Nav.install}
          </button>
        </div>
      </nav>
    </header>
  );
}
