import { createTwoFilesPatch } from 'diff';
export function textDiff(path: string, before: string, after: string): string {
  return createTwoFilesPatch(path, path, before, after, undefined, undefined, {
    context: 3,
  });
}
