"use client";

import Link from "next/link";
import { usePathname } from "next/navigation";
import { useDocs } from "@/context/DocsContext";
import { useState, useEffect } from "react";

interface NavItem {
  href: string;
  label: string;
  icon: string;
}

interface NavCategory {
  title: string;
  items: NavItem[];
}

interface NavSection {
  title: string;
  items?: NavItem[];
  categories?: NavCategory[];
}

export default function Sidebar({ dict, locale }: { dict: any; locale: string }) {
  const pathname = usePathname();
  const { isSidebarOpen, setSidebarOpen } = useDocs();

  const [mounted, setMounted] = useState(false);

  useEffect(() => {
    setMounted(true);
  }, []);

  // Track which sub-categories are expanded
  const [expanded, setExpanded] = useState<Record<string, boolean>>({
    core_libraries: true,
    tooling: true,
    community: false,
  });

  const toggleCategory = (key: string) => {
    setExpanded((prev) => ({ ...prev, [key]: !prev[key] }));
  };

  const getPath = (path: string) => {
    if (path.startsWith("http")) return path;
    return `/${locale}${path}`;
  };

  const navSections: NavSection[] = [
    {
      title: dict.Sidebar.core,
      items: [
        { href: getPath("/docs"), label: dict.Sidebar.items.getting_started, icon: "rocket_launch" },
        { href: getPath("/docs/introduction"), label: dict.Sidebar.items.introduction, icon: "info" },
      ],
    },
    {
      title: dict.Sidebar.language,
      items: [
        { href: getPath("/docs/syntax"), label: dict.Sidebar.items.syntax, icon: "code" },
        { href: getPath("/docs/grammar"), label: dict.Sidebar.items.grammar, icon: "spellcheck" },
        { href: getPath("/docs/cli"), label: dict.Sidebar.items.cli, icon: "terminal" },
        { href: getPath("/docs/functions-classes"), label: dict.Sidebar.items.functions, icon: "data_object" },
        { href: getPath("/docs/lists-errors"), label: dict.Sidebar.items.lists, icon: "list" },
      ],
    },
    {
      title: dict.Sidebar.ecosystem,
      categories: [
        {
          title: dict.Sidebar.core_libraries,
          items: [
            { href: getPath("/docs/stdlib"), label: dict.Sidebar.items.stdlib, icon: "menu_book" },
            { href: getPath("/docs/stdlib/sqlite"), label: dict.Sidebar.items.sqlite, icon: "database" },
            { href: getPath("/docs/ag_istekleri"), label: dict.Sidebar.items.ag_istekleri, icon: "globe_uk" },
            { href: getPath("/docs/huma_sunucu"), label: dict.Sidebar.items.huma_sunucu, icon: "dns" },
            { href: getPath("/docs/gui"), label: dict.Sidebar.items.gui, icon: "window" },
          ],
        },
        {
          title: dict.Sidebar.tooling,
          items: [
            { href: getPath("/docs/package-manager"), label: dict.Sidebar.items.package_manager, icon: "package_2" },
            { href: getPath("/docs/compiler"), label: dict.Sidebar.items.compiler, icon: "settings_input_component" },
            { href: getPath("/docs/ide"), label: dict.Sidebar.items.ide, icon: "desktop_windows" },
          ],
        },
        {
          title: dict.Sidebar.community,
          items: [
            { href: getPath("/docs/my-first-package"), label: dict.Sidebar.items.my_first_package, icon: "star" },
            { href: getPath("/docs/projects"), label: dict.Sidebar.items.projects, icon: "lightbulb" },
            { href: getPath("/docs/changelog"), label: dict.Sidebar.items.changelog, icon: "history_edu" },
          ],
        },
      ],
    },
  ];

  const renderItem = (item: NavItem) => {
    const isActive = pathname === item.href;
    return (
      <li key={item.href}>
        <Link
          href={item.href}
          onClick={() => setSidebarOpen(false)}
          className={`flex items-center gap-3 px-3 py-2 rounded-sm transition-all hover:translate-x-1 ${
            isActive
              ? "text-primary bg-surface-container-high"
              : "text-on-surface-variant hover:bg-surface-container-low"
          }`}
        >
          <span className="material-symbols-outlined text-sm">{item.icon}</span>
          <span>{item.label}</span>
        </Link>
      </li>
    );
  };

  // Determine category key from title for expand/collapse state
  const getCategoryKey = (title: string): string => {
    if (title === dict.Sidebar.core_libraries) return "core_libraries";
    if (title === dict.Sidebar.tooling) return "tooling";
    if (title === dict.Sidebar.community) return "community";
    return title;
  };

  return (
    <>
      {/* Backdrop for mobile */}
      {isSidebarOpen && (
        <div
          className="fixed inset-0 bg-black/50 z-40 lg:hidden backdrop-blur-sm transition-opacity"
          onClick={() => setSidebarOpen(false)}
        />
      )}

      <aside className={`
        fixed lg:sticky top-16 left-0 z-40
        h-[calc(100vh-4rem)] w-64 flex flex-col
        bg-surface-container-lowest py-8 px-6 gap-2 border-r border-outline-variant/10
        overflow-y-auto shrink-0 transition-transform duration-300
        ${isSidebarOpen ? "translate-x-0" : "-translate-x-full lg:translate-x-0"}
      `}>
        <div className="mb-6">
          <h3 className="text-lg font-bold text-on-surface">{dict.Sidebar.title}</h3>
          <p className="text-[10px] uppercase tracking-[0.2em] text-on-surface-variant/60 font-semibold mt-0.5">
            {dict.Footer.build.split(" ")[1]}
          </p>
        </div>
        <nav className="space-y-8">
          {mounted && navSections.map((section, sectionIdx) => {
            const sectionKey = `section-${sectionIdx}-${section.title?.toLowerCase().replace(/\s+/g, '-')}`;
            return (
              <div key={sectionKey}>
                <h4 className="text-xs font-bold text-on-surface-variant uppercase tracking-widest mb-3">
                  {section.title}
                </h4>

                {/* Flat items (Basics, Language Guide) */}
                {section.items && (
                  <ul className="space-y-1 font-body text-sm font-semibold tracking-wide">
                    {section.items.map((item, itemIdx) => (
                      <li key={`${sectionKey}-item-${itemIdx}`}>
                        <Link
                          href={item.href}
                          onClick={() => setSidebarOpen(false)}
                          className={`flex items-center gap-3 px-3 py-2 rounded-sm transition-all hover:translate-x-1 ${
                            pathname === item.href
                              ? "text-primary bg-surface-container-high"
                              : "text-on-surface-variant hover:bg-surface-container-low"
                          }`}
                        >
                          <span className="material-symbols-outlined text-sm">{item.icon}</span>
                          <span>{item.label}</span>
                        </Link>
                      </li>
                    ))}
                  </ul>
                )}

                {/* Collapsible categories (Ecosystem) */}
                {section.categories && (
                  <div className="space-y-4">
                    {section.categories.map((cat, catIdx) => {
                      const catSlug = getCategoryKey(cat.title);
                      const fullCatKey = `sidebar-section-${sectionIdx}-cat-${catIdx}`;
                      const isOpen = expanded[catSlug] ?? false;
                      const hasActiveChild = cat.items.some((item) => pathname === item.href);

                      return (
                        <div key={fullCatKey}>
                          <button
                            onClick={() => toggleCategory(catSlug)}
                            className={`w-full flex items-center justify-between px-3 py-1.5 rounded-sm text-[11px] font-bold uppercase tracking-widest transition-colors ${
                              hasActiveChild
                                ? "text-primary"
                                : "text-on-surface-variant/70 hover:text-on-surface-variant"
                            }`}
                          >
                            <span>{cat.title}</span>
                            <span
                              className={`material-symbols-outlined text-sm transition-transform duration-200 ${
                                isOpen ? "rotate-0" : "-rotate-90"
                              }`}
                            >
                              expand_more
                            </span>
                          </button>
                          <div
                            className={`overflow-hidden transition-all duration-200 ease-in-out ${
                              isOpen ? "max-h-96 opacity-100 mt-1" : "max-h-0 opacity-0"
                            }`}
                          >
                            <ul className="space-y-1 font-body text-sm font-semibold tracking-wide pl-1">
                              {cat.items.map((item, subItemIdx) => (
                                <li key={`${fullCatKey}-sub-${subItemIdx}`}>
                                  <Link
                                    href={item.href}
                                    onClick={() => setSidebarOpen(false)}
                                    className={`flex items-center gap-3 px-3 py-2 rounded-sm transition-all hover:translate-x-1 ${
                                      pathname === item.href
                                        ? "text-primary bg-surface-container-high"
                                        : "text-on-surface-variant hover:bg-surface-container-low"
                                    }`}
                                  >
                                    <span className="material-symbols-outlined text-sm">{item.icon}</span>
                                    <span>{item.label}</span>
                                  </Link>
                                </li>
                              ))}
                            </ul>
                          </div>
                        </div>
                      );
                    })}
                  </div>
                )}
              </div>
            );
          })}
        </nav>
      </aside>
    </>
  );
}
