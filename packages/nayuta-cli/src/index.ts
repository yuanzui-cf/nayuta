#!/usr/bin/env bun
import { createContext } from './app/context';
import { runCli } from './app/cli';

const args = process.argv.slice(2);
await runCli(args, await createContext(args));
