# Testing Strategy

## Scope
This document defines the testing strategy for SlyTube, covering unit tests, component tests, end-to-end tests, and manual testing checklists. The strategy ensures correctness across the full stack: Rust backend, Vue frontend, and cross-platform system integration.

## File Index
- [Testing Strategy](testing.md) - This file

---

## 1. Test Pyramid

```
        ┌──────────────┐
        │   E2E Tests  │  ← Playwright (critical user journeys)
        ├──────────────┤
        │  Component   │  ← Vitest + Testing Library
        │    Tests     │
        ├──────────────┤
        │  Unit Tests  │  ← Vitest (frontend) + cargo test (Rust)
        └──────────────┘
```

| Layer | Tool | Scope | CI Gate |
|-------|------|-------|---------|
| Unit (Frontend) | Vitest | Stores, composables, utilities, formatters | PR |
| Unit (Rust) | `cargo test` | Repositories, crypto, parsing, validation | PR |
| Component | Vitest + Testing Library | Vue components, user interactions | PR |
| E2E | Playwright + tauri-driver | Full user journeys, system integration | Nightly |
| Manual | Checklist | Platform-specific UX, visual, accessibility | Release |

---

## 2. Unit Tests — Frontend (Vitest)

### 2.1 Setup

```bash
bun add -D vitest @vitest/coverage-v8 @vue/test-utils @testing-library/vue \
            @testing-library/user-event jsdom happy-dom @vitest/ui
```

**Configuration:** `vitest.config.ts`

```ts
export default defineConfig({
  plugins: [vue()],
  test: {
    environment: 'happy-dom',
    setupFiles: ['./tests/setup.ts'],
    coverage: {
      provider: 'v8',
      reporter: ['text', 'lcov', 'html'],
      thresholds: { lines: 70, functions: 70, branches: 60, statements: 70 },
      exclude: ['src/components/ui/**', 'src/lib/bindings.ts', '**/*.d.ts'],
    },
  },
  resolve: { alias: { '@': fileURLToPath(new URL('./src', import.meta.url)) } },
});
```

### 2.2 Tauri IPC Mocking

```ts
// tests/setup.ts
import { mockIPC, clearMocks } from '@tauri-apps/api/mocks';

beforeEach(() => {
  mockIPC((cmd, args) => {
    const handler = registry[cmd];
    if (!handler) throw new Error(`Unmocked command: ${cmd}`);
    return handler(args);
  });
});
afterEach(() => { clearMocks(); vi.restoreAllMocks(); });
```

### 2.3 Coverage Targets

| Target | Tests | Priority |
|--------|-------|----------|
| 13 Pinia stores | State transitions, optimistic update + rollback, error paths | P0 |
| `lib/api/*` wrappers | Argument shaping, error normalization, retry logic | P0 |
| Composables | `useInvoke` race cancellation, `useTauriEvent` unlisten, `useTheme` | P0 |
| Formatters | Duration, view counts, relative dates, file sizes, i18n plurals | P1 |
| Deep-link parser | Valid/invalid URL grammar | P1 |

---

## 3. Unit Tests — Rust (`cargo test`)

### 3.1 Test Organization

```
src-tauri/
├── src/
│   ├── db/
│   │   └── repo/
│   │       ├── mod.rs
│   │       ├── download.rs
│   │       └── download_tests.rs    ← inline #[cfg(test)] modules
│   ├── crypto/
│   │   ├── mod.rs
│   │   └── crypto_tests.rs
│   └── services/
│       └── ytdlp/
│           ├── mod.rs
│           └── parse_tests.rs
└── tests/
    ├── integration/
    │   ├── migration_tests.rs
    │   └── download_tests.rs
    └── fixtures/
        ├── seeded.db
        └── nedb_samples/
```

### 3.2 Coverage Targets

| Target | Approach | Coverage Goal |
|--------|----------|---------------|
| Repositories | In-memory SQLite pool per test | ≥80% |
| Crypto | KAT vectors + proptest round-trips | ≥90% |
| yt-dlp parsing | Fixture-based stdout parsing | ≥85% |
| Deep-link validation | Regex validation + fuzzing | ≥90% |
| Error mapping | Each `AppError` variant exercised | ≥80% |

### 3.3 Commands

```bash
# Run all tests
cargo test

# Run with coverage
cargo tarpaulin --out Html --out Lcov

# Run integration tests only
cargo test --features integration

# Run proptests with more iterations
PROPTEST_CASES=10000 cargo test proptest
```

### 3.4 Extractor (src/extractor/main.ts)

Tests verify parsing of all youtubei.js node types:
- `Video` nodes (watch page, search results)
- `GridVideo` nodes (channel pages, playlists)
- `LockupView` nodes (feed/home/search shelves) — VIDEO and SHORT content types
- `Movie` nodes (movie/PMV results)

Helper function tests cover:
- `parseDurationText` — "HH:MM:SS" → seconds
- `parseSubscriberCount` — "1.2M views" → 1200000
- `calculatePublishedDate` — "2 days ago" → ISO timestamp
- `extractNumberFromString` — "1,234,567" → 1234567

---

## 4. Component Tests (Vitest + Testing Library)

### 4.1 Target Components

| Component | Tests | Priority |
|-----------|-------|----------|
| `VideoCard` | Render, click, context menu | P0 |
| `DownloadRow` | Progress display, pause/resume/cancel actions | P0 |
| `PlayerControls` | Play/pause, seek, volume, fullscreen | P0 |
| `QualityPicker` | Format selection, PoToken-gated options | P1 |
| `AppShell` | Sidebar toggle, route navigation | P1 |
| 16 View smoke tests | Mount + primary interaction | P1 |

### 4.2 Example

```ts
// tests/components/VideoCard.test.ts
import { render, screen, fireEvent } from '@testing-library/vue';
import VideoCard from '@/components/video/VideoCard.vue';

describe('VideoCard', () => {
  it('renders title and channel', () => {
    render(VideoCard, { props: { video: mockVideo } });
    expect(screen.getByText(mockVideo.title)).toBeInTheDocument();
  });

  it('emits click event', async () => {
    const { emitted } = render(VideoCard, { props: { video: mockVideo } });
    await fireEvent.click(screen.getByRole('article'));
    expect(emitted().click).toBeTruthy();
  });
});
```

---

## 5. E2E Tests (Playwright)

### 5.1 Setup

```bash
bun add -D @playwright/test
```

Tauri v2 exposes a WebDriver surface via `tauri-driver`:
- **Windows:** `msedgedriver`
- **Linux:** `WebKitWebDriver`
- **macOS:** Unsupported — covered by manual scripts + unit/component tests

### 5.2 Fixture Strategy

| Fixture | Purpose |
|---------|---------|
| `seeded-10k.db` | Realistic dataset for performance-sensitive tests |
| `mock-network.ts` | Recorded HTTP responses (no live YouTube in CI) |
| `launchApp()` | Boots the built binary under tauri-driver |

### 5.3 E2E Scenario Matrix

| ID | Scenario | Platforms | Priority |
|----|----------|-----------|----------|
| E1 | First-run onboarding | Win, Linux, macOS (manual) | P0 |
| E2 | Legacy migration wizard | Win, Linux | P0 |
| E3 | Search → watch | Win, Linux, macOS (manual) | P0 |
| E4 | Subscribe / unsubscribe | Win, Linux | P0 |
| E5 | Playlist lifecycle | Win, Linux | P0 |
| E6 | Download lifecycle (with restart) | Win, Linux, macOS (manual) | P0 |
| E7 | Download failure + retry | Win, Linux | P1 |
| E8 | Settings persistence | Win, Linux | P1 |
| E9 | Proxy enforcement | Win, Linux | P1 |
| E10 | Privacy Strict mode | Win, Linux | P1 |
| E11 | Sync two-device | Win, Linux | P1 |
| E12 | Deep link (warm + cold) | Win, Linux, macOS (manual) | P1 |
| E13 | Tray + window lifecycle | Win, Linux | P1 |
| E14 | Updater flow | Win, Linux | P2 |
| E15 | PoToken degradation | Win, Linux | P2 |
| E16 | Leak guard | Win, Linux | P2 |

### 5.4 Flake Policy

Any test failing >2% over 50 runs is **quarantined and fixed** within the phase — never `retry` masked.

---

## 6. Manual Testing Checklist

### 6.1 Per-Platform Checklist

| Check | Windows | macOS | Linux |
--|-------|-------|-------|-------|
| Fresh install from installer | ☐ | ☐ | ☐ |
| Upgrade install preserves data | ☐ | ☐ | ☐ |
| Uninstall prompts for data removal | ☐ | ☐ | ☐ |
| yt-dlp + ffmpeg sidecars execute | ☐ | ☐ | ☐ |
| Real download completes | ☐ | ☐ | ☐ |
| PoToken status recorded | ☐ | ☐ | ☐ |
| Tray functional | ☐ | ☐ | ☐ |
| Global shortcuts work | ☐ | ☐ | ☐ |
| Deep links route correctly | ☐ | ☐ | ☐ |
| File associations open app | ☐ | ☐ | ☐ |
| Updater end-to-end | ☐ | ☐ | ☐ (AppImage) |
| HiDPI scaling correct | ☐ | ☐ | ☐ |
| Dark/light follows OS | ☐ | ☐ | ☐ |
| Multi-monitor window restore | ☐ | ☐ | ☐ |
| Non-ASCII username paths | ☐ | ☐ | ☐ |

### 6.2 Platform Matrix

| Platform | Version(s) | Arch | Priority |
|----------|-----------|------|----------|
| Windows | 10 22H2, 11 23H2 | x64 | P0 |
| macOS | 13 Ventura, 14 Sonoma, 15 | x64 + arm64 | P0 |
| Ubuntu | 22.04 LTS, 24.04 LTS | x64 | P0 |
| Fedora | 40 | x64 | P1 |
| Arch (rolling) | current | x64 | P1 |
| Debian | 12 | x64 | P2 |
| Windows | 11 | arm64 | P2 |

### 6.3 Accessibility Checklist

| Check | Tool/Method |
|----|-------------|
| axe-core scan: zero critical/serious | Automated |
| Full keyboard-only walkthrough | Manual |
| Screen-reader spot check | NVDA / VoiceOver / Orca |
| Contrast ≥4.5:1 both themes | Automated + manual |
| Focus order logical, focus visible | Manual |
| `prefers-reduced-motion` honored | Manual |

### 6.4 Security Checklist

| Check | Method |
|-------|--------|
| Capability audit: every permission justified | Review |
| CSP verified in production build | Automated |
| No secrets in logs/crash reports | Review |
| `cargo audit` + `bun audit` clean | CI |
| Deep-link fuzzing (1000+ malformed) | Automated |
| SQL injection review: all parameterized | Review |
| PoToken window unreachable from main IPC | Test |

---

## 7. CI Pipeline

### 7.1 Pull Request

| Step | Tool | Gate |
|------|------|------|
| Lint | ESLint + clippy | Block |
| Format | Prettier + cargo fmt | Block |
| Type check | vue-tsc | Block |
| Unit tests (frontend) | Vitest | Block |
| Unit tests (Rust) | cargo test | Block |
| Component tests | Vitest + Testing Library | Block |
| Bindings staleness | git diff --exit-code | Block |

### 7.2 Nightly

| Step | Tool | Gate |
|------|------|------|
| E2E tests (Windows) | Playwright | Report |
| E2E tests (Linux) | Playwright | Report |
| 3-platform debug build | `tauri build --debug` | Block |
| Coverage report | tarpaulin + v8 | Report |
| Dependency audit | cargo audit + bun audit | Block |

### 7.3 Release

| Step | Tool | Gate |
|------|------|------|
| Full E2E suite | Playwright | Block |
| Platform test matrix | Manual | Block |
| Release build + sign | `tauri build` | Block |
| Notarization | Apple notarytool | Block |
| Updater verification | Manual | Block |

---

## 8. Performance Budgets

| Metric | Target | Hard Fail |
|--------|--------|-----------|
| Cold start → interactive | < 1.5 s | > 3.0 s |
| Warm start | < 0.8 s | > 1.5 s |
| Idle RSS | < 180 MB | > 300 MB |
| RSS with 3 active downloads | < 350 MB | > 600 MB |
| Installer size | < 15 MB | > 30 MB |
| Route switch | < 100 ms | > 250 ms |
| Scroll (1000-item grid) | 60 fps | < 45 fps |
| IPC p95 (local DB read) | < 20 ms | > 60 ms |
| FTS search (100k rows) | < 50 ms | > 150 ms |
| DB write (single upsert) | < 5 ms | > 20 ms |
| PoToken (cached) | < 5 ms | > 25 ms |
| 8-hour soak RSS growth | < 50 MB | > 150 MB |

---

## 9. References

- [Phase 08 — Testing & Polish](../phases/08-testing-polish.md)
- [Vitest Documentation](https://vitest.dev/)
- [Playwright Documentation](https://playwright.dev/)
- [Tauri v2 Testing](https://v2.tauri.app/develop/tests/)
- [Testing Library](https://testing-library.com/docs/vue-testing-library/intro/)
