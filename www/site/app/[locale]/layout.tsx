import type { Metadata } from "next";
import { Inter, JetBrains_Mono } from "next/font/google";
import "../globals.css";
import { DocsProvider } from "@/context/DocsContext";

const inter = Inter({
  subsets: ["latin"],
  variable: "--font-inter",
  display: "swap",
  weight: ["300", "400", "500", "600", "700", "800", "900"],
});

const jetbrainsMono = JetBrains_Mono({
  subsets: ["latin"],
  variable: "--font-jetbrains",
  display: "swap",
  weight: ["400", "500", "600", "700"],
});

export const metadata: Metadata = {
  title: {
    template: "%s | Hüma",
    default: "Hüma — Code as You Think",
  },
  description:
    "Hüma is a high-performance programming language featuring native Turkish syntax. Built with a Rust-backed core, it combines the expressiveness of natural language with the raw speed of a systems compiler.",
  keywords: ["hüma", "programming language", "Turkish", "compiler", "Rust"],
  openGraph: {
    title: "Hüma — Code as You Think",
    description:
      "High-performance programming language with native Turkish syntax.",
    type: "website",
  },
};

export async function generateStaticParams() {
  return [{ locale: "en" }, { locale: "tr" }];
}

export default async function RootLayout({
  children,
  params,
}: {
  children: React.ReactNode;
  params: Promise<{ locale: string }>;
}) {
  const { locale } = await params;

  return (
    <html
      lang={locale}
      className={`dark ${inter.variable} ${jetbrainsMono.variable}`}
      data-scroll-behavior="smooth"
    >
      <head>
        <link
          rel="stylesheet"
          href="https://fonts.googleapis.com/css2?family=Material+Symbols+Outlined:wght,FILL@100..700,0..1&display=swap"
        />
        <style>{`
          .material-symbols-outlined {
            font-variation-settings: 'FILL' 0, 'wght' 400, 'GRAD' 0, 'opsz' 24;
          }
        `}</style>
      </head>
      <body className="bg-surface text-on-surface font-body antialiased">
        <DocsProvider>
          {children}
        </DocsProvider>
      </body>
    </html>
  );
}
