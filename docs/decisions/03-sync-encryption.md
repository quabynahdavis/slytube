# ADR 003: Sync Encryption Implementation Location

| Field | Value |
|-------|-------|
| **Status** | Accepted |
| **Date** | 2026-08-09 |
| **Deciders** | Migration Team |
| **Supersedes** | — |
| **Related** | [04-database-choice.md](04-database-choice.md), [05-migration-approach.md](05-migration-approach.md) |

---

## Context

Slytube offers **end-to-end encrypted sync** of user data (subscriptions, playlists, history,
settings) across devices. The server is untrusted: it stores ciphertext and never sees
plaintext or key material.

The existing Electron implementation performs all cryptography in the **renderer** using the
**WebCrypto API**:

| Primitive | Use |
|-----------|-----|
| **AES-GCM (256-bit)** | Record and payload encryption; provides confidentiality + integrity |
| **X25519** | ECDH key agreement for device pairing and per-session key exchange |
| **HKDF (SHA-256)** | Key derivation from the shared secret into distinct encryption/MAC subkeys with domain-separating `info` labels |
| **Argon2 / PBKDF2** | Passphrase stretching for the root sync key |

Characteristics of the current design that constrain any change:

- **The wire protocol is already deployed.** Existing users have ciphertext on the server and
  paired devices in the field. Any new implementation must interoperate byte-for-byte.
- **Sync is bulk work.** A full pull can decrypt thousands of records (history is the worst
  case). Doing this on the renderer's single JS thread visibly stutters the UI.
- **Key material lives in renderer memory.** WebCrypto `CryptoKey` objects can be marked
  non-extractable, but the surrounding envelope handling, derived secrets, and plaintext all sit
  in the JS heap alongside third-party dependencies.

Tauri v2 gives us a Rust backend with mature, audited crypto crates and a real thread pool.
The question is whether sync crypto should move there.

---

## Options Considered

### Option A — Port the crypto to Rust

Implement the full protocol in `src-tauri/src/sync/crypto.rs` using RustCrypto crates and expose
high-level commands (`sync_push`, `sync_pull`, `sync_pair_device`) to the renderer.

```toml
# src-tauri/Cargo.toml (sketch)
aes-gcm      = "0.10"   # AES-256-GCM, AEAD trait
x25519-dalek = "2"      # ECDH key agreement
hkdf         = "0.12"   # RFC 5869 HKDF
sha2         = "0.10"
argon2       = "0.5"    # passphrase stretching
zeroize      = "1"      # secret scrubbing on drop
```

| Pros | Cons |
|------|------|
| Native speed — no JS engine, no WASM boundary | Must reproduce the existing protocol **exactly**, byte for byte |
| Runs on a background Tokio task; UI never blocks | Rust crypto errors are compile-time-safe but protocol errors are not |
| Key material owned by Rust, `Zeroize`d on drop | Requires careful, high-coverage differential testing against the JS impl |
| Streaming decryption of large batches without heap pressure in the webview | Team must be comfortable reviewing Rust crypto code |
| Constant-time implementations by construction in RustCrypto | Debugging crypto mismatches across a language boundary is painful |
| Sync can run while the window is hidden or minimised | |

### Option B — Keep WebCrypto in the Renderer

Port `sync.js` to TypeScript unchanged; continue using `window.crypto.subtle`.

| Pros | Cons |
|------|------|
| Zero protocol risk — it is literally the same code | Bulk decryption blocks the UI thread; history sync visibly janks |
| No new dependencies | Key material and plaintext sit in the renderer heap |
| Fastest to migrate | Sync cannot progress while the webview is throttled/backgrounded |
| WebCrypto is browser-audited | WebCrypto's X25519 support varies across WebKitGTK / WKWebView / WebView2 |
| | Argon2 is **not** in WebCrypto — needs a JS/WASM library anyway |
| | Ties sync availability to webview lifecycle |

### Option C — WASM Crypto Module

Compile a Rust crypto core to WASM and call it from the renderer.

| Pros | Cons |
|------|------|
| Single crypto implementation, callable from JS | Slower than native Rust (typically 1.5–3× for AEAD workloads) |
| Consistent across all webview engines | Still executes on the renderer thread unless moved to a Worker |
| Reuses RustCrypto crates | Adds a WASM build step and a bundle-size cost to a native app |
| | Secrets still land in the webview's linear memory — no real isolation win |
| | Worst of both worlds: Rust's review burden **and** the renderer's threading limits |

---

## Decision

**Adopt Option A — port the sync crypto to Rust.**

All AES-GCM, X25519, HKDF, and passphrase-stretching operations move to
`src-tauri/src/sync/`. The renderer's surface shrinks to intent-level commands:

```
invoke('sync_pair_device', { pairing_code })
invoke('sync_push')
invoke('sync_pull')
listen('sync-progress', ...)
```

No key material, derived secret, or ciphertext envelope crosses into the webview.

### Target module layout

```
src-tauri/src/sync/
├── mod.rs          service wiring, background task, scheduling
├── crypto.rs       AES-GCM / X25519 / HKDF / Argon2 primitives
├── envelope.rs     wire format encode/decode (version-tagged)
├── keystore.rs     key persistence, Zeroize-on-drop wrappers
├── protocol.rs     push/pull, conflict resolution, cursors
└── legacy.rs       decrypt-only support for pre-migration records
```

---

## Rationale

1. **Performance.** Native AES-GCM with AES-NI hardware acceleration is dramatically faster
   than WebCrypto-through-a-webview for batch workloads, and the gap widens with record count.
   A full history pull is the pathological case; it is also the case users hit on every new
   device.

2. **Native crypto quality.** RustCrypto (`aes-gcm`, `x25519-dalek`, `hkdf`) provides audited,
   constant-time implementations with an ergonomic AEAD trait surface. Crucially, `argon2` is
   available natively — WebCrypto has no Argon2, so Option B would require pulling in a JS or
   WASM Argon2 anyway, undermining its "no new dependencies" advantage.

3. **Background processing.** Sync becomes a Tokio task independent of the webview. It can run
   during startup before first paint, continue while the window is minimised, and survive
   navigation within the SPA — none of which is possible in the renderer, where background
   throttling and page lifecycle actively work against long-running work.

4. **Secret containment.** Rust owns the keys, wraps them in `Zeroize` types, and never
   serialises them across IPC. This meaningfully shrinks the blast radius of any renderer-side
   compromise (a malicious dependency, an XSS in embedded content). This is the same reasoning
   that made the *opposite* call acceptable in ADR 002 — Invidious tokens are low-value; sync
   keys are the crown jewels.

5. **Alignment with the rest of the backend.** Sync writes to SQLite (ADR 004). Keeping crypto
   in the renderer would mean: Rust reads rows → serialises to JS → JS decrypts → JS sends back
   → Rust writes. Co-locating crypto with storage collapses that into one in-process pipeline.

---

## Implications

### Protocol fidelity is the dominant risk

The Rust implementation **must match the existing protocol exactly**. Every one of the following
must be byte-identical to the WebCrypto implementation:

- [ ] AES-GCM nonce length (12 bytes) and **nonce derivation/counter scheme**
- [ ] Tag length (16 bytes) and whether the tag is appended or stored separately
- [ ] AAD (additional authenticated data) contents, field order, and encoding
- [ ] HKDF salt, and the exact `info` byte strings used for domain separation
- [ ] X25519 public key encoding and the clamping applied to scalars
- [ ] Argon2 variant (id/i/d), version, memory cost, time cost, parallelism, and salt handling
- [ ] Envelope framing: version byte, field order, length prefixes, base64 vs raw
- [ ] Canonical JSON serialisation of the plaintext **before** encryption (key ordering matters)

**Required verification work:**

- [ ] Extract a golden-vector corpus from the current Electron build: for a fixed key and fixed
      plaintext, capture the exact ciphertext envelopes.
- [ ] Rust unit tests that decrypt every golden vector and re-encrypt to the identical bytes
      (deterministic nonces where the protocol allows; otherwise assert decrypt-round-trip).
- [ ] A differential test harness: same input → JS impl and Rust impl → assert equality.
- [ ] Interop test against a real sync server with a device paired by the old client.

### Legacy decryption for migration

- [ ] `sync/legacy.rs` must be able to **decrypt** any record written by the Electron client,
      including older envelope versions still present on the server.
- [ ] The envelope must carry a version tag; the reader dispatches on it, the writer always
      emits the current version.
- [ ] Records are lazily re-encrypted to the current version on next write — no forced
      server-side rewrite.
- [ ] Legacy paths are **decrypt-only**. Never add a code path that writes a legacy envelope.
- [ ] Document a removal horizon for legacy support (recommend: two minor releases after the
      migration ships) and gate it behind a telemetry-free version check.

### Operational consequences

| Area | Consequence |
|------|-------------|
| **Key storage** | Keys move from renderer memory into Rust-owned state, persisted via the OS keychain where available and an encrypted local store otherwise. |
| **Error surface** | Crypto failures must be distinguishable: *wrong passphrase* vs *corrupt envelope* vs *unsupported version*. Generic "decryption failed" is unacceptable for support. |
| **Progress reporting** | Long syncs emit `sync-progress` events so the renderer can render a determinate progress bar. |
| **Auditability** | Crypto code must be isolated in `sync/crypto.rs` with no I/O, so it can be reviewed and fuzzed independently. |
| **No renderer fallback** | Once ported, there is no JS crypto path. A Rust-side bug takes sync down entirely — hence the emphasis on golden vectors. |
| **Zeroize discipline** | Every secret-bearing struct derives `ZeroizeOnDrop`. Reviewers must check this on any new type touching key material. |

---

## References

- Electron baseline: sync engine within `src/main/index.js` + renderer WebCrypto helpers
- [RustCrypto AEADs — `aes-gcm`](https://docs.rs/aes-gcm/)
- [`x25519-dalek`](https://docs.rs/x25519-dalek/) · [`hkdf`](https://docs.rs/hkdf/) · [`argon2`](https://docs.rs/argon2/)
- [RFC 5869 — HKDF](https://datatracker.ietf.org/doc/html/rfc5869)
- [../architecture/01-electron-vs-tauri.md](../architecture/01-electron-vs-tauri.md) §"Sync Engine"
