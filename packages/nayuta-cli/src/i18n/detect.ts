import { locales } from './register';
import type { LocaleId } from './types';

export function matchLocale(value: string): LocaleId {
  const normalized = value
    .trim()
    .split(/[.@]/)[0]!
    .replaceAll('_', '-')
    .toLowerCase();
  if (/^en(?:-|$)/.test(normalized)) return 'en';
  return (
    locales.find((locale) => locale.aliases.includes(normalized))?.id ?? 'en'
  );
}

export function detectLocale(
  options: {
    explicit?: string;
    saved?: string;
    env?: Record<string, string | undefined>;
    system?: string;
  } = {},
): LocaleId {
  const env = options.env ?? process.env;
  return matchLocale(
    options.explicit ||
      options.saved ||
      env.LC_ALL ||
      env.LC_MESSAGES ||
      env.LANG ||
      options.system ||
      Intl.DateTimeFormat().resolvedOptions().locale,
  );
}
