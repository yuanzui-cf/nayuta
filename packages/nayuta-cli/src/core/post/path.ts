import { slug as githubSlug } from 'github-slugger';
import { NayutaError } from '../errors';

export function postSlug(value: string, explicit = false): string {
  if (explicit && !/^[\p{L}\p{N}]+(?:-[\p{L}\p{N}]+)*$/u.test(value))
    throw new NayutaError('errorPath', { path: value });
  const slug = githubSlug(value.trim())
    .replace(/-+/g, '-')
    .replace(/^-|-$/g, '');
  if (!/^[\p{L}\p{N}]+(?:-[\p{L}\p{N}]+)*$/u.test(slug) || slug === 'assets')
    throw new NayutaError('errorPath', { path: value });
  return slug;
}
export function postId(
  path: string,
  data: Record<string, unknown> = {},
): string {
  if (data.slug) return String(data.slug);
  return path
    .replaceAll('\\', '/')
    .replace(/\.mdx?$/, '')
    .split('/')
    .map((part) => githubSlug(part))
    .join('/')
    .replace(/\/index$/, '');
}
export function isPostPath(path: string): boolean {
  return (
    path
      .replaceAll('\\', '/')
      .split('/')
      .every((part) => part !== 'assets' && !part.startsWith('_')) &&
    /\.mdx?$/.test(path)
  );
}
