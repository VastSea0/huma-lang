"use client";

import Link from "next/link";
import { usePathname } from "next/navigation";
import { useDocs } from "@/context/DocsContext";

export default function Sidebar({ dict, locale }: { dict: any; locale: string }) {
  const pathname = usePathname();
  const { isSidebarOpen, setSidebarOpen } = useDocs();

  const getPath = (path: string) => {
    if (path.startsWith("http")) return path;
    return `/${locale}${path}`;
  };

  const navSections = [
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
        { href: getPath("/docs/cli"), label: dict.Sidebar.items.cli, icon: "terminal" },
        {
          href: getPath("/docs/functions-classes"),
          label: dict.Sidebar.items.functions,
          icon: "data_object",
        },
        {
          href: getPath("/docs/lists-errors"),
          label: dict.Sidebar.items.lists,
          icon: "list",
        },
      ],
    },
    {
      title: dict.Sidebar.ecosystem,
      items: [
        { href: getPath("/docs/stdlib"), label: dict.Sidebar.items.stdlib, icon: "menu_book" },
        {
          href: getPath("/docs/compiler"),
          label: dict.Sidebar.items.compiler,
          icon: "settings_input_component",
        },
        {
          href: getPath("/docs/ide"),
          label: dict.Sidebar.items.ide,
          icon: "desktop_windows",
        },
        {
          href: getPath("/docs/gui"),
          label: dict.Sidebar.items.gui,
          icon: "window",
        },
        {
          href: getPath("/docs/ag_istekleri"),
          label: dict.Sidebar.items.ag_istekleri,
          icon: "globe_uk",
        },
        {
          href: getPath("/docs/huma_sunucu"),
          label: dict.Sidebar.items.huma_sunucu,
          icon: "dns",
        },
        {
          href: getPath("/docs/package-manager"),
          label: dict.Sidebar.items.package_manager,
          icon: "package_2",
        },
        {
          href: getPath("/docs/stdlib/sqlite"),
          label: dict.Sidebar.items.sqlite,
          icon: "database",
        },
        {
          href: getPath("/docs/my-first-package"),
          label: dict.Sidebar.items.my_first_package,
          icon: "star",
        },
        {
          href: getPath("/docs/changelog"),
          label: dict.Sidebar.items.changelog,
          icon: "history_edu",
        },
      ],
    },
  ];

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
          {navSections.map((section) => (
            <div key={section.title}>
              <h4 className="text-xs font-bold text-on-surface-variant uppercase tracking-widest mb-3">
                {section.title}
              </h4>
              <ul className="space-y-1 font-body text-sm font-semibold tracking-wide">
                {section.items.map((item) => {
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
                        <span className="material-symbols-outlined text-sm">
                          {item.icon}
                        </span>
                        <span>{item.label}</span>
                      </Link>
                    </li>
                  );
                })}
              </ul>
            </div>
          ))}
        </nav>
      </aside>
    </>
  );
}
