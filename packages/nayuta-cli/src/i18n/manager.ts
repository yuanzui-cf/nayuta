import { locales } from './register';
import type { LocaleId, MessageKey, MessageParams } from './types';

export class I18n {
  constructor(public locale: LocaleId) {}

  t(key: MessageKey, params: MessageParams = {}): string {
    const language = locales.find((locale) => locale.id === this.locale)!;
    const message = language.messages[key] ?? locales[0]!.messages[key];
    return message.replace(/\{(\w+)\}/g, (placeholder, name: string) =>
      params[name] === undefined ? placeholder : String(params[name]),
    );
  }
}
