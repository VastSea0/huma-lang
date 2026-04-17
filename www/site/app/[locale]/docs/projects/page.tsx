import Link from "next/link";
import CodeBlock from "@/components/CodeBlock";
import type { Metadata } from "next";
import { getDictionary } from "@/dictionaries/dictionaries";

export async function generateMetadata({
  params,
}: {
  params: Promise<{ locale: string }>;
}): Promise<Metadata> {
  const { locale } = await params;
  const dict = await getDictionary(locale as "en" | "tr");
  return {
    title: dict.Docs?.projects?.title || "Projects",
    description: dict.Docs?.projects?.description || "Project examples description",
  };
}

const todoCode = `// 1. Görev Yöneticisi (CLI To-Do List)
"dosya.hb"'yi yükle

gorevler_dosyasi = "gorevler.txt" olsun

ekle fonksiyon olsun gorev alsın {
    mevcut = "" olsun
    dosya_var = dosya_var_mı(gorevler_dosyasi) olsun
    
    dosya_var == 1 ise {
        mevcut = dosya_oku(gorevler_dosyasi) olsun
    }
    
    yeni_icerik = mevcut + gorev + "\\n" olsun
    dosya_yaz(gorevler_dosyasi, yeni_icerik)
    
    "✓ Görev eklendi: " + gorev'i yazdır
}

listele fonksiyon olsun {
    dosya_var = dosya_var_mı(gorevler_dosyasi) olsun
    
    dosya_var == 0 ise {
        "Henüz görev yok."'u yazdır
        0'ı döndür
    }
    
    icerik = dosya_oku(gorevler_dosyasi) olsun
    "=== Görevler ==="'i yazdır
    icerik'i yazdır
}

// Kullanım:
ekle("Alışverişi yap")
ekle("Elektrik faturasını öde")
listele()
`;

const calcCode = `// 2. Gelişmiş Hesap Makinesi
"matematik.hb"'yi yükle

hesapla fonksiyon olsun islem, a, b alsın {
    islem == "topla" ise {
        sonuc = a + b olsun
        a + " + " + b + " = " + sonuc'u yazdır
    } 
    islem == "karekok" ise {
        sonuc = karekok(a) olsun
        "Karekök " + a + " = " + sonuc'u yazdır
    }
    islem == "carp" ise {
        sonuc = a * b olsun
        a + " * " + b + " = " + sonuc'u yazdır
    }
    yoksa {
        "Bilinmeyen işlem türü: " + islem'i yazdır
    }
}

// Uygulamayı Test Edelim:
hesapla("topla", 15, 25)
hesapla("carp", 6, 8)
hesapla("karekok", 144, 0)
`;

const gradesCode = `// 3. Öğrenci Not Takip Sistemi (Listeler ve Döngüler)

not_hesapla fonksiyon olsun isim, notlar alsın {
    toplam = 0 olsun
    boyut = notlar'ın uzunluğu olsun
    
    boyut == 0 ise {
        isim + " için sistemde not bulunamadı."'yı yazdır
        0'ı döndür
    }

    i = 0'dan boyut'a kadar {
        guncel_not = notlar[i] olsun
        toplam = toplam + guncel_not olsun
    }
    
    ortalama = toplam / boyut olsun
    "Öğrenci: " + isim + " | Ortalaması: " + ortalama'yı yazdır
}

// Kullanım örneği:
matematik_notlari = [85, 90, 78, 92, 100] olsun
not_hesapla("Ahmet Yılmaz", matematik_notlari)

// Boş liste ile deneme
fizik_notlari = [] olsun
not_hesapla("Ayşe Kaya", fizik_notlari)
`;

const serverCode = `// 4. Web Sunucu ve API (huma_sunucu)
"huma_sunucu.hb"'yi yükle

uygulama = Sunucu() olsun

// Ana sayfa için işleyici
ana_sayfa fonksiyon olsun istek, cevap alsın {
    mesaj = "Hüma Web Sunucusuna Başarıyla Bağlandınız!" olsun
    cevap'a mesaj'ı gönder
}

// Rotayı tanımlıyoruz
"/", ana_sayfa'yı uygulama'nın rota_ekle'sine ilet

// Hazırsak yayına başlayalım!
8080'i uygulama'nın dinle'siyle çağır
`;

export default async function ProjectsPage({
  params,
}: {
  params: Promise<{ locale: string }>;
}) {
  const { locale } = await params;
  const dict = await getDictionary(locale as "en" | "tr");
  const p = dict.Docs?.projects || {};

  const getPath = (path: string) => `/${locale}${path}`;

  const renderSteps = (steps: string[] | undefined) => {
    if (!steps || !steps.length) return null;
    return (
      <div className="mt-8 bg-surface-container-lowest border border-outline-variant/10 p-6 rounded-2xl relative overflow-hidden group">
        <div className="absolute top-0 right-0 p-4 opacity-[0.03] group-hover:opacity-[0.06] transition-opacity">
          <span className="material-symbols-outlined text-8xl">school</span>
        </div>
        <h3 className="text-sm font-bold text-on-surface uppercase tracking-widest flex items-center gap-2 mb-5">
          <span className="material-symbols-outlined text-primary text-base">school</span>
          {p?.tutorial_title || "Tutorial"}
        </h3>
        <ul className="space-y-4">
          {steps.map((step, idx) => {
            const numMatch = step.match(/^(\\d+)\\.\\s*/);
            const num = numMatch ? numMatch[1] : (idx + 1).toString();
            const text = step.replace(/^\\d+\\.\\s*/, '');
            
            return (
              <li key={idx} className="flex gap-4 text-sm text-on-surface-variant leading-relaxed">
                <span className="w-6 h-6 shrink-0 flex items-center justify-center bg-primary/10 text-primary font-mono text-xs font-bold rounded-full">
                  {num}
                </span>
                <span className="pt-0.5">{text}</span>
              </li>
            )
          })}
        </ul>
      </div>
    );
  };

  return (
    <>
      <main className="flex-1 px-8 md:px-16 py-12 max-w-4xl">
        <nav className="flex gap-2 text-[10px] uppercase tracking-widest text-on-surface-variant/60 mb-4">
          <Link href={getPath("/docs")} className="hover:text-primary transition-colors">
            {dict.Nav.docs}
          </Link>
          <span>/</span>
          <span className="text-on-surface-variant">{dict.Sidebar.community}</span>
          <span>/</span>
          <span className="text-primary">{p?.title || "Projects"}</span>
        </nav>

        <h1 className="text-5xl font-extrabold text-on-surface tracking-tighter mb-6">
          {p?.hero_title}
        </h1>
        <p className="text-lg text-on-surface-variant leading-relaxed mb-12">
          {p?.hero_desc}
        </p>

        {/* Project 1: To-Do App */}
        <section className="mb-20" id="todo">
          <h2 className="text-2xl font-bold text-on-surface mb-4 flex items-center gap-3">
            <span className="w-8 h-8 rounded-full bg-surface-container-high flex items-center justify-center text-sm font-mono text-primary">
              1
            </span>
            {p?.todo?.title?.replace(/^1\\.\\s*/, '')}
          </h2>
          <p className="mb-6 text-on-surface-variant leading-relaxed">
            {p?.todo?.desc}
          </p>
          <CodeBlock code={todoCode} filename="gorevler.hb" />
          {renderSteps(p?.todo?.steps)}
        </section>

        {/* Project 2: Calculator */}
        <section className="mb-20" id="calculator">
          <h2 className="text-2xl font-bold text-on-surface mb-4 flex items-center gap-3">
            <span className="w-8 h-8 rounded-full bg-surface-container-high flex items-center justify-center text-sm font-mono text-primary">
              2
            </span>
            {p?.calculator?.title?.replace(/^2\\.\\s*/, '')}
          </h2>
          <p className="mb-6 text-on-surface-variant leading-relaxed">
            {p?.calculator?.desc}
          </p>
          <CodeBlock code={calcCode} filename="hesap_makinesi.hb" />
          {renderSteps(p?.calculator?.steps)}
        </section>

        {/* Project 3: Grades Tracker */}
        <section className="mb-20" id="grades">
          <h2 className="text-2xl font-bold text-on-surface mb-4 flex items-center gap-3">
            <span className="w-8 h-8 rounded-full bg-surface-container-high flex items-center justify-center text-sm font-mono text-primary">
              3
            </span>
            {p?.grades?.title?.replace(/^3\\.\\s*/, '')}
          </h2>
          <p className="mb-6 text-on-surface-variant leading-relaxed">
            {p?.grades?.desc}
          </p>
          <CodeBlock code={gradesCode} filename="ogrenci_sistemi.hb" />
          {renderSteps(p?.grades?.steps)}
        </section>

        {/* Project 4: HTTP Server */}
        <section className="mb-20" id="server">
          <h2 className="text-2xl font-bold text-on-surface mb-4 flex items-center gap-3">
            <span className="w-8 h-8 rounded-full bg-surface-container-high flex items-center justify-center text-sm font-mono text-primary">
              4
            </span>
            {p?.server?.title?.replace(/^4\\.\\s*/, '')}
          </h2>
          <p className="mb-6 text-on-surface-variant leading-relaxed">
            {p?.server?.desc}
          </p>
          <CodeBlock code={serverCode} filename="sunucu.hb" />
          {renderSteps(p?.server?.steps)}
        </section>

      </main>

      <aside className="hidden xl:block w-64 sticky top-16 h-[calc(100vh-4rem)] py-12 px-8 overflow-y-auto border-l border-outline-variant/10 shrink-0">
        <h5 className="text-[10px] font-bold text-on-surface uppercase tracking-[0.2em] mb-6 opacity-40">
          {locale === "tr" ? "BU SAYFADA" : "ON THIS PAGE"}
        </h5>
        <ul className="space-y-4 text-[11px] font-bold uppercase tracking-widest">
          {[
            { href: "#todo", label: p?.todo?.title?.replace(/^\\d+\\.\\s*/, '') },
            { href: "#calculator", label: p?.calculator?.title?.replace(/^\\d+\\.\\s*/, '') },
            { href: "#grades", label: p?.grades?.title?.replace(/^\\d+\\.\\s*/, '') },
            { href: "#server", label: p?.server?.title?.replace(/^\\d+\\.\\s*/, '') },
          ].map((item) => (
            <li key={item.href}>
              <a
                href={item.href}
                className="text-on-surface-variant/60 hover:text-primary transition-all block"
              >
                {item.label}
              </a>
            </li>
          ))}
        </ul>
      </aside>
    </>
  );
}
