import Link from "next/link";
import { getDictionary } from "@/dictionaries/dictionaries";
import CodeBlock from "@/components/CodeBlock";
import type { Metadata } from "next";

export const metadata: Metadata = {
  title: "Hüma Sunucu (HTTP Server)",
  description:
    "Build high-performance web servers with Hüma's modern, minimalist backend framework.",
};

const serverCode = `yükle "huma_sunucu";

s = Sunucu() olsun
s.kur(3000)

s.getir("/", fonksiyon olsun istek, yanit alsın {
    yanit.html("<h1>Merhaba Hüma!</h1>");
})

s.baslat()`;

const dynamicCode = `s.getir("/kullanici/:id", fonksiyon olsun istek, yanit alsın {
    kid = değer_al(istek.parametreler, "id")
    yanit.metin("Kullanıcı ID: " + kid);
})`;

export default async function HumaSunucuPage({
  params,
}: {
  params: Promise<{ locale: string }>;
}) {
  const { locale } = await params;
  const dict = await getDictionary(locale as "en" | "tr");
  const n = dict.Docs.network;

  const getPath = (path: string) => `/${locale}${path}`;

  const serverMethods = [
    {
      fn: "Sunucu()",
      desc: locale === "tr" ? "Yeni sunucu örneği oluşturur." : "Creates a new server instance.",
    },
    {
      fn: "kur(port)",
      desc: locale === "tr" ? "Sunucu portunu ayarlar." : "Sets the server port.",
    },
    {
      fn: "getir(yol, fonk)",
      desc: locale === "tr" ? "GET rota kaydeder." : "Registers a GET route handler.",
    },
    {
      fn: "gönder(yol, fonk)",
      desc: locale === "tr" ? "POST rota kaydeder." : "Registers a POST route handler.",
    },
    {
      fn: "cors_ayarla(kaynak)",
      desc: locale === "tr" ? "CORS başlıklarını yapılandırır." : "Configures CORS headers.",
    },
    {
      fn: "statik(dizin)",
      desc: locale === "tr" ? "Statik dosya dizinini belirler." : "Sets the static file directory.",
    },
    {
      fn: "baslat()",
      desc: locale === "tr" ? "Sunucuyu başlatır." : "Starts the server.",
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
          <span className="text-primary">{n.server.title}</span>
        </nav>

        {/* Title & Description */}
        <h1 className="text-5xl font-extrabold text-on-surface tracking-tighter mb-6">
          {n.server.title}
        </h1>
        <p className="text-lg text-on-surface-variant leading-relaxed mb-8">
          {n.server.desc}
        </p>

        {/* Quick Installation */}
        <section className="mb-16" id="installation">
          <h2 className="text-2xl font-bold text-on-surface mb-4">
            {locale === "tr" ? "Kurulum" : "Installation"}
          </h2>
          <div className="bg-primary/5 border border-primary/10 p-6 rounded-xl flex items-center justify-between">
            <code className="text-on-surface font-mono text-sm font-bold bg-surface-container-high px-3 py-1.5 rounded border border-outline-variant/10">
              huma paket kur huma_sunucu
            </code>
            <span className="hidden sm:block text-[10px] font-bold bg-primary text-on-primary px-3 py-1 rounded-full uppercase tracking-widest">
              v1.4.0 STABLE
            </span>
          </div>
        </section>

        {/* Quick Start */}
        <section className="mb-16" id="quick-start">
          <h2 className="text-2xl font-bold text-on-surface mb-4">
            {locale === "tr" ? "Hızlı Başlangıç" : "Quick Start"}
          </h2>
          <p className="text-on-surface-variant leading-relaxed mb-6">
            {locale === "tr"
              ? "Sunucunuzu 'Sunucu()' nesnesi ile örneklendirin. 'kur' metodu ile portu belirleyin ve 'getir' metodu ile rotalarınızı tanımlayın."
              : "Instantiate your server with the 'Sunucu()' object. Set the port with 'kur' and define your routes with 'getir'."}
          </p>
          <CodeBlock code={serverCode} filename="sunucu.hb" />
        </section>

        {/* Dynamic Routing */}
        <section className="mb-16" id="dynamic-routing">
          <h2 className="text-2xl font-bold text-on-surface mb-4">
            {locale === "tr" ? "Dinamik Rotalama" : "Dynamic Routing"}
          </h2>
          <p className="text-on-surface-variant leading-relaxed mb-6">
            {locale === "tr"
              ? "URL içerisinden parametre almak için ':' ön ekini kullanın. 'istek.parametreler' nesnesi üzerinden bu parametrelere erişebilirsiniz."
              : "Use the ':' prefix to capture parameters from the URL. You can access these via the 'istek.parametreler' object."}
          </p>
          <CodeBlock code={dynamicCode} filename="rota_param.hb" />
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
                    {locale === "tr" ? "Metot" : "Method"}
                  </th>
                  <th className="text-left py-3 px-5 text-on-surface-variant/60 font-bold text-[10px] uppercase tracking-widest">
                    {locale === "tr" ? "Açıklama" : "Description"}
                  </th>
                </tr>
              </thead>
              <tbody className="divide-y divide-outline-variant/10">
                {serverMethods.map((item) => (
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

        {/* Navigation */}
        <div className="flex justify-between mt-24 pt-8 border-t border-outline-variant/10">
          <Link
            href={getPath("/docs/ag_istekleri")}
            className="flex items-center gap-2 text-on-surface-variant hover:text-primary transition-colors font-bold uppercase tracking-widest text-[10px]"
          >
            <span className="material-symbols-outlined text-base">arrow_back</span>
            {n.requests.title}
          </Link>
          <Link
            href={getPath("/docs/gui")}
            className="flex items-center gap-2 text-on-surface-variant hover:text-primary transition-colors font-bold uppercase tracking-widest text-[10px]"
          >
            {dict.Docs.gui.title}
            <span className="material-symbols-outlined text-base">arrow_forward</span>
          </Link>
        </div>
      </main>
    </>
  );
}
