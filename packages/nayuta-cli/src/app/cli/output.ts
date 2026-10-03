import type { AppContext } from '../context';
import { describeError, NayutaError } from '../../core/errors';
export function output(
  context: AppContext,
  result: { path: string },
  created = false,
): void {
  console.log(
    context.json
      ? JSON.stringify(result)
      : context.i18n.t(created ? 'createdAt' : 'updatedAt', {
          path: result.path,
        }),
  );
}
export function fail(context: AppContext, error: unknown): void {
  const message = describeError(error, context.i18n);
  const record = {
    error: error instanceof NayutaError ? error.code : 'errorUnexpected',
    message,
  };
  if (context.json) console.log(JSON.stringify(record));
  else console.error(message);
  process.exitCode = 1;
}
