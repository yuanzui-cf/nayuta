import { describe, expect, test } from 'bun:test';
import { mkdtemp, rm, writeFile, chmod } from 'node:fs/promises';
import { tmpdir } from 'node:os';
import { join, resolve } from 'node:path';
import { platformPackage } from '../src/platform';
import pkg from '../package.json';

const launcher = resolve(import.meta.dir, '../src/cli.ts');
describe('native launcher', () => {
  test('all supported platforms and unsupported architectures', () => {
    for (const platform of ['darwin', 'linux', 'win32'])
      for (const arch of ['x64', 'arm64']) {
        expect(platformPackage('@scope/nayuta', platform, arch)).toBe(
          `@scope/nayuta-${platform}-${arch}`,
        );
      }
    expect(() => platformPackage('@scope/nayuta', 'linux', 'ia32')).toThrow(
      'Unsupported platform',
    );
  });
  test('missing binaries produce actionable errors', async () => {
    const result = Bun.spawn([process.execPath, launcher], {
      env: { ...process.env, NAYUTA_BINARY: '/nonexistent/nayuta' },
      stderr: 'pipe',
      stdout: 'pipe',
    });
    expect(await result.exited).toBe(1);
    expect(await new Response(result.stderr).text()).toContain('nayuta:');
  });
  test.skipIf(process.platform === 'win32')(
    'spaces, literal shell arguments and exit status pass through',
    async () => {
      const folder = await mkdtemp(join(tmpdir(), 'nayuta launcher '));
      try {
        const binary = join(folder, 'native fixture');
        await writeFile(
          binary,
          `#!/usr/bin/env bun\nif(process.argv[2]==="--version")console.log("nayuta ${pkg.version}");else { console.log(JSON.stringify(process.argv.slice(2)));process.exit(7); }\n`,
        );
        await chmod(binary, 0o755);
        const args = ['a b', '$(never-execute)', '`literal`', '你好'];
        const result = Bun.spawn([process.execPath, launcher, ...args], {
          env: { ...process.env, NAYUTA_BINARY: binary },
          stdout: 'pipe',
          stderr: 'pipe',
        });
        expect(await result.exited).toBe(7);
        expect(JSON.parse(await new Response(result.stdout).text())).toEqual(
          args,
        );
        await writeFile(
          binary,
          '#!/usr/bin/env bun\nconsole.log("nayuta 999.0.0");\n',
        );
        const mismatch = Bun.spawn([process.execPath, launcher], {
          env: { ...process.env, NAYUTA_BINARY: binary },
          stderr: 'pipe',
        });
        expect(await mismatch.exited).toBe(1);
        expect(await new Response(mismatch.stderr).text()).toContain(
          'does not match',
        );
      } finally {
        await rm(folder, { recursive: true, force: true });
      }
    },
  );
});
