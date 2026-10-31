#!/usr/bin/env node
import { spawn, spawnSync } from 'node:child_process';
import { createRequire } from 'node:module';
import { dirname, join } from 'node:path';
import pkg from '../package.json';
import { platformPackage } from './platform';

try {
  const name = platformPackage(pkg.name, process.platform, process.arch);
  let binary = process.env.NAYUTA_BINARY;
  if (!binary) {
    const require = createRequire(import.meta.url);
    try {
      binary = join(
        dirname(require.resolve(`${name}/package.json`)),
        process.platform === 'win32' ? 'nayuta.exe' : 'nayuta',
      );
    } catch {
      throw new Error(
        `Missing native dependency ${name} for ${pkg.name}@${pkg.version}. Reinstall with optional dependencies enabled, or download a native binary from GitHub Releases.`,
      );
    }
  }
  const version = spawnSync(binary, ['--version'], {
    encoding: 'utf8',
    timeout: 10000,
    windowsHide: true,
  });
  if (version.error) throw version.error;
  if (
    version.status !== 0 ||
    version.stdout.trim() !== `nayuta ${pkg.version}`
  ) {
    throw new Error(
      `Binary version does not match ${pkg.name}@${pkg.version}. Reinstall the matching platform package.`,
    );
  }
  const child = spawn(binary, process.argv.slice(2), {
    stdio: 'inherit',
    windowsHide: false,
  });
  const signals = ['SIGINT', 'SIGTERM', 'SIGHUP'] as const;
  const handlers = signals.map((signal) => {
    const handler = () => {
      child.kill(signal);
    };
    process.on(signal, handler);
    return handler;
  });
  const cleanup = () =>
    signals.forEach((signal, i) =>
      process.removeListener(signal, handlers[i]!),
    );
  child.once('error', (error) => {
    cleanup();
    console.error(`nayuta: ${error.message}`);
    process.exitCode = 1;
  });
  child.once('exit', (code, signal) => {
    cleanup();
    if (signal && process.platform !== 'win32')
      process.kill(process.pid, signal);
    else process.exitCode = code ?? 1;
  });
} catch (error) {
  console.error(`nayuta: ${error instanceof Error ? error.message : error}`);
  process.exitCode = 1;
}
