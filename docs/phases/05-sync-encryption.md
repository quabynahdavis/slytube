# Phase 05 — Sync & Encryption

| Field | Value |
|-------|-------|
| **Timeline** | Week 5 – Week 6 |
| **Duration** | 10 working days |
| **Risk Level** | 🟠 Medium-High (correctness-critical, data-loss adjacent) |
| **Blocks** | Phase 06 (sync UI views) |
| **Depends On** | Phase 02 (`sync_*` tables, tombstones), Phase 04 (sync DTOs) |

---

## 1. Goals

1. Replace Node's `crypto` module with a native Rust crypto stack — Argon2id, X25519, HKDF, AES-256-GCM — with constant-time primitives and zeroized key material.
2. Implement the **snapshot protocol**: a versioned, encrypted, delta-capable state exchange between devices via a zero-knowledge relay server.
3. Implement **legacy decryption** so users migrating from OpenTubeX can read snapshots produced by the Electron implementation.
4. Implement **privacy modes** governing what data is eligible for sync, telemetry, and local retention.
5. Deliver deterministic conflict detection and resolution with a user-visible audit trail.

**Security posture:** the sync server is **untrusted**. It stores ciphertext and routing metadata only. It must never be able to derive plaintext, and compromise of the server must not compromise past or future sessions beyond the encrypted blobs it already holds.

---

## 2. Threat Model

| Adversary | Capability | Mitigation |
|-----------|-----------|------------|
| Sync server operator | Reads all stored blobs + metadata | E2E encryption; server sees only device pubkeys, blob sizes, timestamps |
| Network attacker (MITM) | Intercepts/replays traffic | TLS + AEAD + monotonic nonces + snapshot sequence numbers + signature verification |
| Malicious device (revoked) | Holds an old master key | Device revocation + key rotation + epoch counter; revoked device cannot decrypt post-rotation epochs |
| Local attacker (disk) | Reads app data at rest | Keys never persisted in plaintext; OS keychain for the wrapped master key; DB optionally encrypted |
| Offline password guesser | Has a stolen blob | Argon2id with high memory cost; per-user random salt; no password verifier stored server-side |

**Out of scope:** malware with live process memory access on an unlocked device; compromised OS keychain.

---

## 3. Tasks

### 3.1 Rust Crypto Implementation (Day 1–3)

```bash
cargo add aes-gcm chacha20poly1305 x25519-dalek ed25519-dalek hkdf sha2 argon2 \
          zeroize rand_core getrandom subtle base64 --manifest-path src-tauri/Cargo.toml
```

**Key hierarchy:**

```
User password ──Argon2id(salt, m=64MiB, t=3, p=4)──► Master Key (32B)
                                                          │
              ┌───────────────────────────────────────────┼──────────────────────────┐
              ▼                                           ▼                          ▼
   HKDF-SHA256(info="slytube/v1/identity")   HKDF(info="slytube/v1/snapshot")   HKDF(info="slytube/v1/wrap")
              │                                           │                          │
       Ed25519 signing key                    Snapshot content key (32B)     Keychain wrapping key
       X25519 static key (device identity)              │
                                                        ▼
                                      Per-snapshot key = HKDF(content key, salt=snapshot_id)
```

**Module** (`src-tauri/src/crypto/mod.rs`):

```rust
use zeroize::{Zeroize, ZeroizeOnDrop};

#[derive(ZeroizeOnDrop)]
pub struct MasterKey([u8; 32]);

pub struct KdfParams { pub salt: [u8; 16], pub m_cost: u32, pub t_cost: u32, pub p_cost: u32, pub version: u8 }

impl Default for KdfParams {
    fn default() -> Self {
        Self { salt: random_bytes(), m_cost: 65_536 /*64 MiB*/, t_cost: 3, p_cost: 4, version: 1 }
    }
}

pub fn derive_master_key(password: &str, p: &KdfParams) -> Result<MasterKey, CryptoError> {
    let argon = argon2::Argon2::new(
        argon2::Algorithm::Argon2id,
        argon2::Version::V0x13,
        argon2::Params::new(p.m_cost, p.t_cost, p.p_cost, Some(32))?,
    );
    let mut out = [0u8; 32];
    argon.hash_password_into(password.as_bytes(), &p.salt, &mut out)?;
    Ok(MasterKey(out))
}

pub fn subkey(master: &MasterKey, info: &[u8]) -> [u8; 32] {
    let hk = hkdf::Hkdf::<sha2::Sha256>::new(None, &master.0);
    let mut out = [0u8; 32];
    hk.expand(info, &mut out).expect("32 is a valid length");
    out
}
```

**AEAD envelope** — a single canonical binary format for every ciphertext:

```
┌────────┬─────────┬────────┬──────────┬───────────┬─────────────┬──────────┐
│ magic  │ version │ suite  │ epoch    │ nonce     │ ciphertext  │ tag      │
│ "SLYT" │ u8 = 2  │ u8     │ u32 BE   │ 12 bytes  │ N bytes     │ 16 bytes │
└────────┴─────────┴────────┴──────────┴───────────┴─────────────┴──────────┘
suite: 0x01 = AES-256-GCM, 0x02 = XChaCha20-Poly1305 (24B nonce)
AAD = magic || version || suite || epoch || context_id
```

- [ ] **Nonce discipline:** 96-bit nonce = 4-byte device counter prefix ‖ 8 bytes CSPRNG. Counter persisted; a counter regression aborts encryption (never reuse a nonce under one key).
- [ ] Prefer AES-256-GCM where AES-NI is available; fall back to XChaCha20-Poly1305 on platforms without hardware AES (detected via `is_x86_feature_detected!`).
- [ ] All comparisons of secrets use `subtle::ConstantTimeEq`.
- [ ] `MasterKey`, derived subkeys, and plaintext buffers implement `ZeroizeOnDrop`.
- [ ] Master key is held in memory only while sync is unlocked; wrapped copy stored in the OS keychain (`keyring` crate) when "remember" is enabled.
- [ ] Add KAT (known-answer test) vectors for every primitive plus round-trip property tests (`proptest`).

**Device identity & pairing:**

```rust
pub struct DeviceIdentity {
    pub id: String,               // base64url(ed25519 pubkey)
    pub signing: ed25519_dalek::SigningKey,
    pub kex: x25519_dalek::StaticSecret,
    pub name: String,
    pub created_at: i64,
}
```

- [ ] Pairing: new device generates its keypair, derives the master key from the same password, and uploads a **self-signed enrollment record**; an existing device must approve it (emits `sync-device-pending`).
- [ ] Approval produces a short numeric **verification code** derived from `HKDF(shared_secret)` — displayed on both devices to defeat MITM enrollment.
- [ ] Every snapshot is Ed25519-signed by the producing device; consumers verify before decrypting.

### 3.2 Snapshot Protocol (Day 3–6)

**Snapshot document (plaintext, pre-encryption):**

```jsonc
{
  "protocol": 2,
  "snapshotId": "01J...ULID",
  "deviceId": "base64url-pubkey",
  "epoch": 3,
  "createdAt": 1775000000,
  "baseSnapshotId": "01J...",      // null ⇒ full snapshot
  "kind": "full" | "delta",
  "vectorClock": { "deviceA": 187, "deviceB": 42 },
  "collections": {
    "settings":      { "upserts": [...], "deletes": [...] },
    "playlists":     { "upserts": [...], "deletes": [...] },
    "subscriptions": { "upserts": [...], "deletes": [...] },
    "watchHistory":  { "upserts": [...], "deletes": [...] },
    "searchHistory": { "upserts": [...], "deletes": [...] }
  },
  "checksum": "blake3:..."          // over canonicalized collections
}
```

**Wire flow:**

```
Device A                          Relay Server                     Device B
   │  GET  /manifest ──────────────────►│                             │
   │◄──── {snapshots:[{id,device,epoch,size,seq}]}                    │
   │                                    │                             │
   │  build delta since lastSyncedSeq   │                             │
   │  canonicalize → BLAKE3 → sign      │                             │
   │  compress (zstd -3)                │                             │
   │  encrypt (AEAD, per-snapshot key)  │                             │
   │  PUT /snapshots ──────────────────►│  stores ciphertext only     │
   │                                    │◄──── GET /snapshots?since= ─│
   │                                    │───── ciphertext ───────────►│
   │                                    │        verify sig → decrypt │
   │                                    │        → merge → apply      │
```

**Implementation checklist**

- [ ] **Canonicalization** before hashing/signing: sorted keys, no insignificant whitespace, integers not floats, UTF-8 NFC. Two devices must produce byte-identical serializations for identical state.
- [ ] **Compression before encryption** (zstd level 3). Guard against compression-oracle leakage by padding to a 4 KiB bucket.
- [ ] **Delta selection:** send a delta when a common base exists and delta size < 60 % of full; otherwise full snapshot.
- [ ] **Chunking:** snapshots >4 MiB split into 4 MiB encrypted chunks with per-chunk AEAD and a manifest; resumable upload/download.
- [ ] **Vector clocks** per device for causality; `updated_at` is a tiebreaker only, never the primary ordering signal (clock skew).
- [ ] **Tombstones** (`deleted_at`) propagate deletions; garbage-collected after 90 days once all known devices have acknowledged past that sequence.
- [ ] **Idempotency:** applying the same snapshot twice is a no-op (`sync_log` dedupes by `snapshot_id`).
- [ ] **Atomic apply:** the entire merge runs in one SQLite transaction; partial application is impossible.
- [ ] **Scheduling:** debounce 30 s after a local change; periodic every 15 min; immediate on `trigger_sync`; exponential backoff on failure (max 30 min); pause on metered connections when detectable.

**Conflict resolution matrix:**

| Collection | Strategy | Rationale |
|-----------|----------|-----------|
| `settings` | Last-writer-wins per key (vector clock) | Scalar values, low stakes |
| `subscriptions` | Add-wins set (OR-Set semantics) | Accidental unsub loss is worse than a stale sub |
| `playlists` (metadata) | LWW per field | |
| `playlist_items` | Ordered merge by `position`, ties broken by `deviceId`; duplicates deduped by `video_id` | Preserves user intent |
| `watchHistory` | Union; per-video keep max `watch_progress`, max `watched_at` | Monotonic |
| `searchHistory` | Union, capped at 500 most recent | Low stakes |
| Structural conflict (same entity, divergent non-mergeable fields) | Record in `sync_conflicts`, surface to user | Never guess destructively |

- [ ] `resolve_conflict(conflict_id, resolution, merged_data)` applies the user's choice and re-queues a snapshot.
- [ ] Every applied merge writes to `sync_log` for the audit trail shown in the Sync view.

**Sync commands finalized (from Phase 04 stubs):**

| Command | Implemented behavior |
|---------|---------------------|
| `enable_sync(password, server_url, device_name)` | KDF → identity → enrollment → initial full snapshot |
| `disable_sync(wipe_remote: bool)` | Zeroize keys, clear keychain, optionally request remote purge |
| `trigger_sync()` | Full pull-merge-push cycle, returns `SyncResult` |
| `get_sync_status()` | Live state incl. epoch, pending changes, last error |
| `get_sync_devices()` / `remove_sync_device(id)` | Revocation triggers epoch bump + key rotation |
| `approve_sync_device(id, code)` | Verification-code-gated enrollment approval |
| `resolve_conflict(...)` | |
| `export_sync_recovery_kit()` | Encrypted backup of identity + KDF params (user-held) |

### 3.3 Legacy Decryption (Day 6–8)

OpenTubeX (Electron) produced snapshots with Node `crypto`. We must **read** protocol v1 and **write** only v2.

**Legacy format (v1) — reverse-engineered contract:**

| Aspect | Legacy (v1) | New (v2) |
|--------|-------------|----------|
| KDF | PBKDF2-HMAC-SHA256, 100k iters, 16B salt | Argon2id, 64 MiB / t=3 / p=4 |
| Cipher | AES-256-GCM, 12B IV | AES-256-GCM or XChaCha20-Poly1305 |
| Envelope | `salt(16) ‖ iv(12) ‖ ciphertext ‖ tag(16)`, base64 | `SLYT` magic envelope (§3.1) |
| AAD | none | magic ‖ version ‖ suite ‖ epoch ‖ context |
| Payload | raw JSON | canonicalized JSON, zstd-compressed |
| Signature | none | Ed25519 per snapshot |

```rust
pub fn decrypt_legacy_v1(password: &str, blob_b64: &str) -> Result<Vec<u8>, CryptoError> {
    let raw = base64::engine::general_purpose::STANDARD.decode(blob_b64)?;
    if raw.len() < 16 + 12 + 16 { return Err(CryptoError::Malformed); }
    let (salt, rest) = raw.split_at(16);
    let (iv, ct_and_tag) = rest.split_at(12);

    let mut key = [0u8; 32];
    pbkdf2::pbkdf2_hmac::<sha2::Sha256>(password.as_bytes(), salt, 100_000, &mut key);

    let cipher = aes_gcm::Aes256Gcm::new_from_slice(&key)?;
    let pt = cipher.decrypt(aes_gcm::Nonce::from_slice(iv), ct_and_tag)
        .map_err(|_| CryptoError::AuthenticationFailed)?;
    key.zeroize();
    Ok(pt)
}
```

**Migration path**

- [ ] Detect version: legacy blobs lack the `SLYT` magic → route to v1 decryptor.
- [ ] On first successful v1 decrypt, immediately **re-key**: derive a v2 master key from the same password, re-encrypt, upload as a v2 full snapshot, and mark the epoch bump.
- [ ] Support a mixed fleet during transition: v2 devices can read v1; v1 devices cannot read v2. Warn the user that older OpenTubeX installs must upgrade, and never delete v1 blobs until all devices report v2 (or the user confirms).
- [ ] `import_legacy_snapshot(file_path, password)` command for manual recovery from an exported OpenTubeX backup file.
- [ ] Golden-vector tests: fixtures produced by the Electron implementation must decrypt byte-exactly.
- [ ] Wrong password must return a distinct, non-oracle error (`CryptoError::AuthenticationFailed`) with constant-time behavior and rate limiting (max 5 attempts / 60 s).

### 3.4 Privacy Modes (Day 8–9)

A single setting governing data collection, retention, and sync eligibility.

| Mode | Sync scope | Local retention | Telemetry | Network behavior |
|------|-----------|-----------------|-----------|------------------|
| **Standard** (default) | settings, playlists, subscriptions, watch history, search history | Unlimited history | Opt-in crash reports only | Direct or configured proxy |
| **Balanced** | settings, playlists, subscriptions (no history) | Watch history 30 d, search history 7 d | None | Proxy recommended; thumbnails proxied |
| **Strict** | settings + playlists only (no channel/watch data) | Watch history session-only (in-memory), search history disabled | None | Proxy required; external images blocked or proxied; no third-party requests |
| **Ephemeral / Incognito** (session toggle) | Nothing synced | Nothing persisted this session | None | Inherits Strict networking |

```rust
#[derive(Serialize, Deserialize, Clone, Copy, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum PrivacyMode { Standard, Balanced, Strict }

pub struct PrivacyPolicy {
    pub mode: PrivacyMode,
    pub ephemeral: bool,
    pub syncable: HashSet<Collection>,
    pub retention: HashMap<Collection, Option<Duration>>,
    pub require_proxy: bool,
    pub block_remote_images: bool,
}
```

**Requirements**

- [ ] The policy is enforced at the **snapshot builder** level — non-syncable collections are never serialized, not merely filtered client-side.
- [ ] Tightening a mode triggers a purge: local rows outside the new retention window are deleted, and a **tombstone snapshot** removes previously-synced data from other devices.
- [ ] Loosening a mode never resurrects purged data (it's gone) — the UI must state this before confirming.
- [ ] A retention reaper runs hourly and on startup.
- [ ] Ephemeral mode routes all writes to an in-memory overlay discarded on exit; a crash must not leave residue (verified by an E2E test in Phase 08).
- [ ] `get_privacy_report()` returns exactly what is stored and what would be synced — a transparency surface for the Settings view.

### 3.5 Sync Service Wiring & Tests (Day 9–10)

- [ ] `SyncService` background task: scheduler, backoff, connectivity awareness, single-flight (never two cycles at once).
- [ ] Events: `sync-status-changed`, `sync-progress` (`{ phase, current, total }`), `sync-conflict-detected`, `sync-device-pending`, `sync-error`.
- [ ] Offline queue: local changes accumulate as dirty flags; sync resumes on reconnect.
- [ ] Test suite:
  - Unit: KAT vectors, envelope round-trip, nonce-counter regression rejection, canonicalization determinism.
  - Property: `decrypt(encrypt(x)) == x` for arbitrary payloads; merge commutativity/idempotence for CRDT-ish collections.
  - Integration: 3 simulated devices against a mock relay — concurrent edits, offline edit + reconcile, device revocation, legacy v1 → v2 upgrade.
  - Negative: tampered ciphertext, replayed snapshot, wrong signature, wrong password, truncated chunk.

---

## 4. Deliverables

| ID | Deliverable | Acceptance Criteria |
|----|-------------|---------------------|
| D5.1 | `crypto` module | KAT vectors pass; keys zeroized; no nonce reuse possible by construction |
| D5.2 | Device identity & pairing | Verification-code-gated enrollment; Ed25519-signed snapshots |
| D5.3 | Snapshot protocol v2 | Full + delta + chunking + resumable transfer; deterministic canonicalization |
| D5.4 | Conflict engine | Per-collection strategies implemented; unresolvable conflicts surfaced, never silently dropped |
| D5.5 | Legacy v1 decryption + auto re-key | Electron-produced golden fixtures decrypt exactly; automatic upgrade to v2 |
| D5.6 | Privacy modes | Enforced at snapshot build; retention reaper; ephemeral leaves no residue |
| D5.7 | 8 sync commands (real) | Signatures unchanged from Phase 04 |
| D5.8 | Test suite | 3-device integration scenarios green; all negative tests reject correctly |
| D5.9 | Security ADR | Threat model, primitive choices, key hierarchy, rotation policy documented |

---

## 5. Dependencies

**Inbound**

| From | Needs |
|------|-------|
| Phase 02 | `sync_devices`, `sync_state`, `sync_conflicts`, `sync_log`, tombstones, `updated_at` |
| Phase 04 | Frozen sync DTOs, HTTP client factory, keychain access |

**Outbound**

| Phase | Consumes |
|-------|----------|
| 06 | Sync settings view, device list, conflict resolution UI, privacy mode selector, legacy import wizard |
| 08 | Crypto test vectors, 3-device E2E scenarios, ephemeral-residue test |

**External:** relay server availability and API contract; OS keychain.

---

## 6. Risks

| ID | Risk | Prob. | Impact | Mitigation |
|----|------|-------|--------|------------|
| R5.1 | Nonce reuse under one key destroys AES-GCM security | Low | **Critical** | Counter+random construction; persisted counter; regression aborts; property test asserting uniqueness across 10⁶ encryptions |
| R5.2 | Merge bug silently deletes user playlists/subscriptions | Medium | **Critical** | Add-wins semantics; tombstone-only deletion; atomic transaction; pre-merge DB snapshot retained for 7 days; 3-device integration tests |
| R5.3 | Canonicalization differs across platforms → signature failures | Medium | High | Single canonicalizer implementation with cross-platform golden hashes in CI |
| R5.4 | Legacy v1 format assumptions wrong → migration fails | Medium | High | Validate against real OpenTubeX fixtures early (day 1 spike); ship `import_legacy_snapshot` manual path |
| R5.5 | Argon2 64 MiB stalls low-end devices | Medium | Medium | Benchmark on target hardware; adaptive params stored in the envelope; run on `spawn_blocking` with a progress event |
| R5.6 | Revoked device retains access to future data | Medium | High | Epoch bump + content-key rotation on revocation; re-upload full snapshot under new epoch |
| R5.7 | Server metadata leaks behavior patterns (sizes, timing) | Medium | Low | Padding to 4 KiB buckets; jittered sync schedule |
| R5.8 | Password loss = permanent data loss | High | Medium | Recovery-kit export at setup; explicit, unmissable warning; no server-side reset (by design) |
| R5.9 | Clock skew corrupts ordering | Medium | Medium | Vector clocks primary; timestamps advisory only; reject snapshots >24 h in the future |
| R5.10 | Privacy mode tightening fails to purge remote copies | Medium | High | Tombstone snapshot on tighten; verify via `get_privacy_report`; integration test asserting remote removal |

---

## 7. Estimated Duration

| Task | Days |
|------|------|
| 3.0 Legacy format spike | 0.5 |
| 3.1 Rust crypto | 2.5 |
| 3.2 Snapshot protocol | 3.0 |
| 3.3 Legacy decryption + re-key | 1.5 |
| 3.4 Privacy modes | 1.5 |
| 3.5 Service wiring + tests | 1.0 |
| **Total** | **10.0** (2 weeks @ 1 dev) |

> Overlaps Phase 04 (weeks 5–6). Requires the sync DTOs frozen by Phase 04 day 8.

---

## 8. Exit Criteria

- [ ] All crypto KAT vectors pass; `cargo test --package crypto` green on 3 OSes.
- [ ] 3-device simulation: concurrent edits converge to identical DB checksums on all devices.
- [ ] Offline device reconciles after 24 h of divergence with zero data loss.
- [ ] Device revocation verified: revoked device cannot decrypt post-rotation snapshots.
- [ ] Real OpenTubeX v1 snapshot imports and auto-upgrades to v2.
- [ ] Strict mode: packet capture confirms no history/search data leaves the device.
- [ ] Ephemeral mode: after a forced kill, no residue in `slytube.db` or on disk.
- [ ] Security ADR merged in `docs/decisions`.

---

## 9. References

- [Architecture — Sync Encryption Flow](../architecture/03-data-flow.md#sync-encryption-flow)
- [Backend — Sync Commands](../backend/02-tauri-commands.md#sync-commands)
- [Backend — Sync Tables](../backend/01-database-schema.md)
- Previous: [Phase 04 — Backend Commands](04-backend-commands.md) · Next: [Phase 06 — Frontend Migration](06-frontend-migration.md)
