// @ts-check
import mdx from '@astrojs/mdx';
import sitemap from '@astrojs/sitemap';
import { unified } from '@astrojs/markdown-remark';
import { defineConfig } from 'astro/config';
import { remarkReadingTime } from './src/assets/utils/reading-time';
import config from './src/config';

const site = config.site_url;
if (!site) {
  throw new Error('site_url in src/config.ts must be an absolute HTTP(S) URL.');
}

let siteUrl;

try {
  siteUrl = new URL(site);
} catch {
  throw new Error('site_url in src/config.ts must be an absolute HTTP(S) URL.');
}

if (siteUrl.protocol !== 'http:' && siteUrl.protocol !== 'https:') {
  throw new Error('site_url in src/config.ts must be an absolute HTTP(S) URL.');
}

// https://astro.build/config
export default defineConfig({
  site,
  integrations: [mdx(), sitemap()],
  markdown: {
    processor: unified({ remarkPlugins: [remarkReadingTime] }),
  },
});
