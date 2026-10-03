import { dirname, join, resolve } from 'node:path';
import { checkCompatibility } from './compatibility';
import { NayutaError } from '../errors';
import { assertInside, exists } from '../../infrastructure/filesystem';
export async function discoverProject(start: string): Promise<string> {
  let directory = resolve(start);
  while (true) {
    if (
      (await exists(join(directory, 'src/config.ts'))) &&
      (await exists(join(directory, 'src/content.config.ts')))
    ) {
      let metadata;
      try {
        metadata = await Bun.file(join(directory, 'package.json')).json();
      } catch {
        throw new NayutaError('errorProject', { path: directory });
      }
      if (metadata.name !== 'nayuta-theme')
        throw new NayutaError('errorProject', { path: directory });
      checkCompatibility(metadata.version);
      await assertInside(directory, join(directory, 'src/config.ts'));
      await assertInside(directory, join(directory, 'src/content/posts'));
      return directory;
    }
    const parent = dirname(directory);
    if (parent === directory)
      throw new NayutaError('errorProject', { path: start });
    directory = parent;
  }
}
