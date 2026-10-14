export const LANGS = ['tr', 'en'] as const;
export type Lang = (typeof LANGS)[number];

export function isLang(l: string): l is Lang {
  return (LANGS as readonly string[]).includes(l);
}

// ─────────────────────────────────────────────────────────────
// Nav
// ─────────────────────────────────────────────────────────────
export const NAV = {
  tr: { docs: 'belgeleme', examples: 'örnekler', github: 'github', switchLabel: 'EN' },
  en: { docs: 'docs', examples: 'examples', github: 'github', switchLabel: 'TR' },
} satisfies Record<Lang, Record<string, string>>;

// ─────────────────────────────────────────────────────────────
// Home page
// ─────────────────────────────────────────────────────────────
export const HOME = {
  tr: {
    heroTitle: ['Türkçe konuşan', 'programlama dili.'],
    heroDesc:
      'Hüma, Türkçe dilbilgisini koruyan genel amaçlı modern bir programlama dili. ' +
      'Anahtar kelimeler, kontrol yapıları ve hal ekleri Türkçedir. ' +
      'Güvenlik odaklı, versiyonlanabilir. Henüz erken aşamada.',
    heroDocsLink: 'Belgelemeyi oku →',
    heroExamplesLink: 'Örnekler',
    sFeatures: 'özellikler',
    sSyntax: 'sözdizimi',
    sStatus: 'bileşen durumu',
    sInstall: 'kurulum',
    installNote:
      'Araç zinciri rust-toolchain.toml ile Rust 1.94.1\'e sabitlidir. ' +
      'Henüz ikili dağıtım yoktur; kaynaktan derleme gerekir.',
    statusCols: ['bileşen', 'durum', 'kapsam'],
    footerLicense: 'hüma — MIT lisansı',
    footerLinks: ['belgeleme', 'dil tanımı', 'değişim günlüğü', 'github'],
    features: [
      {
        name: 'Türkçe sözdizimi',
        body: 'Anahtar kelimeler Türkçedir: fonksiyon olsun, ise, yoksa, döndür, dene/yakala. ' +
          'Hal ekleri apostrof sonrası lexer düzeyinde doğrulanır; ünlü uyumu, kaynaştırma ve ünsüz benzeşmesi kontrol edilir.',
      },
      {
        name: 'Capability-based güvenlik',
        body: 'Dosya okuma/yazma, ağ, süreç, FFI, GUI varsayılan olarak kapalıdır. ' +
          'Her dış dünya erişimi --izin bayrağıyla açıkça verilir. --tüm-izinler yalnızca güvenilen kod içindir.',
      },
      {
        name: 'Açık hata semantiği',
        body: 'Desteklenen kapsamda hatalar sessizce yutulmaz. VM veya AOT desteklemediği bir yapıyı ' +
          'sessiz bir varsayılan üretmek yerine derleme aşamasında açıkça reddeder.',
      },
      {
        name: 'Nesilsel çöp toplayıcı',
        body: 'Kararlı Gc tutamaçları, genç nesil ve yazma bariyeri, major çevrim toplama, paylaşılmayan ' +
          'isolate heap\'leri. Rust üzerine inşa edilmiş, süreç başına izolasyon.',
      },
    ],
    kwTable: [
      ['olsun', 'değişken ataması'],
      ['fonksiyon olsun … alsın', 'fonksiyon tanımı'],
      ['döndür', 'değer döndürme (örtük yok)'],
      ['ise / yoksa', 'koşullu dallanma'],
      ['olduğu sürece', 'while döngüsü'],
      ["N'den M'e kadar", 'aralık döngüsü (her iki uç dahil)'],
      ['dene / yakala', 'hata yönetimi'],
      ['sınıf olsun', 'sınıf tanımı'],
      ['kendisi', 'sınıf içi self referansı'],
      ['yükle / dışa aktar', 'modül sistemi'],
      ['devam / kır', 'döngü kontrolü'],
      ['ve / veya / değil', 'mantık operatörleri'],
    ],
    statusRows: [
      { name: 'Yorumlayıcı', tag: 'stable', label: 'kararlı', scope: 'Fonksiyon, özyineleme, sınıf, liste/sözlük, modül, döngü, dene/yakala' },
      { name: 'Bytecode VM', tag: 'partial', label: 'kısmi', scope: 'Frame/closure, koleksiyonlar, kontrol akışı — desteklenmeyen yapı açıkça reddedilir' },
      { name: 'Cranelift AOT', tag: 'experimental', label: 'deneysel', scope: 'Sayısal ifadeler ve temel kontrol akışı; metin/modül/sınıf reddedilir' },
      { name: 'LSP', tag: 'partial', label: 'kısmi', scope: 'Ayrıştırıcı tanıları, tamamlama, hover, tanıma git' },
      { name: 'HMI', tag: 'stable', label: 'kararlı', scope: 'Süreç dışı sınır, imza/etki/hata kataloğu, uyumluluk denetimi' },
      { name: 'Heap / GC', tag: 'stable', label: 'kararlı', scope: 'Gc tutamaçları, genç nesil + yazma bariyeri, major çevrim, isolate heap' },
      { name: 'Alan kütüphaneleri', tag: 'experimental', label: 'deneysel', scope: 'Çekirdek kabulünün parçası değil, değişebilir veya kaldırılabilir' },
    ],
    installSteps: [
      { label: '# depoyu klonla', cmd: 'git clone https://github.com/VastSea0/huma-lang.git && cd huma-lang' },
      { label: '# derle', cmd: 'cargo build --release --locked' },
      { label: '# doğrula', cmd: './target/release/huma --version' },
      { label: '# test suite', cmd: 'cargo test --workspace --all-targets --locked' },
    ],
  },

  en: {
    heroTitle: ['A programming language', 'that speaks Turkish.'],
    heroDesc:
      'Hüma is a modern general-purpose programming language that preserves Turkish grammar. ' +
      'Keywords, control structures, and grammatical suffixes are all in Turkish. ' +
      'Security-first, versionable. Still in early stages.',
    heroDocsLink: 'Read the docs →',
    heroExamplesLink: 'Examples',
    sFeatures: 'features',
    sSyntax: 'syntax',
    sStatus: 'component status',
    sInstall: 'installation',
    installNote:
      'The toolchain is pinned to Rust 1.94.1 via rust-toolchain.toml. ' +
      'No binary releases yet — build from source.',
    statusCols: ['component', 'status', 'scope'],
    footerLicense: 'hüma — MIT license',
    footerLinks: ['docs', 'language spec', 'changelog', 'github'],
    features: [
      {
        name: 'Turkish syntax',
        body: 'Keywords are Turkish: fonksiyon olsun, ise, yoksa, döndür, dene/yakala. ' +
          'Grammatical suffixes after the apostrophe are validated at the lexer level — vowel harmony, consonant assimilation and lenition are checked.',
      },
      {
        name: 'Capability-based security',
        body: 'File I/O, networking, process spawning, FFI, and GUI are off by default. ' +
          'Every external resource requires an explicit --izin (allow) flag. --tüm-izinler is for trusted code only.',
      },
      {
        name: 'Explicit error semantics',
        body: 'Within the supported scope, errors are never silently swallowed. If the VM or AOT backend ' +
          'encounters an unsupported construct, it rejects it outright at compile time rather than producing a silent default.',
      },
      {
        name: 'Generational garbage collector',
        body: 'Stable Gc handles, young-generation with write barrier, major-cycle collection, and per-isolate ' +
          'unshared heaps. Built on Rust with per-process isolation.',
      },
    ],
    kwTable: [
      ['olsun', 'variable assignment'],
      ['fonksiyon olsun … alsın', 'function definition'],
      ['döndür', 'return value (no implicit return)'],
      ['ise / yoksa', 'conditional branching'],
      ['olduğu sürece', 'while loop'],
      ["N'den M'e kadar", 'range loop (both ends inclusive)'],
      ['dene / yakala', 'error handling'],
      ['sınıf olsun', 'class definition'],
      ['kendisi', 'self reference inside classes'],
      ['yükle / dışa aktar', 'module system'],
      ['devam / kır', 'loop control'],
      ['ve / veya / değil', 'logical operators'],
    ],
    statusRows: [
      { name: 'Interpreter', tag: 'stable', label: 'stable', scope: 'Functions, recursion, classes, list/dict, modules, loops, try/catch' },
      { name: 'Bytecode VM', tag: 'partial', label: 'partial', scope: 'Frame/closure, collections, control flow — unsupported constructs are explicitly rejected' },
      { name: 'Cranelift AOT', tag: 'experimental', label: 'experimental', scope: 'Numeric expressions and basic control flow; strings/modules/classes rejected' },
      { name: 'LSP', tag: 'partial', label: 'partial', scope: 'Parser diagnostics, completion, hover, go-to-definition' },
      { name: 'HMI', tag: 'stable', label: 'stable', scope: 'Out-of-process boundary, signature/effect/error catalogue, compat checks' },
      { name: 'Heap / GC', tag: 'stable', label: 'stable', scope: 'Gc handles, young-gen + write barrier, major cycle, isolate heap' },
      { name: 'Domain libraries', tag: 'experimental', label: 'experimental', scope: 'Not part of core acceptance criteria, subject to change or removal' },
    ],
    installSteps: [
      { label: '# clone the repo', cmd: 'git clone https://github.com/VastSea0/huma-lang.git && cd huma-lang' },
      { label: '# build', cmd: 'cargo build --release --locked' },
      { label: '# verify', cmd: './target/release/huma --version' },
      { label: '# run tests', cmd: 'cargo test --workspace --all-targets --locked' },
    ],
  },
} as const;

// ─────────────────────────────────────────────────────────────
// Docs page
// ─────────────────────────────────────────────────────────────
export const DOCS = {
  tr: {
    pageTitle: 'Dil tanımı',
    pageVersion: 'Hüma 0.6 — kanonik referans',
    sections: [
      { id: 'kaynak-metin', title: '1. Kaynak metin ve tanımlayıcılar' },
      { id: 'degerler',     title: '2. Değerler' },
      { id: 'atama',        title: '3. Atama' },
      { id: 'ek-ayiraci',   title: '4. Ek ayıracı' },
      { id: 'kosullar',     title: '5. Koşullar' },
      { id: 'donguler',     title: '6. Döngüler' },
      { id: 'fonksiyonlar', title: '7. Fonksiyonlar' },
      { id: 'siniflar',     title: '8. Sınıflar' },
      { id: 'hatalar',      title: '9. Hata yönetimi' },
      { id: 'moduller',     title: '10. Modüller' },
      { id: 'guvenlik',     title: '11. Güvenlik modeli' },
      { id: 'sinirlar',     title: '12. Kaynak sınırları' },
    ],
  },
  en: {
    pageTitle: 'Language reference',
    pageVersion: 'Hüma 0.6 — canonical reference',
    sections: [
      { id: 'kaynak-metin', title: '1. Source text & identifiers' },
      { id: 'degerler',     title: '2. Values' },
      { id: 'atama',        title: '3. Assignment' },
      { id: 'ek-ayiraci',   title: '4. Suffix separator' },
      { id: 'kosullar',     title: '5. Conditions' },
      { id: 'donguler',     title: '6. Loops' },
      { id: 'fonksiyonlar', title: '7. Functions' },
      { id: 'siniflar',     title: '8. Classes' },
      { id: 'hatalar',      title: '9. Error handling' },
      { id: 'moduller',     title: '10. Modules' },
      { id: 'guvenlik',     title: '11. Security model' },
      { id: 'sinirlar',     title: '12. Resource limits' },
    ],
  },
} as const;

// ─────────────────────────────────────────────────────────────
// Examples page
// ─────────────────────────────────────────────────────────────
export const EXAMPLES_META = {
  tr: {
    pageTitle: 'Örnekler',
    pageDesc: 'Gerçek Hüma kaynak dosyalarından alınan, çalışan kod örnekleri.',
    items: [
      { name: 'Fibonacci', desc: 'Özyinelemeli fibonacci hesabı ve while döngüsüyle seri çıktısı.', concepts: ['fonksiyon', 'özyineleme', 'olduğu sürece', 'koşul'] },
      { name: 'Sayaç sınıfı', desc: 'Alan ve metot içeren basit bir sınıf. kendisi ile öz-referans.', concepts: ['sınıf', 'kendisi', 'metot', 'alan'] },
      { name: 'Hata yönetimi', desc: 'dene/yakala ile çalışma zamanı hatalarını yakalama.', concepts: ['dene', 'yakala', 'hata yayılımı'] },
      { name: 'Modül sistemi', desc: 'Bir modül dosyasından fonksiyon dışa aktarma ve takma adla yükleme.', concepts: ['yükle', 'dışa aktar', 'olarak'] },
      { name: 'Liste işlemleri', desc: 'Liste oluşturma, ekleme, indeksleme ve Türkçe sözdizimi ile erişim.', concepts: ["liste", "ekle", "çıkar", "'nin uzunluğu"] },
    ],
  },
  en: {
    pageTitle: 'Examples',
    pageDesc: 'Working code examples taken from real Hüma source files.',
    items: [
      { name: 'Fibonacci', desc: 'Recursive fibonacci computation and series output with a while loop.', concepts: ['function', 'recursion', 'while loop', 'condition'] },
      { name: 'Counter class', desc: 'A simple class with a field and a method. Self-reference with kendisi.', concepts: ['class', 'kendisi (self)', 'method', 'field'] },
      { name: 'Error handling', desc: 'Catching runtime errors with dene/yakala (try/catch).', concepts: ['dene (try)', 'yakala (catch)', 'error propagation'] },
      { name: 'Module system', desc: 'Exporting a function from a module file and loading it with an alias.', concepts: ['yükle (import)', 'dışa aktar (export)', 'alias'] },
      { name: 'List operations', desc: 'Creating, appending, indexing, and iterating over lists with Turkish syntax.', concepts: ['list', 'append', 'remove', 'length'] },
    ],
  },
} as const;
