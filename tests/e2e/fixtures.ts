import { test as base, type Page, type BrowserContext } from '@playwright/test';
import { spawn, type ChildProcess } from 'node:child_process';
import { existsSync } from 'node:fs';
import { resolve, dirname } from 'node:path';
import { fileURLToPath } from 'node:url';
import { chromium } from '@playwright/test';

const __dirname = dirname(fileURLToPath(import.meta.url));
const PROJECT_ROOT = resolve(__dirname, '../..');
const TAURI_BIN = resolve(
  PROJECT_ROOT,
  'src-tauri/target/release/slytube'
);
const DEV_BIN = resolve(
  PROJECT_ROOT,
  'src-tauri/target/debug/slytube'
);

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

let tauriProcess: ChildProcess | null = null;

export type Fixtures = {
  appPage: Page;
  appContext: BrowserContext;
};

export const test = base.extend<Fixtures>({
  appContext: async ({}, use) => {
    const bin = getTauriBinary();

    tauriProcess = spawn(bin, [], {
      cwd: PROJECT_ROOT,
      env: {
        ...process.env,
        TAURI_WEBVIEW_AUTOMATION: 'true',
        RUST_LOG: 'info',
      },
      stdio: ['ignore', 'pipe', 'pipe'],
    });

    let cdpUrl: string | null = null;
    const cdpPattern = /ws:\/\/(localhost|127\.0\.0\.1):(\d+)\/devtools/;

    const stdoutHandler = (d: Buffer) => {
      const line = d.toString();
      console.log(`[tauri] ${line.trim()}`);
      const m = line.match(cdpPattern);
      if (m) cdpUrl = m[0];
    };
    const stderrHandler = (d: Buffer) => {
      const line = d.toString();
      console.error(`[tauri:err] ${line.trim()}`);
      const m = line.match(cdpPattern);
      if (m) cdpUrl = m[0];
    };

    tauriProcess.stdout!.on('data', stdoutHandler);
    tauriProcess.stderr!.on('data', stderrHandler);

    // Wait for CDP endpoint to appear (up to 30s)
    const deadline = Date.now() + 30_000;
    while (!cdpUrl && Date.now() < deadline) {
      await new Promise((r) => setTimeout(r, 200));
    }

    if (!cdpUrl) {
      tauriProcess.kill('SIGKILL');
      throw new Error('Tauri did not expose a CDP endpoint within 30s.');
    }

    const browser = await chromium.connectOverCDP(cdpUrl);
    const context = browser.contexts()[0] ?? await browser.newContext();

    await use(context);

    await browser.close();
    if (tauriProcess) {
      tauriProcess.kill('SIGTERM');
      tauriProcess = null;
    }
  },

  appPage: async ({ appContext }, use) => {
    const page = appContext.pages()[0] ?? await appContext.newPage();
    await use(page);
  },
});

export { expect } from '@playwright/test';
