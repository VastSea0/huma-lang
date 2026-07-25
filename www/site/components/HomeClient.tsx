"use client";

import { useState } from "react";
import type { Dictionary } from "@/dictionaries/dictionaries";

const INSTALL_URL =
  "curl -fsSL https://raw.githubusercontent.com/VastSea0/huma-lang/main/install.sh | sh";

export default function HomeClient({ dict }: { dict: Dictionary }) {
  const [ctaCopied, setCtaCopied] = useState(false);

  const copyInstall = () => {
    navigator.clipboard.writeText(INSTALL_URL);
    setCtaCopied(true);
    setTimeout(() => setCtaCopied(false), 2000);
  };

  return (
    <section className="mt-48 px-8">
      <div className="max-w-[1440px] mx-auto bg-surface-container-lowest rounded-xl p-12 md:p-24 relative overflow-hidden flex flex-col md:flex-row items-center justify-between gap-12">
        <div className="absolute top-0 right-0 w-96 h-96 bg-primary/10 blur-[150px] -mr-48 -mt-48" />
        <div className="z-10 max-w-xl">
          <h2 className="text-4xl md:text-5xl font-black tracking-tight mb-6">
            {dict.CTA.headline}
          </h2>
          <p className="text-on-surface-variant text-lg">
            {dict.CTA.description}
          </p>
        </div>
        <div className="z-10 flex flex-col gap-4 w-full md:w-auto">
          <div className="bg-surface-container-highest p-4 rounded-sm font-mono text-sm border border-outline-variant/10 flex items-center justify-between gap-8">
            <span className="text-on-surface-variant text-xs break-all">
              {INSTALL_URL}
            </span>
            <button
              onClick={copyInstall}
              className="material-symbols-outlined text-on-surface-variant/40 hover:text-primary cursor-pointer transition-colors bg-transparent border-none p-0"
              aria-label={dict.CTA.copy_install_command}
            >
              {ctaCopied ? "check" : "content_copy"}
            </button>
          </div>
          <div className="text-[10px] font-mono uppercase tracking-widest text-on-surface-variant/40 text-center md:text-right">
            {dict.CTA.available_platforms}
          </div>
        </div>
      </div>
    </section>
  );
}
