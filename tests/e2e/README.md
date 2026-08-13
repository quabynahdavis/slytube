# E2E Tests

End-to-end tests using WebdriverIO + `@wdio/tauri-service` connected to the Tauri app via `tauri-driver`.

## Prerequisites

1. **Build the Tauri app** (required before running E2E):

   ```bash
   bunx tauri build             # release build (recommended)
   bunx tauri build --debug     # faster build, slower runtime
   ```

2. **Install tauri-driver** (one-time):

   ```bash
   cargo install tauri-driver
   ```

3. **Linux only**: ensure a display server is running (or use xvfb):

   ```bash
   xvfb-run bun run test:e2e
   ```

## Running

```bash
bun run test:e2e              # headless
bun run test:e2e:debug         # verbose logging
```

## How It Works

- `wdio.conf.ts` — WebdriverIO configuration with the `tauri` service.
- The `tauri` service launches `tauri-driver`, which proxies WebDriver commands to the Tauri app.
- On Linux this uses `WebKitWebDriver` under the hood.
- `tests/e2e/**/*.spec.ts` — Mocha-style test files driven by the global `browser` instance.

## Writing Tests

```ts
describe('My Feature', () => {
  it('does something', async () => {
    const el = await browser.$('#some-element');
    await el.click();
    await expect(el).toHaveText('expected');
  });
});
```

The global `browser` is a WebdriverIO instance attached to the running Tauri window.

## Notes

- Tests run against the **built binary**, not the dev server.
- Network calls to Invidious/YouTube hit live servers — mock them for deterministic CI.
- The `tauri` service auto-detects the binary (release → debug fallback).
