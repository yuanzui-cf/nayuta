import { describe, expect, test } from 'bun:test';
import { detectLocale, I18n, locales, matchLocale } from '../src/i18n';
import { messageKeys } from '../src/i18n/types';

describe('locale detection', () => {
  test.each([
    ['zh_CN.UTF-8', 'zh-CN'],
    ['zh-Hans', 'zh-CN'],
    ['zh-Hans-CN', 'zh-CN'],
    ['zh_TW.UTF-8', 'zh-TW'],
    ['zh-Hant-TW', 'zh-TW'],
    ['zh-Hant-HK', 'zh-HK'],
    ['zh_HK.UTF-8', 'zh-HK'],
    ['en-AU', 'en'],
    ['fr-FR', 'en'],
    ['zh-Hant', 'en'],
    ['zh-MO', 'en'],
    ['zh', 'en'],
    ['C', 'en'],
    ['POSIX', 'en'],
  ] as const)('%s → %s', (input, expected) =>
    expect(matchLocale(input)).toBe(expected),
  );
  test('honors precedence and falls back after choosing the locale', () => {
    expect(detectLocale({ explicit: 'fr', saved: 'zh-TW', env: {} })).toBe(
      'en',
    );
    expect(detectLocale({ saved: 'zh-HK', env: { LC_ALL: 'zh_CN' } })).toBe(
      'zh-HK',
    );
    expect(detectLocale({ env: { LC_ALL: 'C', LANG: 'zh_CN' } })).toBe('en');
    expect(detectLocale({ env: {}, system: 'zh-TW' })).toBe('zh-TW');
  });
});

test('all locales have every key and identical interpolation parameters', () => {
  for (const locale of locales) {
    expect(Object.keys(locale.messages).sort()).toEqual(
      [...messageKeys].sort(),
    );
    for (const key of messageKeys) {
      expect(locale.messages[key].match(/\{\w+\}/g)).toEqual(
        locales[0]!.messages[key].match(/\{\w+\}/g),
      );
    }
  }
});

test('Taiwan and Hong Kong use independent vocabulary and interpolate literally', () => {
  expect(new I18n('zh-TW').t('createBlog')).toBe('建立部落格');
  expect(new I18n('zh-HK').t('createBlog')).toBe('建立網誌');
  expect(new I18n('en').t('createdAt', { path: '$&/my-blog' })).toBe(
    'Created: $&/my-blog',
  );
});
