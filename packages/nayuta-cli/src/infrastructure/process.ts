import { NayutaError } from '../core/errors';
export interface ProcessOptions {
  cwd?: string;
  env?: Record<string, string | undefined>;
  allowFailure?: boolean;
}
export async function run(
  args: readonly string[],
  options: ProcessOptions = {},
) {
  const child = Bun.spawn([...args], {
    cwd: options.cwd,
    env: options.env ?? process.env,
    stdin: 'ignore',
    stdout: 'pipe',
    stderr: 'pipe',
  });
  const [exitCode, stdout, stderr] = await Promise.all([
    child.exited,
    new Response(child.stdout).text(),
    new Response(child.stderr).text(),
  ]);
  if (exitCode !== 0 && !options.allowFailure)
    throw new NayutaError('errorProcess', {
      command: args[0]!,
      detail: stderr.trim() || stdout.trim() || String(exitCode),
    });
  return { exitCode, stdout: stdout.trim(), stderr: stderr.trim() };
}
