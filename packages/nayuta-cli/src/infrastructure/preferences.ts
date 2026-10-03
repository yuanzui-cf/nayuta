import { mkdir } from 'node:fs/promises';
import { homedir } from 'node:os';
import { dirname, join } from 'node:path';
import { matchLocale, type LocaleId } from '../i18n';
export function preferencesPath(): string {
  const base =
    process.platform === 'win32'
      ? (process.env.APPDATA ?? join(homedir(), 'AppData', 'Roaming'))
      : (process.env.XDG_CONFIG_HOME ?? join(homedir(), '.config'));
  return join(base, 'nayuta', 'config.json');
}
export async function readPreferences(): Promise<{ locale?: string }> {
  try {
    const data: unknown = await Bun.file(preferencesPath()).json();
    if (
      data &&
      typeof data === 'object' &&
      'locale' in data &&
      typeof data.locale === 'string'
    )
      return { locale: data.locale };
  } catch {
    /* Missing or malformed preferences use system defaults. */
  }
  return {};
}
export async function saveLocale(locale: LocaleId): Promise<void> {
  const path = preferencesPath();
  await mkdir(dirname(path), { recursive: true });
  await Bun.write(
    path,
    `${JSON.stringify({ locale: matchLocale(locale) }, null, 2)}\n`,
  );
}
