import { existsSync } from 'node:fs';
import { resolve, dirname } from 'node:path';
import { fileURLToPath } from 'node:url';

const __dirname = dirname(fileURLToPath(import.meta.url));
const PROJECT_ROOT = resolve(__dirname, '..');
const TAURI_BIN = resolve(PROJECT_ROOT, 'src-tauri/target/release/slytube');
const DEV_BIN = resolve(PROJECT_ROOT, 'src-tauri/target/debug/slytube');
const TAURI_DRIVER_PORT = 4444;

function getTauriBinary(): string {
  if (existsSync(TAURI_BIN)) return TAURI_BIN;
  if (existsSync(DEV_BIN)) return DEV_BIN;
  throw new Error(
    `Tauri binary not found.\n` +
      `  Release: ${TAURI_BIN}\n` +
      `  Debug:   ${DEV_BIN}\n` +
      'Run `bunx tauri build` (or `bunx tauri build --debug`) first.'
  );
}

export const config: WebdriverIO.Config = {
  runner: 'local',
  specs: ['./tests/e2e/**/*.spec.ts'],
  maxInstances: 1,
  capabilities: [
    {
      browserName: 'webkit',
      'tauri:options': {
        binary: getTauriBinary(),
      },
    },
  ],
  logLevel: 'warn',
  bail: 0,
  waitforTimeout: 10_000,
  connectionRetryTimeout: 30_000,
  connectionRetryCount: 3,
  services: [
    [
      'tauri',
      {
        port: TAURI_DRIVER_PORT,
        host: '127.0.0.1',
        command: 'tauri-driver',
      },
    ],
  ],
  framework: 'mocha',
  reporters: ['spec'],
  mochaOpts: {
    ui: 'bdd',
    timeout: 60_000,
  },
};
