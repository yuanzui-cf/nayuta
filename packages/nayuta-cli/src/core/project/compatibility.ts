import { NayutaError } from '../errors';
export function checkCompatibility(version: unknown): void {
  if (typeof version !== 'string' || !/^0\.2\.\d+(?:[-+].*)?$/.test(version))
    throw new NayutaError('errorVersion', { version: String(version) });
}
