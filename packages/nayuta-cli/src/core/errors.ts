import type { MessageKey, MessageParams } from '../i18n';
import type { I18n } from '../i18n';
import { messageKeys } from '../i18n/types';

export class NayutaError extends Error {
  constructor(
    public code: MessageKey,
    public params: MessageParams = {},
  ) {
    super(code);
    this.name = 'NayutaError';
  }
}

export function describeError(error: unknown, i18n: I18n): string {
  if (error instanceof NayutaError) {
    const aliases: Record<string, MessageKey> = {
      site_url: 'siteUrl',
      exclude_in_search: 'excludeInSearch',
      'copyright.enabled': 'copyrightEnabled',
      'copyright.license': 'copyrightLicense',
    };
    const params = { ...error.params };
    for (const key of ['field', 'detail']) {
      const value = String(params[key] ?? '');
      const message = aliases[value] ?? value;
      if (messageKeys.includes(message as MessageKey))
        params[key] = i18n.t(message as MessageKey);
    }
    return i18n.t(error.code, params);
  }
  return i18n.t('errorUnexpected', {
    detail: error instanceof Error ? error.message : String(error),
  });
}
