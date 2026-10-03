import { copyFile, mkdir, rm, stat } from 'node:fs/promises';
import { extname, join, relative, resolve } from 'node:path';
import { z } from 'zod';
import type { I18n } from '../../i18n';
import { discoverProject } from '../project/discover';
import { date, today, validate } from '../blog/schema';
import { NayutaError } from '../errors';
import {
  assertInside,
  atomicWrite,
  exists,
  readText,
} from '../../infrastructure/filesystem';
import { readSiteConfig } from '../../infrastructure/site-config';
import { renderPost } from '../../templates/post';
import { postSchema } from './schema';
import { isPostPath, postId, postSlug } from './path';
import {
  parseFrontmatter,
  prepareFrontmatter,
  type FrontmatterSnapshot,
} from './frontmatter';

export interface PostEntry {
  path: string;
  id: string;
  title: string;
  publishDate: string;
  draft: boolean;
}
export async function listPosts(directory: string): Promise<PostEntry[]> {
  const root = await discoverProject(directory);
  const postsRoot = join(root, 'src/content/posts');
  if (!(await exists(postsRoot))) return [];
  const posts: PostEntry[] = [];
  for await (const file of new Bun.Glob('**/*.{md,mdx}').scan({
    cwd: postsRoot,
    onlyFiles: true,
    followSymlinks: false,
  })) {
    if (!isPostPath(file)) continue;
    const path = join(postsRoot, file);
    await assertInside(root, path);
    const snapshot = parseFrontmatter(await readText(path), path);
    posts.push({
      path,
      id: postId(file, snapshot.values),
      title: snapshot.values.title,
      publishDate: snapshot.values.publishDate,
      draft: snapshot.values.draft ?? false,
    });
  }
  return posts.sort(
    (a, b) =>
      b.publishDate.localeCompare(a.publishDate) || a.id.localeCompare(b.id),
  );
}
export interface CreatePostOptions {
  directory: string;
  title: string;
  i18n: I18n;
  slug?: string;
  format?: 'md' | 'mdx';
  publishDate?: string;
  description?: string;
  tags?: string[];
  published?: boolean;
  datePrefix?: boolean;
  cover?: string;
  assetBase?: string;
}
export async function createPost(options: CreatePostOptions) {
  const root = await discoverProject(options.directory);
  const publication = validate(date, options.publishDate ?? today());
  const slug = postSlug(
    options.slug ?? options.title,
    options.slug !== undefined,
  );
  const folderName = `${options.datePrefix === false ? '' : `${publication}-`}${slug}`;
  const postsRoot = join(root, 'src/content/posts');
  const folder = join(postsRoot, folderName);
  await assertInside(root, folder);
  const format = validate(z.enum(['md', 'mdx']), options.format ?? 'md');
  const path = join(folder, `index.${format}`);
  const id = postId(relative(postsRoot, path));
  if (
    (await exists(folder)) ||
    (await listPosts(root)).some((post) => post.id === id)
  )
    throw new NayutaError('errorExists', { path: folder });
  let localCover: string | undefined;
  let cover: string | undefined;
  if (options.cover) {
    if (/^https?:\/\//i.test(options.cover))
      cover = validate(z.url({ protocol: /^https?$/ }), options.cover);
    else {
      const candidate = resolve(
        options.assetBase ?? options.directory,
        options.cover,
      );
      if ((await exists(candidate)) && (await stat(candidate)).isFile()) {
        localCover = candidate;
        if (!/\.(avif|gif|jpe?g|png|svg|webp)$/i.test(candidate))
          throw new NayutaError('errorCover', { path: candidate });
        cover = `./assets/cover${extname(candidate).toLowerCase()}`;
      } else if (options.cover.startsWith('/')) {
        const publicPath = join(root, 'public', options.cover);
        await assertInside(root, publicPath);
        if (!(await exists(publicPath)))
          throw new NayutaError('errorCover', { path: options.cover });
        cover = options.cover;
      } else throw new NayutaError('errorCover', { path: candidate });
    }
  }
  const data = validate(postSchema, {
    title: options.title,
    publishDate: publication,
    description: options.description,
    tags: options.tags
      ? [...new Set(options.tags.map((tag) => tag.trim()))]
      : [],
    draft: !options.published,
    cover,
  });
  await mkdir(postsRoot, { recursive: true });
  try {
    await mkdir(folder);
  } catch (error) {
    if ((error as NodeJS.ErrnoException).code === 'EEXIST')
      throw new NayutaError('errorExists', { path: folder });
    throw error;
  }
  try {
    await mkdir(join(folder, 'assets'));
    await Bun.write(join(folder, 'assets/.gitkeep'), '');
    if (localCover && cover) await copyFile(localCover, join(folder, cover));
    await Bun.write(
      path,
      renderPost(
        Object.fromEntries(
          Object.entries(data).filter(([, value]) => value !== undefined),
        ) as typeof data,
        options.i18n,
      ),
    );
  } catch (error) {
    await rm(folder, { recursive: true, force: true });
    throw error;
  }
  return { path, id, url: `/posts/${id}`, draft: !options.published };
}
export async function readPost(
  directory: string,
  path: string,
): Promise<FrontmatterSnapshot> {
  const root = await discoverProject(directory);
  const absolute = resolve(root, path);
  const postsRoot = join(root, 'src/content/posts');
  await assertInside(postsRoot, absolute);
  if (!isPostPath(relative(postsRoot, absolute)) || !(await exists(absolute)))
    throw new NayutaError('errorPost', { path });
  return parseFrontmatter(await readText(absolute), absolute);
}
export async function validatePostContent(
  root: string,
  path: string,
  content: string,
): Promise<void> {
  const { values } = parseFrontmatter(content, path);
  if (values.cover && !/^https?:\/\//i.test(values.cover)) {
    const cover = values.cover.startsWith('/')
      ? join(root, 'public', values.cover)
      : resolve(path, '..', values.cover);
    await assertInside(root, cover);
    if (!(await exists(cover)))
      throw new NayutaError('errorCover', { path: values.cover });
  }
  const config = await readSiteConfig(root);
  const enabled = values.copyright?.enabled ?? config.values.copyright?.enabled;
  const license = values.copyright?.license ?? config.values.copyright?.license;
  if (enabled && !license)
    throw new NayutaError('errorValidation', {
      field: 'copyright.license',
      detail: 'Required when enabled',
    });
}
export async function updatePost(
  directory: string,
  snapshot: FrontmatterSnapshot,
  patch: Record<string, unknown>,
) {
  const root = await discoverProject(directory);
  await assertInside(join(root, 'src/content/posts'), snapshot.path);
  const content = prepareFrontmatter(snapshot, patch);
  await validatePostContent(root, snapshot.path, content);
  await atomicWrite(snapshot.path, content, snapshot.source);
  return { path: snapshot.path };
}
