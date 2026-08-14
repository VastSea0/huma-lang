import { defineConfig } from 'astro/config';

// https://astro.build/config
export default defineConfig({
  site: 'https://huma-lang.org',
  base: process.env.ASTRO_BASE !== undefined ? process.env.ASTRO_BASE : '/projects/huma-lang',
  output: 'static'
});
