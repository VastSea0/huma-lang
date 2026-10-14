import Nav from '../components/Nav';
import CodeBlock from '../components/CodeBlock';
import Cmd from '../components/Cmd';
import { HOME, type Lang } from '../i18n';
import s from './page.module.css';

type T = { text: string; type?: 'kw' | 'fn' | 'str' | 'num' | 'cm' | 'op' };
type L = T[];

function kw(text: string): T { return { text, type: 'kw' }; }
function fn_(text: string): T { return { text, type: 'fn' }; }
function str(text: string): T { return { text, type: 'str' }; }
function num(text: string): T { return { text, type: 'num' }; }
function t(text: string): T { return { text }; }

const HERO_EXAMPLE: L[] = [
  [fn_('fibonacci'), t(' '), kw('fonksiyon olsun'), t(' n '), kw('alsın'), t(' {')],
  [t('    n <= '), num('1'), t(' '), kw('ise'), t(" { n'i "), kw('döndür'), t(' }')],
  [t('    fibonacci(n - '), num('1'), t(') + fibonacci(n - '), num('2'), t(")'yi "), kw('döndür')],
  [t('}')],
  [t('')],
  [t('i = '), num("0'dan"), t(' '), num("9'a"), t(' '), kw('kadar'), t(' {')],
  [t('    '), str('"fib(" + i + ") = " + fibonacci(i)'), t("'yi "), kw('yazdır')],
  [t('}')],
];

const SYNTAX_EXAMPLE: L[] = [
  [t('ad = '), str('"Hüma"'), t(' '), kw('olsun')],
  [t('')],
  [fn_('topla'), t(' '), kw('fonksiyon olsun'), t(' a, b '), kw('alsın'), t(' {')],
  [t("    (a + b)'yi "), kw('döndür')],
  [t('}')],
  [t('')],
  [t('sonuç > '), num('5'), t(' '), kw('ise'), t(' {')],
  [t('    '), str('"büyük"'), t("'ü "), kw('yazdır')],
  [t('} '), kw('yoksa'), t(' {')],
  [t('    '), str('"küçük"'), t("'ü "), kw('yazdır')],
  [t('}')],
  [t('')],
  [kw('dene'), t(' {')],
  [t('    '), num('10'), t(' / '), num('0'), t("'ı "), kw('yazdır')],
  [t('} '), kw('yakala'), t(' hata {')],
  [t("    hata'yı "), kw('yazdır')],
  [t('}')],
];

const TAG_CLASS: Record<string, string> = {
  stable: s.tagStable,
  partial: s.tagPartial,
  experimental: s.tagExperimental,
};

export default async function HomePage({
  params,
}: {
  params: Promise<{ lang: string }>;
}) {
  const { lang } = await params;
  const h = HOME[lang as Lang];
  const base = `/${lang}`;

  return (
    <>
      <Nav lang={lang as Lang} path="" />
      <main className={s.page}>

        {/* ── Hero ── */}
        <section className={s.hero}>
          <h1 className={s.heroTitle}>
            {h.heroTitle[0]}<br />
            <em>{h.heroTitle[1]}</em>
          </h1>
          <p className={s.heroDesc}>{h.heroDesc}</p>
          <div className={s.heroCode}>
            <CodeBlock filename="fibonacci.hb" lines={HERO_EXAMPLE} />
          </div>
          <div className={s.heroLinks}>
            <a href={`${base}/belgeleme`}>{h.heroDocsLink}</a>
            <a href={`${base}/ornekler`}>{h.heroExamplesLink}</a>
            <a href="https://github.com/VastSea0/huma-lang" target="_blank" rel="noopener noreferrer">
              GitHub
            </a>
          </div>
        </section>

        {/* ── Özellikler / Features ── */}
        <section className={s.section}>
          <p className={s.sectionTitle}>{h.sFeatures}</p>
          <div className={s.props}>
            {h.features.map((f) => (
              <div key={f.name} className={s.prop}>
                <p className={s.propName}>{f.name}</p>
                <p className={s.propText}>{f.body}</p>
              </div>
            ))}
          </div>
        </section>

        {/* ── Sözdizimi / Syntax ── */}
        <section className={s.section}>
          <p className={s.sectionTitle}>{h.sSyntax}</p>
          <div className={s.syntaxGrid}>
            <table className={s.kwTable}>
              <tbody>
                {h.kwTable.map(([kw, def]) => (
                  <tr key={kw}>
                    <td>{kw}</td>
                    <td>{def}</td>
                  </tr>
                ))}
              </tbody>
            </table>
            <CodeBlock filename="ornek.hb" lines={SYNTAX_EXAMPLE} lineNumbers />
          </div>
        </section>

        {/* ── Durum / Status ── */}
        <section className={s.section}>
          <p className={s.sectionTitle}>{h.sStatus}</p>
          <table className={s.statusTable}>
            <thead>
              <tr>
                {h.statusCols.map((c) => <th key={c}>{c}</th>)}
              </tr>
            </thead>
            <tbody>
              {h.statusRows.map((r) => (
                <tr key={r.name}>
                  <td>{r.name}</td>
                  <td><span className={`${s.tag} ${TAG_CLASS[r.tag]}`}>{r.label}</span></td>
                  <td>{r.scope}</td>
                </tr>
              ))}
            </tbody>
          </table>
        </section>

        {/* ── Kurulum / Install ── */}
        <section className={s.section}>
          <p className={s.sectionTitle}>{h.sInstall}</p>
          <p className={s.installNote}>{h.installNote}</p>
          <div className={s.installSteps}>
            {h.installSteps.map((step) => (
              <div key={step.label} className={s.step}>
                <span className={s.stepLabel}>{step.label}</span>
                <Cmd cmd={step.cmd} />
              </div>
            ))}
          </div>
        </section>

        {/* ── Footer ── */}
        <footer className={s.footer}>
          <span className={s.footerLeft}>{h.footerLicense}</span>
          <ul className={s.footerLinks}>
            <li><a href={`${base}/belgeleme`}>{h.footerLinks[0]}</a></li>
            <li>
              <a href="https://github.com/VastSea0/huma-lang/blob/main/docs/DIL_TANIMI.md"
                 target="_blank" rel="noopener noreferrer">
                {h.footerLinks[1]}
              </a>
            </li>
            <li>
              <a href="https://github.com/VastSea0/huma-lang/blob/main/CHANGELOG.md"
                 target="_blank" rel="noopener noreferrer">
                {h.footerLinks[2]}
              </a>
            </li>
            <li>
              <a href="https://github.com/VastSea0/huma-lang"
                 target="_blank" rel="noopener noreferrer">
                {h.footerLinks[3]}
              </a>
            </li>
          </ul>
        </footer>
      </main>
    </>
  );
}
