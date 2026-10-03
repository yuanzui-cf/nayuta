import type { MessageKey, MessageParams } from '../i18n';
import type { I18n } from '../i18n';

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
  return error instanceof NayutaError
    ? i18n.t(error.code, error.params)
    : i18n.t('errorUnexpected', {
        detail: error instanceof Error ? error.message : String(error),
      });
}
