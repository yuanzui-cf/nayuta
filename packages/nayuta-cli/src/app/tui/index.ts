import { createCliRenderer } from '@opentui/core';
import type { AppContext } from '../context';
import { NayutaError } from '../../core/errors';
import { TuiApp } from './app';
export async function runTui(context: AppContext): Promise<void> {
  if (
    !process.stdin.isTTY ||
    !process.stdout.isTTY ||
    process.env.TERM === 'dumb'
  )
    throw new NayutaError('errorTerminal');
  let finish!: () => void;
  const closed = new Promise<void>((resolve) => {
    finish = resolve;
  });
  const renderer = await createCliRenderer({
    exitOnCtrlC: false,
    screenMode: 'alternate-screen',
    consoleMode: 'disabled',
    onDestroy: finish,
  });
  const app = new TuiApp(renderer, context);
  try {
    app.start();
    await closed;
  } finally {
    app.destroy();
  }
}
