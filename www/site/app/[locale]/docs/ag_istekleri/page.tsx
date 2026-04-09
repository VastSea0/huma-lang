import Link from "next/link";
import { getDictionary } from "@/dictionaries/dictionaries";
import CodeBlock from "@/components/CodeBlock";
import type { Metadata } from "next";

export const metadata: Metadata = {
  title: "Network Requests (HTTP)",
  description:
    "Consume REST APIs and handle HTTP communication in Hüma using the ag_istekleri library.",
};

const requestCode = `yükle "ag_istekleri";

# 1. Başlık Nesnesi Hazırla (v1.1.0+)
h = metinden_nesneye("{}") olsun
değer_ata(h, "Authorization", "Bearer TOKEN_PRO")
değer_ata(h, "Content-Type", "application/json")

# 2. İsteği Gönder (URL, Başlık)
url = "https://api.github.com/user" olsun
cevap = getir(url, h) olsun

cevap.durum == 200 ise {
    cevap.içerik'i yazdır;
} yoksa {
    "İstek Hatası: " + cevap.hata_mesajı'nı yazdır;
}`;

export default async function AgIstekleriPage({
  params,
}: {
  params: Promise<{ locale: string }>;
}) {
  const { locale } = await params;
  const dict = await getDictionary(locale as "en" | "tr");
  const n = dict.Docs.network;

  const getPath = (path: string) => `/${locale}${path}`;

  const apiFunctions = [
    {
      fn: "getir(url, [headers])",
      desc: locale === "tr" ? "URL'ye GET isteği gönderir." : "Sends a GET request to the URL.",
    },
    {
      fn: "gönder(url, veri, [headers])",
      desc: locale === "tr" ? "URL'ye POST isteği gönderir." : "Sends a POST request to the URL.",
    },
    {
      fn: "güncelle(url, veri, [headers])",
      desc: locale === "tr" ? "URL'ye PUT isteği gönderir." : "Sends a PUT request to the URL.",
    },
    {
      fn: "sil(url, [headers])",
      desc: locale === "tr" ? "URL'ye DELETE isteği gönderir." : "Sends a DELETE request to the URL.",
    },
  ];

  return (
    <>
      <main className="flex-1 px-8 md:px-16 py-12 max-w-4xl">
        {/* Breadcrumb */}
        <nav className="flex gap-2 text-[10px] uppercase tracking-widest text-on-surface-variant/60 mb-4">
          <Link href={getPath("/docs")} className="hover:text-primary transition-colors">
            {dict.Nav.docs}
          </Link>
          <span>/</span>
          <span className="text-on-surface-variant">{dict.Sidebar.core_libraries}</span>
          <span>/</span>
          <span className="text-primary">{n.requests.title}</span>
        </nav>

        {/* Title & Description */}
        <h1 className="text-5xl font-extrabold text-on-surface tracking-tighter mb-6">
          {n.requests.title}
        </h1>
        <p className="text-lg text-on-surface-variant leading-relaxed mb-8">
          {n.requests.desc}
        </p>

        {/* Quick Installation */}
        <section className="mb-16" id="installation">
          <h2 className="text-2xl font-bold text-on-surface mb-4">
            {locale === "tr" ? "Kurulum" : "Installation"}
          </h2>
          <div className="bg-primary/5 border border-primary/10 p-6 rounded-xl flex items-center justify-between">
            <code className="text-on-surface font-mono text-sm font-bold bg-surface-container-high px-3 py-1.5 rounded border border-outline-variant/10">
              huma paket kur ag_istekleri
            </code>
            <span className="hidden sm:block text-[10px] font-bold bg-primary text-on-primary px-3 py-1 rounded-full uppercase tracking-widest">
              v1.1.0 STABLE
            </span>
          </div>
        </section>

        {/* Basic Usage */}
        <section className="mb-16" id="basic-usage">
          <h2 className="text-2xl font-bold text-on-surface mb-4">
            {locale === "tr" ? "Temel Kullanım" : "Basic Usage"}
          </h2>
          <p className="text-on-surface-variant leading-relaxed mb-6">
            {locale === "tr"
              ? "Kütüphaneyi yükledikten sonra 'getir' (GET) ve 'gönder' (POST) gibi fonksiyonlar üzerinden API istekleri gerçekleştirebilirsiniz."
              : "After importing the library, you can perform API requests using functions like 'getir' (GET) and 'gönder' (POST)."}
          </p>
          <CodeBlock code={requestCode} filename="api_istek.hb" />
        </section>

        {/* API Reference */}
        <section className="mb-16" id="api-reference">
          <h2 className="text-2xl font-bold text-on-surface mb-6">
            {locale === "tr" ? "API Referansı" : "API Reference"}
          </h2>
          <div className="overflow-x-auto bg-surface-container-low rounded-lg border border-outline-variant/10">
            <table className="w-full text-sm border-collapse">
              <thead>
                <tr className="border-b border-outline-variant/20">
                  <th className="text-left py-3 px-5 text-on-surface-variant/60 font-bold text-[10px] uppercase tracking-widest">
                    {locale === "tr" ? "Fonksiyon" : "Function"}
                  </th>
                  <th className="text-left py-3 px-5 text-on-surface-variant/60 font-bold text-[10px] uppercase tracking-widest">
                    {locale === "tr" ? "Açıklama" : "Description"}
                  </th>
                </tr>
              </thead>
              <tbody className="divide-y divide-outline-variant/10">
                {apiFunctions.map((item) => (
                  <tr key={item.fn} className="hover:bg-surface-container-lowest transition-colors">
                    <td className="py-4 px-5">
                      <code className="text-primary font-mono text-xs font-bold">{item.fn}</code>
                    </td>
                    <td className="py-4 px-5 text-on-surface-variant text-sm leading-relaxed">
                      {item.desc}
                    </td>
                  </tr>
                ))}
              </tbody>
            </table>
          </div>
        </section>

        {/* Info Box */}
        <div className="bg-tertiary/5 border-l-4 border-tertiary p-8 rounded-r-2xl">
          <div className="flex items-center gap-3 mb-4 text-tertiary">
            <span className="material-symbols-outlined text-2xl">package_2</span>
            <h3 className="text-lg font-bold">
               {locale === "tr" ? "Ekosistem Modülü" : "Ecosystem Module"}
            </h3>
          </div>
          <p className="text-on-surface-variant text-sm leading-relaxed">
            {locale === "tr"
              ? "ag_istekleri, Hüma'nın standart kütüphanesinden bağımsız olarak gelişen bir topluluk paketidir. Bu yapı, HTTP protokollerindeki güncellemelerin derleyici sürümünden bağımsız olarak yayınlanmasını sağlar."
              : "ag_istekleri is a community package that evolves independently of Hüma's built-in library. This structure ensures that HTTP protocol updates can be released regardless of the compiler version."}
          </p>
        </div>

        {/* Navigation */}
        <div className="flex justify-between mt-24 pt-8 border-t border-outline-variant/10">
          <Link
            href={getPath("/docs/stdlib")}
            className="flex items-center gap-2 text-on-surface-variant hover:text-primary transition-colors font-bold uppercase tracking-widest text-[10px]"
          >
            <span className="material-symbols-outlined text-base">arrow_back</span>
            {dict.Docs.stdlib.title}
          </Link>
          <Link
            href={getPath("/docs/huma_sunucu")}
            className="flex items-center gap-2 text-on-surface-variant hover:text-primary transition-colors font-bold uppercase tracking-widest text-[10px]"
          >
            {n.server.title}
            <span className="material-symbols-outlined text-base">arrow_forward</span>
          </Link>
        </div>
      </main>
    </>
  );
}
