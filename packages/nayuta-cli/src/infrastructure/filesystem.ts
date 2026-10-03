import { lstat, open, realpath, rename, rm, stat } from 'node:fs/promises';
import { dirname, isAbsolute, relative, resolve, sep } from 'node:path';
import { NayutaError } from '../core/errors';
export async function exists(path: string): Promise<boolean> {
  try {
    await lstat(path);
    return true;
  } catch (error) {
    if ((error as NodeJS.ErrnoException).code === 'ENOENT') return false;
    throw error;
  }
}
export async function assertInside(root: string, path: string): Promise<void> {
  const inside = (base: string, target: string) => {
    const part = relative(base, target);
    return !isAbsolute(part) && part !== '..' && !part.startsWith(`..${sep}`);
  };
  if (!inside(resolve(root), resolve(path)))
    throw new NayutaError('errorPath', { path });
  let ancestor = resolve(path);
  while (!(await exists(ancestor))) ancestor = dirname(ancestor);
  if (!inside(await realpath(root), await realpath(ancestor)))
    throw new NayutaError('errorPath', { path });
}
export async function atomicWrite(
  path: string,
  content: string,
  expected: string,
): Promise<void> {
  const lockPath = `${path}.nayuta-lock`;
  let lock;
  try {
    lock = await open(lockPath, 'wx', 0o600);
  } catch (error) {
    if ((error as NodeJS.ErrnoException).code === 'EEXIST')
      throw new NayutaError('errorBusy', { path });
    throw error;
  }
  const temporary = `${path}.nayuta-${crypto.randomUUID()}`;
  try {
    if ((await Bun.file(path).text()) !== expected)
      throw new NayutaError('errorChanged', { path });
    const file = await open(temporary, 'wx', (await stat(path)).mode);
    try {
      await file.writeFile(content);
    } finally {
      await file.close();
    }
    if ((await Bun.file(path).text()) !== expected)
      throw new NayutaError('errorChanged', { path });
    await rename(temporary, path);
  } finally {
    await lock.close();
    await rm(lockPath, { force: true });
    await rm(temporary, { force: true });
  }
}
