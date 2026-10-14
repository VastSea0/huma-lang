'use client';
import { useState } from 'react';
import s from './Cmd.module.css';

export default function Cmd({ cmd }: { cmd: string }) {
  const [done, setDone] = useState(false);
  const copy = () => {
    navigator.clipboard.writeText(cmd);
    setDone(true);
    setTimeout(() => setDone(false), 1800);
  };
  return (
    <div className={s.row}>
      <span className={s.dollar}>$</span>
      <code className={s.text}>{cmd}</code>
      <button className={s.btn} onClick={copy} aria-label="kopyala">
        {done ? 'kopyalandı' : 'kopyala'}
      </button>
    </div>
  );
}
