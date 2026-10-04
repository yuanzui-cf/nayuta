import { root } from './distribution';
import { readVersions, versionChanges, writeVersions } from './versions';

const files = await readVersions(root);
const version = files[0]!.version;
const tag = (
  process.env.NAYUTA_RELEASE_TAG ??
  (process.env.GITHUB_REF?.startsWith('refs/tags/')
    ? process.env.GITHUB_REF.slice(10)
    : undefined)
)?.replace(/^v/, '');
if (tag && tag !== version)
  throw new Error(`Tag ${tag} differs from theme ${version}`);
if (process.argv.includes('--write')) {
  await writeVersions(root, versionChanges(files, version));
} else if (files.some((file) => file.version !== version)) {
  throw new Error(
    `Versions differ: ${files.map((file) => `${file.path}: ${file.version}`).join(', ')}. Run bun run cli:version:sync.`,
  );
}
console.log(`Nayuta theme, Rust and npm versions: ${version}`);
