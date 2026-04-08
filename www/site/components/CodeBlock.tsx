"use client";

import { useState } from "react";

interface CodeBlockProps {
  code: string;
  language?: string;
  filename?: string;
  showLineNumbers?: boolean;
  variant?: "default" | "terminal" | "output";
}

// Simple Hüma syntax highlighter
function tokenize(line: string): string {
  // Keywords
  const keywords = [
    "olsun",
    "ise",
    "yoksa",
    "olduğu sürece",
    "dene",
    "hata",
    "yükle",
    "döndür",
    "alsın",
    "bu",
    "fonksiyon",
    "sınıf",
  ];
  const builtinFunctions = [
    "yazdır",
    "ekle",
    "çıkar",
    "yükle",
    "boyut",
    "beklet",
  ];

  let result = line;

  // Escape HTML first
  result = result
    .replace(/&/g, "&amp;")
    .replace(/</g, "&lt;")
    .replace(/>/g, "&gt;");

  // Comments (// ...)
  result = result.replace(
    /(\/\/[^\n]*)/g,
    '<span class="token-comment">$1</span>'
  );

  // Don't further tokenize if the whole line is a comment
  if (result.includes('<span class="token-comment">')) {
    return result;
  }

  // Strings
  result = result.replace(
    /("(?:[^"\\]|\\.)*")/g,
    '<span class="token-string">$1</span>'
  );

  // Numbers
  result = result.replace(
    /\b(\d+(?:\.\d+)?)\b/g,
    '<span class="token-number">$1</span>'
  );

  // Keywords
  keywords.forEach((kw) => {
    // Escape for regex if needed, though these are all safe
    const escapedKw = kw.replace(/[.*+?^${}()|[\]\\]/g, "\\$&");
    result = result.replace(
      new RegExp(`\\b(${escapedKw})\\b`, "g"),
      '<span class="token-keyword">$1</span>'
    );
  });

  // Built-in functions
  builtinFunctions.forEach((fn) => {
    result = result.replace(
      new RegExp(`\\b(${fn})\\b`, "g"),
      '<span class="token-function">$1</span>'
    );
  });

  return result;
}

export default function CodeBlock({
  code,
  language = "huma",
  filename,
  showLineNumbers = true,
  variant = "default",
}: CodeBlockProps) {
  const [copied, setCopied] = useState(false);

  const lines = code.trim().split("\n");

  const handleCopy = () => {
    navigator.clipboard.writeText(code.trim());
    setCopied(true);
    setTimeout(() => setCopied(false), 2000);
  };

  if (variant === "terminal") {
    return (
      <div className="bg-surface-container-lowest rounded-lg overflow-hidden border border-outline-variant/10 group relative mb-6">
        <div className="flex items-center justify-between px-4 py-2 bg-surface-container-low border-b border-outline-variant/5">
          <span className="text-[10px] font-mono text-on-surface-variant/50 uppercase tracking-widest">
            Terminal — Bash
          </span>
          <button
            onClick={handleCopy}
            className="text-on-surface-variant/40 hover:text-primary transition-colors"
            aria-label="Copy code"
          >
            <span className="material-symbols-outlined text-lg">
              {copied ? "check" : "content_copy"}
            </span>
          </button>
        </div>
        <pre className="p-6 overflow-x-auto">
          <code className="font-mono text-sm leading-relaxed text-tertiary">
            {code.trim().split("\n").map((line, i) => {
              // Simple bash highlighter
              const highlighted = line
                .replace(/^(\$?\s*)(huma|çalıştır|derle|paket|paket)\b/g, '$1<span class="text-primary font-bold">$2</span>')
                .replace(/(--?\w+)/g, '<span class="text-on-surface-variant/60">$1</span>');
              return (
                <div key={i} dangerouslySetInnerHTML={{ __html: highlighted }} />
              );
            })}
          </code>
        </pre>
      </div>
    );
  }

  return (
    <div className="bg-surface-container-lowest rounded-lg overflow-hidden border border-outline-variant/10 relative mb-6">
      {/* Header */}
      <div className="flex items-center justify-between px-4 py-2 bg-surface-container-low border-b border-outline-variant/5">
        <div className="flex items-center gap-4">
          {filename && (
            <span className="text-[10px] font-mono text-on-surface-variant/50 uppercase tracking-widest">
              {filename}
            </span>
          )}
          {!filename && (
            <span className="text-[10px] font-mono text-on-surface-variant/50 uppercase tracking-widest">
              {language === "huma" ? "Hüma" : language}
            </span>
          )}
          <span className="w-2 h-2 rounded-full bg-primary animate-pulse" />
        </div>
        <button
          onClick={handleCopy}
          className="text-on-surface-variant/40 hover:text-primary transition-colors"
          aria-label="Copy code"
        >
          <span className="material-symbols-outlined text-lg">
            {copied ? "check" : "content_copy"}
          </span>
        </button>
      </div>

      {/* Code */}
      <pre className="p-6 overflow-x-auto">
        <code className="font-mono text-sm leading-relaxed">
          {showLineNumbers ? (
            <div className="flex gap-4">
              {/* Line numbers */}
              <div className="text-on-surface-variant/30 text-right select-none w-6 shrink-0">
                {lines.map((_, i) => (
                  <div key={i}>{i + 1}</div>
                ))}
              </div>
              {/* Code content */}
              <div className="flex-1 min-w-0">
                {lines.map((line, i) => (
                  <div
                    key={i}
                    dangerouslySetInnerHTML={{ __html: tokenize(line) }}
                  />
                ))}
              </div>
            </div>
          ) : (
            <div>
              {lines.map((line, i) => (
                <div
                  key={i}
                  dangerouslySetInnerHTML={{ __html: tokenize(line) }}
                />
              ))}
            </div>
          )}
        </code>
      </pre>
    </div>
  );
}
