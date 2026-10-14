import s from './CodeBlock.module.css';

type TokenType = 'kw' | 'fn' | 'str' | 'num' | 'cm' | 'op';
type Token = { text: string; type?: TokenType };
type Line = Token[];

interface Props {
  filename?: string;
  lines: Line[];
  lineNumbers?: boolean;
}

export default function CodeBlock({ filename, lines, lineNumbers = false }: Props) {
  return (
    <div className={s.wrap}>
      {filename && <span className={s.filename}>{filename}</span>}
      <div className={s.pre}>
        {lines.map((tokens, i) => (
          <div key={i} className={s.line}>
            {lineNumbers && <span className={s.ln}>{i + 1}</span>}
            <span className={s.code}>
              {tokens.map((t, j) =>
                t.type
                  ? <span key={j} className={s[t.type]}>{t.text}</span>
                  : <span key={j}>{t.text}</span>
              )}
            </span>
          </div>
        ))}
      </div>
    </div>
  );
}
