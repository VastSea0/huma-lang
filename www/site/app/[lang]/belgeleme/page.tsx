import Nav from '../../components/Nav';
import CodeBlock from '../../components/CodeBlock';
import { DOCS, type Lang } from '../../i18n';
import s from './page.module.css';

type T = { text: string; type?: 'kw' | 'fn' | 'str' | 'num' | 'cm' | 'op' };
type L = T[];

function kw(t: string): T { return { text: t, type: 'kw' }; }
function fn_(t: string): T { return { text: t, type: 'fn' }; }
function str(t: string): T { return { text: t, type: 'str' }; }
function num(t: string): T { return { text: t, type: 'num' }; }
function cm(t: string): T { return { text: t, type: 'cm' }; }
function t(text: string): T { return { text }; }

const CODE: Record<string, L[]> = {
  degerler: [
    [t('sayı = '), num('42'), t(' '), kw('olsun')],
    [t('ondalık = '), num('1.25e-3'), t(' '), kw('olsun')],
    [t('metin = '), str('"Türkçe UTF-8"'), t(' '), kw('olsun')],
    [t('evet = '), kw('doğru'), t(' '), kw('olsun')],
    [t('yok = '), kw('boş'), t(' '), kw('olsun')],
    [t('dizi = ['), num('1'), t(', '), num('2'), t(', '), num('3'), t('] '), kw('olsun')],
    [t('sözlük = {'), str('"ad"'), t(': '), str('"Hüma"'), t('} '), kw('olsun')],
  ],
  atama: [
    [t('x = '), num('10'), t(' '), kw('olsun')],
    [t('x = x + '), num('1'), t(' '), kw('olsun')],
    [t('dizi['), num('0'), t('] = '), num('20'), t(' '), kw('olsun')],
    [t('nesne.alan = '), num('3'), t(' '), kw('olsun')],
  ],
  ekler: [
    [t("değer'i "), kw('yazdır')],
    [t("metin'i "), kw('döndür')],
    [t("1'den 5'e "), kw('kadar')],
    [t("dizi'nin uzunluğu")],
  ],
  kosullar: [
    [t('x > '), num('0'), t(' '), kw('ve'), t(' x < '), num('10'), t(' '), kw('ise'), t(' {')],
    [t('    '), str('"aralıkta"'), t("'yı "), kw('yazdır')],
    [t('} '), kw('yoksa'), t(' {')],
    [t('    '), str('"dışarıda"'), t("'yı "), kw('yazdır')],
    [t('}')],
  ],
  donguler: [
    [t('i = '), num('0'), t(' '), kw('olsun')],
    [t('i < '), num('10'), t(' '), kw('olduğu sürece'), t(' {')],
    [t('    i = i + '), num('1'), t(' '), kw('olsun')],
    [t('    i = '), num('4'), t(' '), kw('ise'), t(' { '), kw('devam'), t(' }')],
    [t('    i = '), num('8'), t(' '), kw('ise'), t(' { '), kw('kır'), t(' }')],
    [t('}')],
    [t('')],
    [t("j = 1'den 5'e "), kw('kadar'), t(' {')],
    [t("    j'yi "), kw('yazdır')],
    [t('}')],
  ],
  fonksiyonlar: [
    [fn_('topla'), t(' '), kw('fonksiyon olsun'), t(' a, b '), kw('alsın'), t(' {')],
    [t("    (a + b)'yi "), kw('döndür')],
    [t('}')],
    [t('')],
    [t('sonuç = topla('), num('2'), t(', '), num('3'), t(') '), kw('olsun')],
  ],
  siniflar: [
    [fn_('sayaç'), t(' '), kw('sınıf olsun'), t(' {')],
    [t('    değer = '), num('0'), t(' '), kw('olsun')],
    [t('')],
    [t('    '), fn_('artır'), t(' '), kw('fonksiyon olsun'), t(' miktar '), kw('alsın'), t(' {')],
    [t('        kendisi.değer = kendisi.değer + miktar '), kw('olsun')],
    [t("        kendisi.değer'i "), kw('döndür')],
    [t('    }')],
    [t('}')],
    [t('')],
    [t('s = '), fn_('sayaç'), t('() '), kw('olsun')],
    [t('s.artır('), num('2'), t(")'yi "), kw('yazdır')],
  ],
  hatalar: [
    [kw('dene'), t(' {')],
    [t('    '), num('10'), t(' / '), num('0'), t("'ı "), kw('yazdır')],
    [t('} '), kw('yakala'), t(' hata {')],
    [t("    hata'yı "), kw('yazdır')],
    [t('}')],
  ],
  moduller: [
    [cm('// hesap.hb')],
    [fn_('topla'), t(' '), kw('fonksiyon olsun'), t(' a, b '), kw('alsın'), t(' { (a + b)\'yi '), kw('döndür'), t(' }')],
    [t("topla'yı "), kw('dışa aktar')],
    [t('')],
    [cm('// ana.hb')],
    [kw('yükle'), t(' '), str('"hesap.hb"'), t(' '), kw('olarak'), t(' hesap')],
    [t('hesap.topla('), num('2'), t(', '), num('3'), t(")'ü "), kw('yazdır')],
  ],
  guvenlik: [
    [cm('# huma.json → "veri": "huma run uygulama.hb --izin dosya-okuma --izin ağ-istemci"')],
    [t('huma paket run veri')],
  ],
};

// ── Language-dependent section bodies ─────────────────────────────────
function SectionBodies({ lang }: { lang: Lang }) {
  if (lang === 'tr') return <TrSections />;
  return <EnSections />;
}

function TrSections() {
  return (
    <>
      <div id="kaynak-metin" className={s.docSection}>
        <h2>1. Kaynak metin ve tanımlayıcılar</h2>
        <p>Kaynak dosyaları UTF-8 kodlamalıdır ve <code>.hb</code> uzantısı kullanır. Tek kaynak dosyası en fazla 64 MiB olabilir.</p>
        <p>Tanımlayıcılar Unicode harf, rakam ve <code>_</code> içerebilir; ilk karakter harf veya <code>_</code> olmalıdır. Lexer tanımlayıcıları Unicode NFC biçimine getirir. Türkçe karakterler doğrudan kullanılabilir: <code>değer</code>, <code>öğrenci_sayısı</code>.</p>
        <p>Anahtar sözcükler tanımlayıcı olarak kullanılamaz. <code>doğru</code>, <code>yanlış</code>, <code>liste</code>, <code>devam</code>, <code>kır</code> ve ASCII eşdeğerleri ayrılmıştır.</p>
        <ul>
          <li><code>{"//"}</code> ile satır sonuna kadar yorum yazılır.</li>
          <li>Noktalı virgül isteğe bağlıdır.</li>
          <li>Kaçış dizileri: <code>{'\\n \\r \\t \\\\ \\" \\xNN'}</code>. Bilinmeyen kaçışlar hatadır.</li>
        </ul>
      </div>

      <div id="degerler" className={s.docSection}>
        <h2>2. Değerler</h2>
        <p>Hüma 0.6&apos;nın tek sayısal türü sonlu IEEE-754 <code>f64</code>&apos;tür. <code>NaN</code> veya sonsuz değer üretilemez; böyle bir sonuç yakalanabilir çalışma zamanı hatasıdır. Metinler sayısal işlemlerde örtük olarak sayıya dönüştürülmez.</p>
        <div className={s.codeWrap}><CodeBlock filename="degerler.hb" lines={CODE.degerler} /></div>
        <p><code>+</code> iki sayıyı toplar; işlenenlerden en az biri metinse diğer değerin kanonik gösterimini metne ekler. Sözlük anahtarları yalnızca metin olabilir.</p>
      </div>

      <div id="atama" className={s.docSection}>
        <h2>3. Atama</h2>
        <p>Atama cümlesi <code>olsun</code> ile biter. Sol taraf yalnızca değişken, nesne alanı veya indekslenebilir öğe olabilir.</p>
        <div className={s.codeWrap}><CodeBlock filename="atama.hb" lines={CODE.atama} /></div>
        <p>Eşitlik için ifadelerde <code>=</code> veya <code>==</code> kabul edilir; atama yalnızca cümle sonundaki <code>olsun</code> ile oluşur.</p>
      </div>

      <div id="ek-ayiraci" className={s.docSection}>
        <h2>4. Ek ayıracı</h2>
        <p>Apostrof, bir Hüma tanımlayıcısını Türkçe görünümlü durum ekinden ayıran programlama dili işaretidir. Bilinmeyen ekler sözdizimi hatasıdır.</p>
        <div className={s.codeWrap}><CodeBlock filename="ekler.hb" lines={CODE.ekler} /></div>
        <p>Kabul edilen ek biçimleri:</p>
        <ul>
          <li><strong>Belirtme:</strong> i ı u ü yi yı yu yü ni nı nu nü</li>
          <li><strong>Yönelme:</strong> a e ya ye</li>
          <li><strong>Bulunma:</strong> da de ta te</li>
          <li><strong>Ayrılma:</strong> dan den tan ten</li>
          <li><strong>İlgi:</strong> nin nın nun nün in ın un ün</li>
          <li><strong>Diğer:</strong> si sı su sü lar ler ca ce ça çe daki deki la le yla yle</li>
        </ul>
        <p>Lexer, ünlü uyumu, kaynaştırma ve ünsüz benzeşmesini doğrular. Tek harfli adlar ve kısaltmalarda yalnızca ekin tanımlı olması denetlenir.</p>
      </div>

      <div id="kosullar" className={s.docSection}>
        <h2>5. Koşullar ve mantık</h2>
        <div className={s.codeWrap}><CodeBlock filename="kosul.hb" lines={CODE.kosullar} /></div>
        <p>Operatör önceliği (yüksekten düşüğe): parantez/erişim/çağrı, tekli <code>-</code> ve <code>değil</code>, <code>* / %</code>, <code>+ -</code>, karşılaştırma, eşitlik, <code>ve</code>, <code>veya</code>.</p>
        <p><code>ve</code> ve <code>veya</code> kısa devrelidir. Doğruluk: <code>boş</code>, <code>0</code>, boş metin, boş liste ve boş sözlük yanlıştır; diğer geçerli değerler doğrudur.</p>
      </div>

      <div id="donguler" className={s.docSection}>
        <h2>6. Döngüler</h2>
        <div className={s.codeWrap}><CodeBlock filename="dongu.hb" lines={CODE.donguler} /></div>
        <p>Aralık döngüsünde her iki uç dahildir. <code>devam</code> ve <code>kır</code> yalnızca döngü içinde geçerlidir.</p>
      </div>

      <div id="fonksiyonlar" className={s.docSection}>
        <h2>7. Fonksiyonlar ve kapsam</h2>
        <div className={s.codeWrap}><CodeBlock filename="fonksiyon.hb" lines={CODE.fonksiyonlar} /></div>
        <p>Fonksiyon parametreleri ve fonksiyon içinde ilk kez atanan adlar yerel kapsamdadır. Örtük dönüş yoktur; <code>döndür</code> çalışmazsa sonuç <code>Boş</code> olur.</p>
        <p>Argüman sayısı parametre sayısıyla tam eşleşmelidir. Anonim ve iç içe fonksiyonlar tanımlandıkları andaki sözcüksel kapsamı yakalar. Varsayılan azami çağrı derinliği 32&apos;dir.</p>
      </div>

      <div id="siniflar" className={s.docSection}>
        <h2>8. Sınıflar</h2>
        <div className={s.codeWrap}><CodeBlock filename="sinif.hb" lines={CODE.siniflar} /></div>
        <p>Sınıf çağrısı argümansız yapılır; alanlar oluşturulduktan sonra açık bir <code>ilklendir</code> metodu çağrılabilir. <code>kendisi&apos;nin değer&apos;i</code> biçimi de özellik erişimi için kabul edilir.</p>
      </div>

      <div id="hatalar" className={s.docSection}>
        <h2>9. Hata yönetimi</h2>
        <div className={s.codeWrap}><CodeBlock filename="hata.hb" lines={CODE.hatalar} /></div>
        <p>Tanımsız ad, tip uyuşmazlığı, sıfıra bölme, sınır dışı indeks, çağrılamayan değer ve bulunamayan modül hata olarak yayılır.</p>
      </div>

      <div id="moduller" className={s.docSection}>
        <h2>10. Modüller</h2>
        <div className={s.codeWrap}><CodeBlock filename="modul.hb" lines={CODE.moduller} /></div>
        <p>Arama sırası: mevcut dizin → <code>lib/</code> → <code>huma_modulleri/</code> → yüklenen modülün dizini. Aynı çözülmüş modül bir örnekte bir kez yüklenir; döngüsel yükleme açık hata verir.</p>
      </div>

      <div id="guvenlik" className={s.docSection}>
        <h2>11. Güvenlik modeli</h2>
        <div className={s.codeWrap}><CodeBlock filename="terminal" lines={CODE.guvenlik} /></div>
        <p>Dosya okuma/yazma, ağ, süreç, FFI, veritabanı ve GUI yetenekleri varsayılan olarak kapalıdır. <code>--tüm-izinler</code> yalnızca güvenilen kod içindir.</p>
        <p>Native kütüphanelerin varsayılan yolu süreç dışı HMI&apos;dır. FFI yalnızca <code>f64()</code>, <code>f64(f64)</code> ve <code>f64(f64,f64)</code> imzalarını kabul eder.</p>
      </div>

      <div id="sinirlar" className={s.docSection}>
        <h2>12. Kaynak sınırları</h2>
        <ul>
          <li>Yürütme adımı: 10.000.000</li>
          <li>İç içe çağrı derinliği: 32</li>
          <li>Dil çıktısı: 16 MiB</li>
          <li>Tek koleksiyon literali: 1.000.000 öğe</li>
          <li>Komut/ifade iç içe geçmesi: 200 düzey (sözdizimi hatası)</li>
          <li>Kaynak ve bytecode payload: 64 MiB</li>
        </ul>
        <p>Sınır aşımı sessiz kısaltma değil, yakalanabilir hata üretir.</p>
      </div>
    </>
  );
}

function EnSections() {
  return (
    <>
      <div id="kaynak-metin" className={s.docSection}>
        <h2>1. Source text &amp; identifiers</h2>
        <p>Source files must be UTF-8 encoded and use the <code>.hb</code> extension. A single source file may not exceed 64 MiB.</p>
        <p>Identifiers may contain Unicode letters, digits, and <code>_</code>; the first character must be a letter or <code>_</code>. The lexer normalises identifiers to Unicode NFC. Turkish characters are used directly: <code>değer</code>, <code>öğrenci_sayısı</code>.</p>
        <p>Keywords cannot be used as identifiers. <code>doğru</code>, <code>yanlış</code>, <code>liste</code>, <code>devam</code>, <code>kır</code> and their ASCII equivalents are reserved.</p>
        <ul>
          <li><code>{"//"}</code> begins a line comment.</li>
          <li>Semicolons are optional.</li>
          <li>Escape sequences: <code>{'\\n \\r \\t \\\\ \\" \\xNN'}</code>. Unknown escapes are errors.</li>
        </ul>
      </div>

      <div id="degerler" className={s.docSection}>
        <h2>2. Values</h2>
        <p>Hüma 0.6&apos;s only numeric type is finite IEEE-754 <code>f64</code>. <code>NaN</code> and infinity cannot be produced; such a result is a catchable runtime error. Strings are never implicitly coerced to numbers.</p>
        <div className={s.codeWrap}><CodeBlock filename="values.hb" lines={CODE.degerler} /></div>
        <p><code>+</code> adds two numbers; if either operand is a string, the other is converted to its canonical string form. Dictionary keys must be strings.</p>
      </div>

      <div id="atama" className={s.docSection}>
        <h2>3. Assignment</h2>
        <p>An assignment statement ends with <code>olsun</code>. The left-hand side must be a variable, object field, or indexable element.</p>
        <div className={s.codeWrap}><CodeBlock filename="assign.hb" lines={CODE.atama} /></div>
        <p>Both <code>=</code> and <code>==</code> are accepted for equality in expressions; assignment is only created by a trailing <code>olsun</code>.</p>
      </div>

      <div id="ek-ayiraci" className={s.docSection}>
        <h2>4. Suffix separator</h2>
        <p>The apostrophe separates a Hüma identifier from a Turkish-looking grammatical suffix. Unknown suffixes are syntax errors.</p>
        <div className={s.codeWrap}><CodeBlock filename="suffixes.hb" lines={CODE.ekler} /></div>
        <p>Accepted suffix forms:</p>
        <ul>
          <li><strong>Accusative:</strong> i ı u ü yi yı yu yü ni nı nu nü</li>
          <li><strong>Dative:</strong> a e ya ye</li>
          <li><strong>Locative:</strong> da de ta te</li>
          <li><strong>Ablative:</strong> dan den tan ten</li>
          <li><strong>Genitive:</strong> nin nın nun nün in ın un ün</li>
          <li><strong>Other:</strong> si sı su sü lar ler ca ce ça çe daki deki la le yla yle</li>
        </ul>
        <p>The lexer validates vowel harmony, consonant lenition, and assimilation for names where pronunciation can be inferred from spelling.</p>
      </div>

      <div id="kosullar" className={s.docSection}>
        <h2>5. Conditions &amp; logic</h2>
        <div className={s.codeWrap}><CodeBlock filename="cond.hb" lines={CODE.kosullar} /></div>
        <p>Operator precedence (highest to lowest): parentheses/access/call, unary <code>-</code> and <code>değil</code>, <code>* / %</code>, <code>+ -</code>, comparison, equality, <code>ve</code>, <code>veya</code>.</p>
        <p><code>ve</code> and <code>veya</code> are short-circuit. Falsy values: <code>boş</code>, <code>0</code>, empty string, empty list, empty dict. Everything else is truthy.</p>
      </div>

      <div id="donguler" className={s.docSection}>
        <h2>6. Loops</h2>
        <div className={s.codeWrap}><CodeBlock filename="loops.hb" lines={CODE.donguler} /></div>
        <p>Range loops are inclusive on both ends. <code>devam</code> and <code>kır</code> are only valid inside loops.</p>
      </div>

      <div id="fonksiyonlar" className={s.docSection}>
        <h2>7. Functions &amp; scope</h2>
        <div className={s.codeWrap}><CodeBlock filename="fn.hb" lines={CODE.fonksiyonlar} /></div>
        <p>Function parameters and names first assigned inside a function are local. There is no implicit return; if <code>döndür</code> never runs, the result is <code>Boş</code> (null).</p>
        <p>Argument count must exactly match the parameter count. Nested functions close over the lexical scope at the time of definition. Default maximum call depth is 32.</p>
      </div>

      <div id="siniflar" className={s.docSection}>
        <h2>8. Classes</h2>
        <div className={s.codeWrap}><CodeBlock filename="class.hb" lines={CODE.siniflar} /></div>
        <p>A class is instantiated with no arguments; an explicit <code>ilklendir</code> method may be called afterward. The <code>kendisi&apos;nin alan&apos;ı</code> form is also accepted for field access.</p>
      </div>

      <div id="hatalar" className={s.docSection}>
        <h2>9. Error handling</h2>
        <div className={s.codeWrap}><CodeBlock filename="errors.hb" lines={CODE.hatalar} /></div>
        <p>Undefined names, type mismatches, division by zero, out-of-bounds indexing, uncallable values, and missing modules all propagate as errors.</p>
      </div>

      <div id="moduller" className={s.docSection}>
        <h2>10. Modules</h2>
        <div className={s.codeWrap}><CodeBlock filename="modules.hb" lines={CODE.moduller} /></div>
        <p>Search order: current directory → <code>lib/</code> → <code>huma_modulleri/</code> → the loading module&apos;s directory. The same resolved module is loaded only once per instance; circular loading is an explicit error.</p>
      </div>

      <div id="guvenlik" className={s.docSection}>
        <h2>11. Security model</h2>
        <div className={s.codeWrap}><CodeBlock filename="terminal" lines={CODE.guvenlik} /></div>
        <p>File I/O, networking, process spawning, FFI, database, and GUI capabilities are off by default. <code>--tüm-izinler</code> is for trusted code only and does not provide OS sandboxing.</p>
        <p>The default path for native libraries is the out-of-process HMI. In-process narrow FFI requires <code>--güvenilir-süreç-içi-ffi</code>. FFI only accepts <code>f64()</code>, <code>f64(f64)</code>, and <code>f64(f64,f64)</code> signatures.</p>
      </div>

      <div id="sinirlar" className={s.docSection}>
        <h2>12. Resource limits</h2>
        <ul>
          <li>Execution steps: 10,000,000</li>
          <li>Call depth: 32</li>
          <li>Language output: 16 MiB</li>
          <li>Single collection literal: 1,000,000 elements</li>
          <li>Statement/expression nesting: 200 levels (syntax error)</li>
          <li>Source and bytecode payload: 64 MiB</li>
        </ul>
        <p>Exceeding a limit produces a catchable error, not silent truncation.</p>
      </div>
    </>
  );
}

export default async function Belgeleme({
  params,
}: {
  params: Promise<{ lang: string }>;
}) {
  const { lang } = await params;
  const d = DOCS[lang as Lang];

  return (
    <>
      <Nav lang={lang as Lang} path="belgeleme" />
      <div style={{ maxWidth: 'var(--w)', margin: '0 auto', padding: '0 var(--px)' }}>
        <div className={s.layout}>
          <aside className={s.sidebar}>
            <p className={s.sidebarTitle}>{lang === 'tr' ? 'belgeleme' : 'reference'}</p>
            <ul className={s.sidebarNav}>
              {d.sections.map((sec) => (
                <li key={sec.id}>
                  <a href={`#${sec.id}`}>{sec.title}</a>
                </li>
              ))}
            </ul>
          </aside>

          <article className={s.content}>
            <h1 className={s.pageTitle}>{d.pageTitle}</h1>
            <p className={s.pageVersion}>{d.pageVersion}</p>
            <SectionBodies lang={lang as Lang} />
          </article>
        </div>
      </div>
    </>
  );
}
