import { chmod, rm } from 'node:fs/promises';
import { resolve } from 'node:path';

const root = resolve(import.meta.dir, '..');
await rm(resolve(root, 'dist'), { recursive: true, force: true });
const result = await Bun.build({
  entrypoints: [resolve(root, 'src/index.ts')],
  outdir: resolve(root, 'dist'),
  target: 'bun',
  packages: 'external',
  splitting: true,
  sourcemap: 'external',
});
if (!result.success) throw new AggregateError(result.logs, 'Build failed');
await chmod(resolve(root, 'dist/index.js'), 0o755);
