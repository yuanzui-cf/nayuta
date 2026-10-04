import { dirname, join } from 'node:path';
import { platforms, root, run } from './distribution';
import { buildBinary } from './native-cli';

const args = process.argv.slice(2);
if (args.length === 1 && ['--help', '-h'].includes(args[0]!)) {
  console.log(
    'Usage: bun run cli:release [Rust target]\n' +
      'Default: current OS/architecture. Builds the native binary, npm packages, template and checksums.\n' +
      'Targets: ' +
      platforms.map((platform) => platform.target).join(', '),
  );
} else {
  try {
    if (args.length > 1) throw new Error('Supply at most one Rust target.');
    const platform = args[0]
      ? platforms.find((platform) => platform.target === args[0])
      : platforms.find(
          (platform) =>
            platform.platform === process.platform &&
            platform.arch === process.arch,
        );
    if (!platform) throw new Error('Unsupported platform or Rust target.');
    console.log(
      await run([
        process.execPath,
        join(root, 'scripts/sync-version.ts'),
        '--check',
      ]),
    );
    const binary = await buildBinary({
      release: true,
      target: platform.target,
    });
    process.env.NAYUTA_BUILD_DIR = dirname(dirname(dirname(binary)));
    for (const script of [
      'package-cli.ts',
      'archive-cli.ts',
      'package-template.ts',
    ]) {
      const command = [process.execPath, join(root, 'scripts', script)];
      if (script !== 'package-template.ts') command.push(platform.target);
      console.log(await run(command));
    }
    console.log(`Release artifacts: ${join(root, 'dist/cli')}`);
  } catch (error) {
    console.error(error instanceof Error ? error.message : error);
    process.exitCode = 1;
  }
}
