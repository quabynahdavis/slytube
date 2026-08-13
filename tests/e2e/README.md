# E2E Tests

End-to-end tests using Playwright connected to the Tauri app via Chrome DevTools Protocol (CDP).

## Prerequisites

1. **Build the Tauri app** (required before running E2E):

   ```bash
   bunx tauri build             # release build (recommended)
   bunx tauri build --debug     # faster build, slower runtime
   ```

2. **Install Playwright browsers** (one-time):

   ```bash
   bunx playwright install chromium
   ```

3. **Linux only**: ensure a display server is running (or use xvfb):

   ```bash
   xvfb-run bun run test:e2e
   ```

## Running

```bash
bun run test:e2e              # headless
bun run test:e2e:headed        # with browser window
bun run test:e2e:debug         # step-through debugger
```

## How It Works

- `playwright.config.ts` — Playwright configuration (test dir, timeouts, reporter).
- `tests/e2e/fixtures.ts` — Launches the built Tauri binary with `TAURI_WEBVIEW_AUTOMATION=true`,
  captures the CDP endpoint from stdout, and connects Playwright via `chromium.connectOverCDP`.
- `tests/e2e/*.spec.ts` — Test scenarios.

## Writing Tests

```ts
import { test, expect } from './fixtures';

test('my scenario', async ({ appPage }) => {
  await appPage.waitForLoadState('domcontentloaded');
  // ... interact with the app
});
```

The `appPage` fixture gives you a Playwright `Page` attached to the running Tauri window.

## Notes

- Tests run against the **built binary**, not the dev server.
- Network calls to Invidious/YouTube will hit live servers unless mocked.
- For CI, use `xvfb-run` on Linux and mock network responses.
