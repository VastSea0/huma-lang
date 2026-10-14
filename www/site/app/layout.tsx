import type { Metadata } from 'next';
import './globals.css';

export const metadata: Metadata = {
  title: 'Hüma — Türkçe konuşan programlama dili',
  description:
    'Hüma, Türkçe dilbilgisini koruyan genel amaçlı modern bir programlama dili. Hızlı, güvenli, versiyonlanabilir.',
  openGraph: {
    title: 'Hüma Programlama Dili',
    description: 'Türkçe sözdizimli, güvenlik odaklı, genel amaçlı programlama dili.',
    type: 'website',
  },
};

export default function RootLayout({
  children,
}: {
  children: React.ReactNode;
}) {
  return (
    <html lang="tr">
      <head>
        <link rel="preconnect" href="https://fonts.googleapis.com" />
        <link rel="preconnect" href="https://fonts.gstatic.com" crossOrigin="anonymous" />
        <link
          href="https://fonts.googleapis.com/css2?family=Inter:wght@400;500;600&family=JetBrains+Mono:wght@400;500&display=swap"
          rel="stylesheet"
        />
      </head>
      <body>{children}</body>
    </html>
  );
}
