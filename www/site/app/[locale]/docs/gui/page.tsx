import Link from "next/link";
import { getDictionary } from "@/dictionaries/dictionaries";
import CodeBlock from "@/components/CodeBlock";
import type { Metadata } from "next";

export const metadata: Metadata = {
  title: "Graphical User Interface (GUI)",
  description: "egui-based native interface creation tools for Hüma.",
};

const guiCode = `yükle "gui";

sayac = 0 olsun

cizim_fks fonksiyon olsun {
    yazı_ekle("Merhaba Hüma GUI!", "başlık")
    boşluk_bırak(10.0)
    
    yazı_ekle("Sayaç değeri: " + sayac)
    
    buton_ekle("Artır") ise {
        sayac = sayac + 1
    }
}

pencere_oluştur("Hüma Örnek", 400.0, 300.0, cizim_fks)`;

export default async function GuiPage({
  params,
}: {
  params: Promise<{ locale: string }>;
}) {
  const { locale } = await params;
  const dict = await getDictionary(locale as "en" | "tr");
  const g = dict.Docs.gui;

  const getPath = (path: string) => `/${locale}${path}`;

  const functions = [
    {
      name: "pencere_oluştur(başlık, w, h, çizim_fks)",
      desc: locale === "tr" ? "Yeni bir masaüstü penceresi oluşturur ve çizim döngüsünü başlatır." : "Creates a new desktop window and starts the drawing loop.",
    },
    {
      name: "buton_ekle(metin, [r, g, b, w, h])",
      desc: locale === "tr" ? "Bir buton ekler. Tıklandığında doğru (1) döner." : "Adds a button. Returns true (1) when clicked.",
    },
    {
      name: "yazı_ekle(metin, [stil | r, g, b])",
      desc: locale === "tr" ? "Metin etiketi ekler. Stil olarak 'başlık', 'kalın' veya 'eğik' alabilir." : "Adds a text label. Can take 'başlık', 'kalın', or 'eğik' as style.",
    },
    {
      name: "metin_kutusu_ekle(değişken, [w])",
      desc: locale === "tr" ? "Tek satırlık metin giriş alanı ekler. Değişen metni döndürür." : "Adds a single-line text entry field. Returns changed text.",
    },
    {
      name: "kaydırıcı_ekle(değer, min, max)",
      desc: locale === "tr" ? "Sayısal bir kaydırıcı (Slider) ekler." : "Adds a numerical slider.",
    },
    {
      name: "yan_yana_diz(fks)",
      desc: locale === "tr" ? "İçindeki bileşenleri yatay düzende hizalar." : "Aligns components horizontally.",
    },
  ];

  return (
    <>
      <main className="flex-1 px-8 md:px-16 py-12 max-w-4xl">
        <nav className="flex gap-2 text-[10px] uppercase tracking-widest text-on-surface-variant/60 mb-4">
          <Link href={getPath("/docs")} className="hover:text-primary transition-colors">
            {dict.Nav.docs}
          </Link>
          <span>/</span>
          <span className="text-on-surface-variant">{dict.Sidebar.ecosystem}</span>
          <span>/</span>
          <span className="text-primary">{g.title}</span>
        </nav>

        <h1 className="text-5xl font-extrabold text-on-surface tracking-tighter mb-6">
          {g.hero_title}
        </h1>
        <p className="text-lg text-on-surface-variant leading-relaxed mb-12">
          {g.hero_desc}
        </p>

        <div className="bg-primary/5 border border-primary/10 p-6 rounded-xl mb-12">
           <div className="font-mono text-sm text-primary">yükle &quot;gui&quot;;</div>
        </div>

        {/* Quick Start */}
        <section className="mb-16">
          <h2 className="text-2xl font-bold text-on-surface mb-6 flex items-center gap-3">
             <span className="w-8 h-8 rounded-full bg-surface-container-high flex items-center justify-center text-sm font-mono text-primary">
              01
            </span>
            {g.quick_start.title}
          </h2>
          <p className="text-on-surface-variant leading-relaxed mb-8">
            {g.quick_start.desc}
          </p>
          <CodeBlock code={guiCode} filename="arayuz.hb" />
        </section>

        {/* Documentation */}
        <section className="mb-16">
          <h2 className="text-2xl font-bold text-on-surface mb-6 flex items-center gap-3">
             <span className="w-8 h-8 rounded-full bg-surface-container-high flex items-center justify-center text-sm font-mono text-primary">
              02
            </span>
            {locale === "tr" ? "Fonksiyon Referansı" : "Function Reference"}
          </h2>
          <div className="space-y-4">
            {functions.map((fn) => (
              <div key={fn.name} className="bg-surface-container-low rounded-lg p-5 border border-outline-variant/10">
                <code className="text-primary font-mono text-sm font-bold block mb-2">{fn.name}</code>
                <p className="text-sm text-on-surface-variant">
                  {fn.desc}
                </p>
              </div>
            ))}
          </div>
        </section>

        {/* Info Box */}
        <div className="bg-secondary/5 border-l-4 border-secondary p-8 rounded-r-2xl mt-12">
          <div className="flex items-center gap-3 mb-4 text-secondary">
            <span className="material-symbols-outlined text-2xl">rocket_launch</span>
            <h3 className="text-lg font-bold">
               {locale === "tr" ? "Yerel Performans" : "Native Performance"}
            </h3>
          </div>
          <p className="text-on-surface-variant text-sm leading-relaxed">
            {locale === "tr"
              ? "Hüma GUI kütüphanesi, Rust tarafından sağlanan donanım hızlandırmalı grafik API'lerini kullanır. Bu sayede karmaşık arayüzler bile sıfır gecikme ile çalışır."
              : "The Hüma GUI library utilizes hardware-accelerated graphics APIs provided by Rust. This ensures that even complex interfaces run with zero latency."}
          </p>
        </div>

        {/* Navigation */}
        <div className="flex justify-between mt-24 pt-8 border-t border-outline-variant/10">
          <Link
            href={getPath("/docs/huma_sunucu")}
            className="flex items-center gap-2 text-sm text-on-surface-variant hover:text-primary transition-colors font-bold uppercase tracking-widest text-[10px]"
          >
            <span className="material-symbols-outlined text-base">arrow_back</span>
            {dict.Docs.network.server.title}
          </Link>
          <Link
            href={getPath("/docs/package-manager")}
            className="flex items-center gap-2 text-sm text-on-surface-variant hover:text-primary transition-colors font-bold uppercase tracking-widest text-[10px]"
          >
            {dict.Docs.package_manager.title}
            <span className="material-symbols-outlined text-base">arrow_forward</span>
          </Link>
        </div>
      </main>
    </>
  );
}
