import Nav from '../../components/Nav';
import CodeBlock from '../../components/CodeBlock';
import { EXAMPLES_META, type Lang } from '../../i18n';
import s from './page.module.css';

type T = { text: string; type?: 'kw' | 'fn' | 'str' | 'num' | 'cm' | 'op' };
type L = T[];

function kw(t: string): T { return { text: t, type: 'kw' }; }
function fn_(t: string): T { return { text: t, type: 'fn' }; }
function str(t: string): T { return { text: t, type: 'str' }; }
function num(t: string): T { return { text: t, type: 'num' }; }
function cm(t: string): T { return { text: t, type: 'cm' }; }
function t(text: string): T { return { text }; }

const CODE: L[][] = [
  // 0 — Fibonacci
  [
    [fn_('fibonacci'), t(' '), kw('fonksiyon olsun'), t(' n '), kw('alsın'), t(' {')],
    [t("    n <= "), num('1'), t(' '), kw('ise'), t(" { n'i "), kw('döndür'), t(' }')],
    [t('    fibonacci(n - '), num('1'), t(') + fibonacci(n - '), num('2'), t(")'yi "), kw('döndür')],
    [t('}')],
    [t('')],
    [t("i = "), num('0'), t(' '), kw('olsun')],
    [t('i < '), num('10'), t(' '), kw('olduğu sürece'), t(' {')],
    [t('    '), str('"fib(" + i + ") = " + fibonacci(i)'), t("'yi "), kw('yazdır')],
    [t('    i = i + '), num('1'), t(' '), kw('olsun')],
    [t('}')],
  ],
  // 1 — Sayaç / Counter
  [
    [fn_('sayaç'), t(' '), kw('sınıf olsun'), t(' {')],
    [t('    değer = '), num('0'), t(' '), kw('olsun')],
    [t('')],
    [t('    '), fn_('artır'), t(' '), kw('fonksiyon olsun'), t(' miktar '), kw('alsın'), t(' {')],
    [t('        kendisi.değer = kendisi.değer + miktar '), kw('olsun')],
    [t("        kendisi.değer'i "), kw('döndür')],
    [t('    }')],
    [t('')],
    [t('    '), fn_('sıfırla'), t(' '), kw('fonksiyon olsun'), t(' {')],
    [t('        kendisi.değer = '), num('0'), t(' '), kw('olsun')],
    [t('    }')],
    [t('}')],
    [t('')],
    [t('s = '), fn_('sayaç'), t('() '), kw('olsun')],
    [t('s.artır('), num('5'), t(")'yi "), kw('yazdır')],
    [cm('// Çıktı: 5')],
  ],
  // 2 — Hata / Error
  [
    [fn_('böl'), t(' '), kw('fonksiyon olsun'), t(' a, b '), kw('alsın'), t(' {')],
    [t('    b = '), num('0'), t(' '), kw('ise'), t(' {')],
    [t('        '), str('"Sıfıra bölme"'), t("'yi "), kw('döndür')],
    [t('    }')],
    [t("    (a / b)'yi "), kw('döndür')],
    [t('}')],
    [t('')],
    [kw('dene'), t(' {')],
    [t('    böl('), num('10'), t(', '), num('0'), t(")'u "), kw('yazdır')],
    [t('} '), kw('yakala'), t(' hata {')],
    [t("    "), str('"Yakalandı: "'), t(" + hata'yı "), kw('yazdır')],
    [t('}')],
  ],
  // 3 — Modül / Module
  [
    [cm('// matematik.hb')],
    [fn_('topla'), t(' '), kw('fonksiyon olsun'), t(' a, b '), kw('alsın'), t(' {')],
    [t("    (a + b)'yi "), kw('döndür')],
    [t('}')],
    [t("topla'yı "), kw('dışa aktar')],
    [t('')],
    [cm('// ana.hb')],
    [kw('yükle'), t(' '), str('"matematik.hb"'), t(' '), kw('olarak'), t(' mat')],
    [t('mat.topla('), num('4'), t(', '), num('5'), t(")'i "), kw('yazdır')],
    [cm('// Çıktı: 9')],
  ],
  // 4 — Liste / List
  [
    [t('öğeler = ['), str('"elma"'), t(', '), str('"armut"'), t(', '), str('"kiraz"'), t('] '), kw('olsun')],
    [t('')],
    [t("öğeler'e ["), str('"üzüm"'), t("]'yi ekle")],
    [t("öğeler'in uzunluğu'nu "), kw('yazdır')],
    [cm('// Çıktı: 4')],
    [t('')],
    [t('i = '), num('0'), t(' '), kw('olsun')],
    [t("i < öğeler'in uzunluğu "), kw('olduğu sürece'), t(' {')],
    [t("    öğeler[i]'yi "), kw('yazdır')],
    [t('    i = i + '), num('1'), t(' '), kw('olsun')],
    [t('}')],
  ],
];

const FILES = ['fibonacci.hb', 'sayac.hb', 'hata.hb', 'ana.hb', 'liste.hb'];

export default async function Ornekler({
  params,
}: {
  params: Promise<{ lang: string }>;
}) {
  const { lang } = await params;
  const m = EXAMPLES_META[lang as Lang];

  return (
    <>
      <Nav lang={lang as Lang} path="ornekler" />
      <div className={s.layout}>
        <div className={s.header}>
          <h1 className={s.pageTitle}>{m.pageTitle}</h1>
          <p className={s.pageDesc}>{m.pageDesc}</p>
        </div>

        <div className={s.examples}>
          {m.items.map((item, i) => (
            <div key={item.name} className={s.example}>
              <div className={s.exMeta}>
                <p className={s.exName}>{item.name}</p>
                <p className={s.exDesc}>{item.desc}</p>
                <ul className={s.exConcepts}>
                  {item.concepts.map((c) => (
                    <li key={c}>{c}</li>
                  ))}
                </ul>
              </div>
              <CodeBlock filename={FILES[i]} lines={CODE[i]} lineNumbers />
            </div>
          ))}
        </div>
      </div>
    </>
  );
}
