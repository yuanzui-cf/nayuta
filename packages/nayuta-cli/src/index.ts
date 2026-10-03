#!/usr/bin/env bun
import { createContext } from './app/context';
import { runCli } from './app/cli';

const args = process.argv.slice(2);
await runCli(
  !args.length &&
    process.stdin.isTTY &&
    process.stdout.isTTY &&
    process.env.TERM !== 'dumb'
    ? ['tui']
    : args,
  await createContext(args),
);
