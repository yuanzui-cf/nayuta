import { resolve } from 'node:path';
import { detectLocale, I18n } from '../i18n';
import { readPreferences } from '../infrastructure/preferences';
export interface AppContext {
  cwd: string;
  i18n: I18n;
  json: boolean;
}
export async function createContext(args: string[]): Promise<AppContext> {
  function option(name: string) {
    const index = args.indexOf(name);
    return index < 0
      ? args
          .find((argument) => argument.startsWith(`${name}=`))
          ?.slice(name.length + 1)
      : args[index + 1];
  }
  const preferences = await readPreferences();
  return {
    cwd: resolve(option('--cwd') ?? process.cwd()),
    i18n: new I18n(
      detectLocale({ explicit: option('--locale'), saved: preferences.locale }),
    ),
    json: args.includes('--json'),
  };
}
