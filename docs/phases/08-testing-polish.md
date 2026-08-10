# Phase 08 — Testing & Polish

| Field | Value |
|-------|-------|
| **Timeline** | Week 14 – Week 15 |
| **Duration** | 10 working days |
| **Risk Level** | 🟠 Medium-High (last line of defense; discovers deferred debt) |
| **Blocks** | 1.0 release |
| **Depends On** | Phases 01–07 (all feature work complete) |

---

## 1. Goals

1. Establish a durable automated test suite: unit (Vitest + `cargo test`), component, and E2E (Playwright + WebDriver).
2. **Prove data-migration integrity** — the single highest-consequence correctness requirement of the entire project.
3. Profile and fix performance regressions across startup, memory, IPC, rendering, and database access.
4. Validate the application on Windows, macOS (Intel + Apple Silicon), and three Linux distributions.
5. Close accessibility, security, and UX polish gaps.
6. Produce release artifacts, documentation, and a rollback plan for 1.0.

---

## 2. Tasks

### 2.1 Unit Tests — Vitest (Day 1–3)

```bash
bun add -D vitest @vitest/coverage-v8 @vue/test-utils @testing-library/vue \
            @testing-library/user-event jsdom happy-dom @vitest/ui
```

```ts
// vitest.config.ts
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

**Tauri IPC mocking:**

```ts
// tests/setup.ts
import { mockIPC, clearMocks } from '@tauri-apps/api/mocks';

beforeEach(() => {
  mockIPC((cmd, args) => {
    const handler = registry[cmd];
    if (!handler) throw new Error(`Unmocked command: ${cmd}`);  // fail loudly
    return handler(args);
  });
});
afterEach(() => { clearMocks(); vi.restoreAllMocks(); });
```

**Coverage plan:**

| Target | Tests | Priority |
|--------|-------|----------|
| 13 Pinia stores | State transitions, optimistic update + rollback, error paths, event application | P0 |
| `lib/api/*` wrappers | Argument shaping, error normalization, retry logic | P0 |
| Composables | `useInvoke` race cancellation, `useTauriEvent` unlisten, `useInfiniteScroll`, `useTheme` | P0 |
| Formatters | Duration, view counts, relative dates, file sizes, i18n plurals | P1 |
| Components | 16 view smoke tests + `VideoCard`, `DownloadRow`, `PlayerControls`, `QualityPicker` | P1 |
| Deep-link parser (TS side) | Valid/invalid URL grammar | P1 |

**Rust unit tests:**

- [ ] `cargo test` for repositories (in-memory SQLite), crypto KATs, yt-dlp output parsing, deep-link validation, error mapping.
- [ ] `cargo tarpaulin` (or `llvm-cov`) coverage ≥70 % on `commands/`, `db/`, `crypto/`.
- [ ] `proptest` for merge commutativity/idempotence and envelope round-trips.
- [ ] `cargo audit` + `cargo deny` clean; `bun audit` clean.

### 2.2 E2E Tests — Playwright (Day 3–5)

Tauri v2 exposes a WebDriver surface via `tauri-driver` (`msedgedriver` on Windows, `WebKitWebDriver` on Linux; **macOS is unsupported** — cover macOS with manual scripts + unit/component tests).

```ts
// tests/e2e/fixtures.ts — launches the built binary under tauri-driver
export const test = base.extend<{ app: SlyTubeApp }>({
  app: async ({}, use) => {
    const app = await launchApp({ dbFixture: 'seeded-10k.db', mockNetwork: true });
    await use(app);
    await app.close();     // asserts clean exit, no orphan processes
  },
});
```

**E2E scenario matrix:**

| ID | Scenario | Assertions |
|----|----------|-----------|
| E1 | First-run onboarding | DB created, migrations applied, defaults present |
| E2 | Legacy migration wizard | Import completes; record counts match fixture; rollback restores |
| E3 | Search → watch | Results render; player loads; progress recorded |
| E4 | Subscribe / unsubscribe | Persisted across restart; feed updates |
| E5 | Playlist lifecycle | Create → add → reorder → export → delete |
| E6 | Download lifecycle | Start → progress events → pause → **restart app** → resume → complete → open folder |
| E7 | Download failure | Network kill mid-download → error state → retry succeeds |
| E8 | Settings persistence | Change theme/proxy/quality → restart → values retained |
| E9 | Proxy enforcement | With proxy on, no direct-egress requests observed |
| E10 | Privacy Strict mode | History/search not written; no third-party image requests |
| E11 | Sync two-device | Mock relay; edits converge; conflict surfaced and resolvable |
| E12 | Deep link (warm + cold) | `opentubex://watch/<id>` routes correctly both ways |
| E13 | Tray + window lifecycle | Close-to-tray, restore, quit terminates all processes |
| E14 | Updater flow | Mock endpoint → banner → download → restart prompt |
| E15 | PoToken degradation | Force PoToken failure → app still plays/downloads at reduced formats |
| E16 | Leak guard | 30 route switches + 10 PoToken cycles → no listener growth, no extra windows |

- [ ] Deterministic fixtures: seeded DB snapshots, recorded HTTP responses (no live YouTube in CI).
- [ ] Screenshots + video captured on failure; traces uploaded as CI artifacts.
- [ ] Flake policy: any test failing >2 % over 50 runs is quarantined and fixed within the phase — never `retry` masked.

### 2.3 Data Migration Verification (Day 5–6) — **Highest Consequence**

The NeDB → SQLite import (Phase 02) must be proven, not assumed.

**Verification harness:**

```
tests/migration/
├── fixtures/
│   ├── small/        (~50 records, hand-verified golden output)
│   ├── typical/      (~5k records, anonymized real profile)
│   ├── large/        (~100k records, synthetic)
│   ├── corrupt/      (truncated final line, invalid JSON, duplicate _ids)
│   ├── legacy-v1/    (Electron-encrypted sync snapshots)
│   └── unicode/      (emoji, RTL, CJK, NFC/NFD titles)
├── expected/         (canonical JSON dumps per fixture)
└── verify.ts         (row counts, checksums, field-level diffs)
```

**Verification checklist**

- [ ] **Row-count parity:** every live NeDB doc maps to exactly one target row (tombstones excluded); counts asserted per collection.
- [ ] **Field-level diff:** dump migrated tables to canonical JSON and diff against `expected/` — zero unexplained differences.
- [ ] **Referential integrity:** `PRAGMA foreign_key_check` returns empty; no orphan `playlist_items` or `watch_history` rows.
- [ ] **Ordering preserved:** playlist item `position` sequence matches the source array order exactly.
- [ ] **Type coercions:** timestamps (ms→s), durations (string→int), view counts (`"1.2M"`→int), booleans (`"true"`→1) verified per rule table in Phase 02.
- [ ] **Unicode fidelity:** emoji/RTL/CJK titles round-trip byte-identically; NFC normalization applied consistently.
- [ ] **Idempotence:** running the import twice produces zero additional rows and zero mutations (SHA-based skip verified).
- [ ] **Corruption resilience:** corrupt fixtures import all valid records, log the invalid ones, and never abort or partially commit.
- [ ] **Rollback fidelity:** `rollback_legacy_migration` restores a byte-identical pre-import `slytube.db`.
- [ ] **Dry-run parity:** the dry-run report is identical to the real run's report.
- [ ] **Legacy crypto:** `legacy-v1/` snapshots decrypt and auto-upgrade to v2; a wrong password fails cleanly without an oracle.
- [ ] **Scale:** the 100k fixture imports in <60 s and stays under 400 MB peak RSS.
- [ ] **Manual sign-off:** a human compares 25 randomly sampled records (across all collections) between the OpenTubeX UI and SlyTube UI.

> **Release gate:** 1.0 does not ship if any migration verification item fails.

### 2.4 Performance Profiling (Day 6–8)

**Budgets:**

| Metric | Target | Hard fail |
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

**Method**

- [ ] **Frontend:** Chrome DevTools performance traces (Windows/WebView2), Vue DevTools component timings, `vite-bundle-visualizer` for chunk analysis.
- [ ] **Rust:** `cargo flamegraph` on hot paths (search, feed refresh, migration); `criterion` micro-benchmarks for crypto and parsing.
- [ ] **Memory:** `heaptrack`/Instruments for leaks; the 8-hour soak with periodic playback + downloads + sync.
- [ ] **Database:** `EXPLAIN QUERY PLAN` on every list/search query — assert index usage, no table scans on 100k-row tables; `ANALYZE` scheduled.
- [ ] **IPC:** instrument `client.ts` to log p50/p95 per command in dev; export a summary report.
- [ ] **Startup:** measure and trim the critical path — defer non-essential services (PoToken, sync scheduler, update check) behind the first paint.

**Known optimization levers (apply as needed)**

- [ ] Lazy-init `PoTokenService` and `SyncService` (do not touch them at startup).
- [ ] Batch and debounce progress-event DB writes (already specified in Phase 02 — verify).
- [ ] `shallowRef` for large lists; virtualize everything >50 rows.
- [ ] Thumbnail cache on disk with an LRU cap (500 MB) to cut repeated network fetches.
- [ ] Preload the next route's chunk on link hover.
- [ ] `PRAGMA optimize` on close; `VACUUM` offered in Settings → Advanced.

### 2.5 Platform Testing (Day 8–9)

| Platform | Version(s) | Arch | Priority |
|----------|-----------|------|----------|
| Windows | 10 22H2, 11 23H2 | x64 | P0 |
| macOS | 13 Ventura, 14 Sonoma, 15 | x64 + arm64 | P0 |
| Ubuntu | 22.04 LTS, 24.04 LTS | x64 | P0 |
| Fedora | 40 | x64 | P1 |
| Arch (rolling) | current | x64 | P1 |
| Debian | 12 | x64 | P2 |
| Windows | 11 | arm64 | P2 |

**Per-platform checklist (each must pass):**

- [ ] Fresh install from the produced installer; app launches.
- [ ] Upgrade install over the previous version preserves data.
- [ ] Uninstall removes binaries; user data removal is prompted, not silent.
- [ ] yt-dlp + ffmpeg sidecars execute; a real download completes.
- [ ] PoToken status recorded (works / falls back).
- [ ] Tray, menus, global shortcuts, deep links, file associations (per the Phase 07 matrix).
- [ ] Updater end-to-end (where supported).
- [ ] HiDPI / fractional scaling (125 %, 150 %, 200 %) renders correctly.
- [ ] Dark/light mode follows the OS.
- [ ] Multi-monitor: window restore, mini-player placement.
- [ ] Non-ASCII username paths (`C:\Users\Müller`, `/home/用户`) work for DB and downloads.
- [ ] Antivirus/Gatekeeper/SmartScreen behavior recorded.

**Distribution artifacts:**

| Platform | Artifacts |
|----------|-----------|
| Windows | `.msi`, `.exe` (NSIS), signed |
| macOS | `.dmg`, `.app.tar.gz`, signed + notarized, universal or per-arch |
| Linux | `.deb`, `.rpm`, `.AppImage`; optional Flatpak manifest |

### 2.6 Accessibility, Security & UX Polish (Day 9–10)

**Accessibility**

- [ ] `axe-core` automated scan on all 16 views — zero critical/serious violations.
- [ ] Full keyboard-only walkthrough of the primary journey.
- [ ] Screen-reader spot check: NVDA (Windows), VoiceOver (macOS), Orca (Linux).
- [ ] Contrast ≥4.5:1 for text in both themes; verify custom player controls.
- [ ] Focus order logical; focus visible; focus trapped correctly in dialogs; `Esc` closes.
- [ ] `prefers-reduced-motion` disables non-essential animation.

**Security**

- [ ] Capability audit: every permission justified; remove anything unused.
- [ ] CSP verified in the production build (no `unsafe-eval`; document any `unsafe-inline` for styles).
- [ ] Secrets audit: no tokens/passwords/keys in logs, crash reports, or the `store` plugin.
- [ ] Dependency audit: `cargo audit`, `cargo deny`, `bun audit` clean; licenses compiled into `LICENSE-THIRD-PARTY`.
- [ ] Deep-link and file-import fuzzing (1000+ malformed inputs) — no panics, no unvalidated actions.
- [ ] SQL injection review: all queries parameterized (sqlx enforces this — verify no dynamic string SQL).
- [ ] Verify the PoToken window is unreachable from the main window's IPC surface.

**UX polish**

- [ ] Consistent empty/error/loading states across all views.
- [ ] Every error message is actionable (what happened + what to do).
- [ ] No layout shift on image load; skeletons match final geometry.
- [ ] Copy review: consistent terminology, sentence case, no developer jargon.
- [ ] Icons consistent (Hugeicons only, single weight).
- [ ] First-run experience: brief onboarding + optional legacy-import prompt.

### 2.7 Release Preparation (Day 10)

- [ ] `CHANGELOG.md` for 1.0 assembled from commit history.
- [ ] User docs: install guide, migration guide from OpenTubeX, FAQ, troubleshooting.
- [ ] `README.md` updated with build instructions and platform prerequisites.
- [ ] Release CI workflow: tag → build 3 platforms → sign → notarize → publish artifacts + `latest.json`.
- [ ] Staged rollout plan (5 % → 25 % → 100 %) with a documented rollback procedure.
- [ ] Known-issues list published (including PoToken platform status).
- [ ] Post-1.0 backlog filed for everything descoped during the migration.

---

## 3. Deliverables

| ID | Deliverable | Acceptance Criteria |
|----|-------------|---------------------|
| D8.1 | Vitest suite | ≥70 % line coverage on stores/composables/api; all 16 views smoke-tested |
| D8.2 | Rust test suite | ≥70 % coverage on `commands/`, `db/`, `crypto/`; KATs + proptests green |
| D8.3 | Playwright E2E | 16 scenarios green on Windows + Linux; macOS covered by scripted manual runs |
| D8.4 | Migration verification report | All checklist items pass on 6 fixture sets; human sign-off attached |
| D8.5 | Performance report | Every budget met or an accepted, documented exception |
| D8.6 | Platform test matrix | Completed for all P0 platforms; P1 issues triaged |
| D8.7 | A11y + security audit | Zero critical findings; capability audit documented |
| D8.8 | Release artifacts + docs | Signed installers for 3 OSes; user + migration docs published |
| D8.9 | CI pipeline | Full suite runs on PR (unit) and nightly (E2E + 3-platform build) |

---

## 4. Dependencies

**Inbound**

| From | Needs |
|------|-------|
| 01 | CI skeleton, baseline metrics for comparison |
| 02 | Migration importer + fixtures |
| 03 | PoToken diagnostics + leak-test hooks |
| 04 | Command contract tests, latency budgets |
| 05 | Crypto vectors, multi-device scenarios |
| 06 | Views/components as test targets, perf budgets |
| 07 | Platform features to validate, signing/notarization set up |

**Outbound:** 1.0 release.

**External:** signing certificates, notarization service, CI runners for 3 OSes, real-hardware access for macOS arm64 and HiDPI testing.

---

## 5. Risks

| ID | Risk | Prob. | Impact | Mitigation |
|----|------|-------|--------|------------|
| R8.1 | Two weeks is insufficient to test 15 weeks of work | **High** | **High** | Shift-left: unit tests written during Phases 02–06, not deferred here; this phase *verifies*, it does not *author* the bulk of tests. If red at day 5, cut P1 platforms and delay 1.0 by one week |
| R8.2 | Migration bug found late → data loss for real users | Medium | **Critical** | Migration verification is day 5–6, not day 10; hard release gate; rollback command shipped; pre-import backup mandatory |
| R8.3 | macOS E2E unsupported by `tauri-driver` | **High** | Medium | Scripted manual test plan for macOS; compensate with heavier unit/component coverage; consider a macOS UI-automation spike post-1.0 |
| R8.4 | E2E flakiness erodes trust in CI | High | Medium | Deterministic fixtures, no live network, quarantine policy, fixed within the phase |
| R8.5 | Perf budgets missed with no time to fix | Medium | High | Profile continuously from Phase 06; treat budgets as gates at each milestone, not a final surprise |
| R8.6 | Notarization/signing delays block release | Medium | High | Certificates obtained by week 10 (Phase 07 dependency); dry-run notarization early |
| R8.7 | Platform-specific bug found on a P1 distro | High | Low | Triage matrix: P0 blocks release, P1 documented as a known issue, P2 deferred |
| R8.8 | Coverage thresholds gamed by trivial tests | Medium | Medium | Review test quality at the gate; require assertions on behavior, not just rendering |
| R8.9 | Accessibility findings require structural rework | Medium | Medium | Run axe scans during Phase 06 milestones, not only here |
| R8.10 | Deferred debt from Phases 03/06 surfaces now | High | High | Maintain a running "deferred items" register from Phase 01; review it at the start of this phase and size it explicitly |

---

## 6. Estimated Duration

| Task | Days |
|------|------|
| 2.1 Unit tests (Vitest + Rust) | 3.0 |
| 2.2 E2E tests | 2.0 |
| 2.3 Migration verification | 1.5 |
| 2.4 Performance profiling | 2.0 |
| 2.5 Platform testing | 1.5 |
| 2.6 A11y / security / UX polish | 1.5 |
| 2.7 Release preparation | 0.5 |
| **Raw total** | **12.0** |
| **Allocated** | **10.0** — requires unit tests to be substantially pre-written during Phases 02–06 |

---

## 7. Release Gate (1.0 Go/No-Go)

**Hard blockers — no release if any fail:**

- [ ] Migration verification: 100 % of the Phase 08 §2.3 checklist passing.
- [ ] No P0 platform (Windows 10/11, macOS 13–15, Ubuntu 22.04/24.04) with a blocking defect.
- [ ] Signed and notarized artifacts for all 3 OSes.
- [ ] Updater verified end-to-end on Windows, macOS, AppImage.
- [ ] No critical security or accessibility findings.
- [ ] `cargo audit` / `bun audit` clean of high-severity advisories.
- [ ] Cold start < 3.0 s and idle RSS < 300 MB on the slowest supported hardware.
- [ ] Full E2E suite green (Windows + Linux) with 0 quarantined P0 scenarios.

**Soft gates — documented as known issues if unmet:**

- PoToken unavailable on a given platform (fallback verified).
- P1/P2 distro-specific defects.
- Non-English locale completeness.
- MPRIS / SMTC now-playing integration.

---

## 8. References

- [Phase 02 — NeDB Migration](02-database-yt-dlp.md)
- [Phase 06 — Frontend Performance](06-frontend-migration.md)
- [Phase 07 — Platform Behavior Matrix](07-system-integration.md)
- [Architecture — Migration Risk Assessment](../architecture/01-electron-vs-tauri.md)
- Previous: [Phase 07 — System Integration](07-system-integration.md)
