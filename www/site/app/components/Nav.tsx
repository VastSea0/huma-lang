import Link from 'next/link';
import s from './Nav.module.css';
import { NAV, type Lang } from '../i18n';

interface Props {
  lang: Lang;
  /** current route segment after /[lang]/, e.g. '' | 'belgeleme' | 'ornekler' */
  path?: string;
}

export default function Nav({ lang, path = '' }: Props) {
  const n = NAV[lang];
  const other = lang === 'tr' ? 'en' : 'tr';

  // Build the same page URL in the other language
  const otherBase = `/${other}`;
  const otherPath = path ? `/${path}` : '';
  const otherHref = `${otherBase}${otherPath}`;

  const base = `/${lang}`;

  return (
    <nav className={s.nav}>
      <div className={s.inner}>
        <Link href={base} className={s.logo}>
          hüma<span>.</span>
        </Link>

        <div className={s.links}>
          <Link href={`${base}/belgeleme`}>{n.docs}</Link>
          <span className={s.sep}>·</span>
          <Link href={`${base}/ornekler`}>{n.examples}</Link>
          <span className={s.sep}>·</span>
          <a href="https://github.com/VastSea0/huma-lang" target="_blank" rel="noopener noreferrer">
            {n.github}
          </a>
        </div>

        <div className={s.langSwitch}>
          <span className={s.langActive}>{lang.toUpperCase()}</span>
          <span className={s.langSep}>/</span>
          <Link href={otherHref}>{other.toUpperCase()}</Link>
        </div>
      </div>
    </nav>
  );
}
