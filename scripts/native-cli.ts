import { spawn } from 'node:child_process';
import { join } from 'node:path';
import { root, run } from './distribution';

export async function buildBinary(
  options: { release?: boolean; target?: string } = {},
): Promise<string> {
  const output = await run([
    'cargo',
    '+1.95.0',
    'build',
    '--locked',
    '--manifest-path',
    join(root, 'packages/nayuta-cli/Cargo.toml'),
    '--message-format=json-render-diagnostics',
    ...(options.release ? ['--release'] : []),
    ...(options.target ? ['--target', options.target] : []),
  ]);
  for (const line of output.split('\n')) {
    const event = JSON.parse(line) as {
      reason: string;
      target?: { name: string };
      executable?: string;
    };
    if (
      event.reason === 'compiler-artifact' &&
      event.target?.name === 'nayuta' &&
      event.executable
    ) {
      return event.executable;
    }
  }
  throw new Error('Cargo did not produce the nayuta executable.');
}

if (import.meta.main) {
  try {
    const binary = await buildBinary();
    const child = spawn(binary, process.argv.slice(2), { stdio: 'inherit' });
    const signals = ['SIGINT', 'SIGTERM', 'SIGHUP'] as const;
    const handlers = signals.map((signal) => {
      const handler = () => child.kill(signal);
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
    console.error(error instanceof Error ? error.message : error);
    process.exitCode = 1;
  }
}
