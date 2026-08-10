# 05 - Sync & Encryption

> **Domain:** `backend`
> **Status:** Design specification (implementation target for `src-tauri/src/sync`)
> **Related:** [01-database-schema.md](01-database-schema.md), [02-tauri-commands.md](02-tauri-commands.md#10-module-sync)

---

## 1. Overview

SlyTube sync is **end-to-end encrypted** and **snapshot-based**. The server is a dumb blob store: it sees ciphertext, a device id, a collection name, and a monotonic version. It never sees plaintext, and it holds no key material capable of decrypting anything.

Design commitments:

| Property | Mechanism |
|---|---|
| Confidentiality | AES-256-GCM, keys derived client-side only |
| Integrity / authenticity | GCM tag over ciphertext + AAD binding collection, version, and device |
| Forward secrecy between devices | X25519 ECDH per pairing, ephemeral keys |
| Key from a human secret | PBKDF2-HMAC-SHA256 (legacy) / Argon2id (preferred) → HKDF-SHA256 per collection |
| Replay resistance | Monotonic `version` in AAD; server rejects regressions, client rejects stale |
| Metadata minimisation | `enhanced` privacy mode strips timestamps and drops sensitive collections |

---

## 2. Rust Crypto Stack

```toml
# src-tauri/Cargo.toml
[dependencies]
aes-gcm       = { version = "0.10", features = ["aes", "alloc"] }
x25519-dalek  = { version = "2", features = ["static_secrets", "zeroize"] }
hkdf          = "0.12"
pbkdf2        = { version = "0.12", features = ["simple"] }
argon2        = "0.5"
sha2          = "0.10"
hmac          = "0.12"
rand          = "0.8"
rand_core     = { version = "0.6", features = ["getrandom"] }
zeroize       = { version = "1", features = ["derive"] }
subtle        = "2"
base64        = "0.22"
```

| Crate | Role | Notes |
|---|---|---|
| `aes-gcm` | AEAD | AES-256-GCM, 96-bit nonce, 128-bit tag. Hardware AES-NI when available |
| `x25519-dalek` | ECDH | Device pairing and per-device key wrapping |
| `hkdf` | KDF | Domain separation: one root secret → many purpose-bound subkeys |
| `pbkdf2` | Password KDF | **Legacy only** — required to read v1 payloads |
| `argon2` | Password KDF | Argon2id for all new vaults |
| `zeroize` | Hygiene | Every key type is `ZeroizeOnDrop` |
| `subtle` | Constant time | Tag/fingerprint comparison |

### 2.1 Key material types

```rust
// src-tauri/src/sync/crypto/keys.rs
use zeroize::{Zeroize, ZeroizeOnDrop};

#[derive(Zeroize, ZeroizeOnDrop)]
pub struct RootKey([u8; 32]);

#[derive(Zeroize, ZeroizeOnDrop)]
pub struct CollectionKey([u8; 32]);

#[derive(Zeroize, ZeroizeOnDrop)]
pub struct SharedSecret([u8; 32]);

impl RootKey {
    pub fn generate() -> Self {
        let mut k = [0u8; 32];
        rand::RngCore::fill_bytes(&mut rand::rngs::OsRng, &mut k);
        Self(k)
    }
    pub(crate) fn as_bytes(&self) -> &[u8; 32] { &self.0 }
}
```

Keys live only in memory, inside an `Arc<RwLock<Option<UnlockedVault>>>` in `AppState`. The vault auto-locks after 30 minutes of sync inactivity and on OS suspend.

---

## 3. Key Derivation

```
                     user passphrase
                            │
                  Argon2id (m=64 MiB, t=3, p=4)      salt: 16B random, stored
                            │
                            ▼
                      ROOT KEY (32B)
                            │
        ┌───────────────────┼───────────────────────────┐
        │ HKDF-SHA256       │ HKDF-SHA256               │ HKDF-SHA256
        │ info="slytube/v2/ │ info="slytube/v2/         │ info="slytube/v2/
        │  collection/…"    │  device-wrap"             │  auth"
        ▼                   ▼                           ▼
  COLLECTION KEYS     DEVICE WRAP KEY            AUTH TOKEN KEY
  (one per collection) (wraps X25519 secret)     (server auth, never
                                                  reveals root key)
```

### 3.1 Passphrase → root key

```rust
use argon2::{Argon2, Algorithm, Params, Version};

pub const ARGON2_MEM_KIB: u32 = 65_536;   // 64 MiB
pub const ARGON2_TIME: u32    = 3;
pub const ARGON2_LANES: u32   = 4;

pub fn derive_root_argon2(passphrase: &str, salt: &[u8; 16]) -> Result<RootKey, CryptoError> {
    let params = Params::new(ARGON2_MEM_KIB, ARGON2_TIME, ARGON2_LANES, Some(32))
        .map_err(|e| CryptoError::Kdf(e.to_string()))?;
    let argon = Argon2::new(Algorithm::Argon2id, Version::V0x13, params);

    let mut out = [0u8; 32];
    argon
        .hash_password_into(passphrase.as_bytes(), salt, &mut out)
        .map_err(|e| CryptoError::Kdf(e.to_string()))?;
    Ok(RootKey::from_bytes(out))
}
```

Argon2id at 64 MiB is deliberately run on `spawn_blocking` — it takes ~250 ms and must not stall the async runtime.

### 3.2 Legacy PBKDF2 path

v1 vaults used PBKDF2-HMAC-SHA256. It is retained **for reading only**.

```rust
use pbkdf2::pbkdf2_hmac;
use sha2::Sha256;

pub const LEGACY_PBKDF2_ITERS: u32 = 210_000;

pub fn derive_root_pbkdf2(passphrase: &str, salt: &[u8], iters: u32) -> RootKey {
    let mut out = [0u8; 32];
    pbkdf2_hmac::<Sha256>(passphrase.as_bytes(), salt, iters, &mut out);
    RootKey::from_bytes(out)
}
```

### 3.3 HKDF sub-keys

```rust
use hkdf::Hkdf;
use sha2::Sha256;

pub fn derive_collection_key(root: &RootKey, collection: &str) -> CollectionKey {
    let hk = Hkdf::<Sha256>::new(None, root.as_bytes());
    let info = format!("slytube/v2/collection/{collection}");
    let mut out = [0u8; 32];
    hk.expand(info.as_bytes(), &mut out).expect("32 is a valid HKDF length");
    CollectionKey::from_bytes(out)
}

pub fn derive_purpose_key(root: &RootKey, purpose: &str) -> [u8; 32] {
    let hk = Hkdf::<Sha256>::new(None, root.as_bytes());
    let mut out = [0u8; 32];
    hk.expand(format!("slytube/v2/{purpose}").as_bytes(), &mut out).unwrap();
    out
}
```

Per-collection keys mean a compromised `history` key does not expose `settings`, and a collection can be re-keyed independently.

---

## 4. Device Pairing (X25519)

Pairing lets a second device obtain the root key without the passphrase ever crossing the wire.

```
Device A (has root key)                     Device B (new)
─────────────────────────                   ─────────────────────────
                                            generate ephemeral
                                            (b_priv, b_pub)
                                            show QR / 8-word code
                                            containing b_pub + nonce
scan code → b_pub
generate ephemeral (a_priv, a_pub)
shared = X25519(a_priv, b_pub)
wrap_key = HKDF(shared, info="pair/v2",
                salt = nonce_a || nonce_b)
sealed = AES-256-GCM(wrap_key,
           root_key, aad = a_pub||b_pub)
upload {a_pub, nonce_a, sealed} ──────────► fetch
                                            shared = X25519(b_priv, a_pub)
                                            wrap_key = HKDF(...)
                                            root_key = open(sealed)
                                            verify fingerprint out-of-band
```

```rust
use x25519_dalek::{EphemeralSecret, PublicKey};

pub struct PairingOffer {
    pub public_key: [u8; 32],
    pub nonce: [u8; 16],
}

pub fn seal_root_for_peer(
    root: &RootKey,
    peer_pub: &[u8; 32],
    nonce_a: &[u8; 16],
    nonce_b: &[u8; 16],
) -> Result<SealedRoot, CryptoError> {
    let a_secret = EphemeralSecret::random_from_rng(rand::rngs::OsRng);
    let a_pub = PublicKey::from(&a_secret);
    let shared = a_secret.diffie_hellman(&PublicKey::from(*peer_pub));

    let mut salt = Vec::with_capacity(32);
    salt.extend_from_slice(nonce_a);
    salt.extend_from_slice(nonce_b);

    let hk = Hkdf::<Sha256>::new(Some(&salt), shared.as_bytes());
    let mut wrap = [0u8; 32];
    hk.expand(b"slytube/v2/pair", &mut wrap).unwrap();

    let mut aad = Vec::with_capacity(64);
    aad.extend_from_slice(a_pub.as_bytes());
    aad.extend_from_slice(peer_pub);

    let sealed = aead_encrypt(&wrap, root.as_bytes(), &aad)?;
    wrap.zeroize();

    Ok(SealedRoot { sender_public: *a_pub.as_bytes(), nonce: *nonce_a, sealed })
}
```

**Fingerprint verification.** Both devices display `SHA256(a_pub || b_pub)` rendered as six words from a fixed list. Users must confirm they match — this is what defeats an active MITM on the pairing channel; the crypto alone cannot.

---

## 5. Snapshot Protocol

### 5.1 Model

Sync is **not** an operation log. Each collection is serialised whole, encrypted, and uploaded as one versioned blob. This is a deliberate trade: it costs bandwidth on large collections but eliminates an entire class of divergence bugs, needs no server-side merge logic, and makes the wire format trivially auditable.

```
┌───────────────────────────────────────────────────────────────┐
│ PUSH                                                          │
│  1. build_snapshot(collections)   → plaintext structs         │
│  2. canonical JSON  (sorted keys, no whitespace)              │
│  3. zstd level 3                                              │
│  4. AES-256-GCM with per-collection key + AAD                 │
│  5. PUT /v2/blobs/{collection}   If-Match: <version>          │
├───────────────────────────────────────────────────────────────┤
│ PULL                                                          │
│  1. GET /v2/manifest             → versions + digests         │
│  2. skip collections whose version == local                   │
│  3. GET /v2/blobs/{collection}                                │
│  4. decrypt + verify AAD, decompress, parse                   │
│  5. apply via db::*::persist_sync / apply_sync                │
└───────────────────────────────────────────────────────────────┘
```

### 5.2 Envelope

```rust
#[derive(Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SyncEnvelope {
    pub format: u8,              // 2
    pub collection: String,
    pub version: u64,            // monotonic, per collection
    pub device_id: String,       // public key fingerprint, hex
    pub kdf: KdfDescriptor,
    pub nonce: String,           // base64, 12 bytes
    pub ciphertext: String,      // base64, includes 16-byte GCM tag
    pub compression: String,     // 'zstd' | 'none'
    pub created_at: Option<i64>, // omitted in 'enhanced' privacy mode
}

#[derive(Serialize, Deserialize)]
#[serde(tag = "algo", rename_all = "camelCase")]
pub enum KdfDescriptor {
    #[serde(rename = "argon2id")]
    Argon2id { salt: String, m: u32, t: u32, p: u32 },
    #[serde(rename = "pbkdf2-sha256")]
    Pbkdf2 { salt: String, iterations: u32 },
}
```

### 5.3 AAD binding

The AAD is what stops a server from swapping a `settings` blob for a `history` blob, or replaying an old version:

```rust
fn build_aad(collection: &str, version: u64, device_id: &str, format: u8) -> Vec<u8> {
    format!("slytube|v{format}|{collection}|{version}|{device_id}").into_bytes()
}
```

Any mismatch fails the GCM tag check, and decryption returns a generic error — never a hint about *which* field was wrong.

### 5.4 Encrypt / decrypt

```rust
use aes_gcm::{Aes256Gcm, Nonce, KeyInit, aead::{Aead, Payload}};

pub fn aead_encrypt(key: &[u8; 32], plaintext: &[u8], aad: &[u8]) -> Result<Vec<u8>, CryptoError> {
    let cipher = Aes256Gcm::new(key.into());

    let mut nonce_bytes = [0u8; 12];
    rand::RngCore::fill_bytes(&mut rand::rngs::OsRng, &mut nonce_bytes);
    let nonce = Nonce::from_slice(&nonce_bytes);

    let mut out = cipher
        .encrypt(nonce, Payload { msg: plaintext, aad })
        .map_err(|_| CryptoError::Encrypt)?;

    // Prepend the nonce so the blob is self-contained.
    let mut blob = nonce_bytes.to_vec();
    blob.append(&mut out);
    Ok(blob)
}

pub fn aead_decrypt(key: &[u8; 32], blob: &[u8], aad: &[u8]) -> Result<Vec<u8>, CryptoError> {
    if blob.len() < 12 + 16 {
        return Err(CryptoError::Decrypt);
    }
    let (nonce_bytes, ct) = blob.split_at(12);
    Aes256Gcm::new(key.into())
        .decrypt(Nonce::from_slice(nonce_bytes), Payload { msg: ct, aad })
        .map_err(|_| CryptoError::Decrypt)   // deliberately opaque
}
```

**Nonce policy.** 96-bit random nonces with a fresh per-collection key. Collision risk is negligible at realistic blob counts (<2⁻⁷⁰ at 10⁶ blobs), and re-keying on passphrase change resets the budget.

### 5.5 Canonical serialisation

Before encryption, JSON is canonicalised: object keys sorted lexicographically, no insignificant whitespace, integers never rendered in exponent form, and `null`-valued optional fields omitted. Without this, two devices with identical data produce different ciphertexts and different digests, defeating the manifest's skip-if-unchanged optimisation.

### 5.6 Versioning & conflicts

Each collection carries a `version` incremented on every successful push. Uploads use `If-Match: <expected-version>`:

| Server response | Client action |
|---|---|
| `204 No Content` | Push accepted, bump local version |
| `412 Precondition Failed` | Remote moved ahead → pull, merge, retry once |
| `409 Conflict` | Version regression detected → surface to the user |
| `429` | Exponential backoff (2 s, 8 s, 30 s) |

Merge is **per-collection**, using the same rules the `persist_sync` / `apply_sync` commands implement (see [02-tauri-commands.md](02-tauri-commands.md)):

| Collection | Strategy |
|---|---|
| `subscriptions` | Union by `channelId`; tombstones win only if newer |
| `playlists` | Union of items by `playlistItemId`; header fields last-write-wins on `lastUpdatedAt` |
| `history` | Per-video `MAX(timeWatched)`, `MAX(watchProgress)` |
| `playback_speeds` | Last-write-wins per `videoId` |
| `profiles` | Last-write-wins on `updatedAt`; subscription sets unioned |
| `sessions` | Never merged — most recent device wins wholesale |
| `settings` | Last-write-wins per key, with a device-local exclusion list |

---

## 6. Collections

```rust
pub const ALL_COLLECTIONS: &[&str] = &[
    "subscriptions",
    "playlists",
    "history",
    "playback_speeds",
    "profiles",
    "sessions",
    "settings",
];
```

| Collection | Source tables | Typical size | Sensitivity | Enhanced mode |
|---|---|---|---|---|
| `subscriptions` | `profile_subscriptions` | 1–5k rows | Medium | Synced |
| `playlists` | `playlists`, `playlist_videos` | 100–50k rows | Medium | Synced |
| `history` | `history` | 1k–100k rows | **High** | Synced, timestamps coarsened |
| `playback_speeds` | `settings` (`speedOverrides`) | <1k | Low | Synced |
| `profiles` | `profiles` | <50 | Low | Synced |
| `sessions` | `tab_sessions` | <100 | **High** (reveals current activity) | **Excluded** |
| `settings` | `settings` | <300 | Mixed | Synced minus device-local keys |

Never synced under any mode: `subscription_cache` (regenerable), `search_history` (high-signal, low-value), `downloads` (device-local paths), `migration_state`, PoTokens, and window bounds.

### 6.1 Snapshot shape

```rust
#[derive(Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Snapshot {
    pub schema: u32,
    pub collections: BTreeMap<String, serde_json::Value>,
    pub generated_at: Option<i64>,
}

pub async fn build_snapshot(
    pool: &SqlitePool,
    wanted: &[String],
    privacy: PrivacyMode,
) -> Result<Snapshot, AppError> {
    let mut collections = BTreeMap::new();

    for name in wanted {
        if !privacy.allows(name) {
            continue;
        }
        let value = match name.as_str() {
            "subscriptions"   => json!(db::profiles::all_subscriptions(pool).await?),
            "playlists"       => json!(db::playlists::find_all_with_videos(pool).await?),
            "history"         => json!(privacy.scrub_history(db::history::find_all(pool).await?)),
            "playback_speeds" => json!(db::settings::find_one(pool, "speedOverrides").await?),
            "profiles"        => json!(db::profiles::find_all(pool).await?),
            "sessions"        => json!(db::tab_sessions::find_all(pool).await?),
            "settings"        => json!(privacy.filter_settings(db::settings::find_all(pool).await?)),
            other => return Err(AppError::Invalid(format!("unknown collection: {other}"))),
        };
        collections.insert(name.clone(), value);
    }

    Ok(Snapshot {
        schema: 2,
        collections,
        generated_at: privacy.include_timestamps().then(now_ms),
    })
}
```

### 6.2 Size guards

`history` dominates. Guards:

1. Cap at the most recent 50 000 entries (configurable).
2. zstd-3 typically achieves ~8:1 on history JSON.
3. Reject any single blob over 32 MiB; the UI then suggests pruning.
4. Push only collections whose canonical digest changed since the last successful push.

---

## 7. Legacy Decryption (Migration)

v1 blobs (Electron era) must remain readable. Differences:

| Aspect | v1 (legacy) | v2 (current) |
|---|---|---|
| Password KDF | PBKDF2-HMAC-SHA256, 100 000 iters | Argon2id 64 MiB / t=3 / p=4 |
| Root → data key | Root used directly | HKDF per collection |
| Cipher | AES-256-GCM | AES-256-GCM |
| AAD | none | `slytube\|v2\|<collection>\|<version>\|<device>` |
| Nonce | 12B, prefixed | 12B, prefixed |
| Compression | none | zstd |
| Envelope | `{ v, salt, iv, data }` | `SyncEnvelope` |

```rust
#[derive(Deserialize)]
struct LegacyEnvelope {
    v: u8,               // 1
    salt: String,        // base64
    iv: String,          // base64
    data: String,        // base64 ciphertext || tag
    #[serde(default = "default_legacy_iters")]
    iterations: u32,
}

pub fn decrypt_legacy(passphrase: &str, raw: &[u8]) -> Result<Vec<u8>, CryptoError> {
    let env: LegacyEnvelope = serde_json::from_slice(raw).map_err(|_| CryptoError::Format)?;
    if env.v != 1 {
        return Err(CryptoError::UnsupportedVersion(env.v));
    }

    let salt = b64.decode(&env.salt).map_err(|_| CryptoError::Format)?;
    let iv   = b64.decode(&env.iv).map_err(|_| CryptoError::Format)?;
    let ct   = b64.decode(&env.data).map_err(|_| CryptoError::Format)?;
    if iv.len() != 12 {
        return Err(CryptoError::Format);
    }

    let root = derive_root_pbkdf2(passphrase, &salt, env.iterations);

    // v1 had no AAD and used the root key directly.
    Aes256Gcm::new(root.as_bytes().into())
        .decrypt(Nonce::from_slice(&iv), Payload { msg: &ct, aad: b"" })
        .map_err(|_| CryptoError::Decrypt)
}

fn default_legacy_iters() -> u32 { 100_000 }
```

### 7.1 Migration flow

```rust
pub async fn migrate_legacy_vault(
    app: &AppHandle,
    pool: &SqlitePool,
    passphrase: &str,
) -> Result<MigrationSummary, AppError> {
    let mut summary = MigrationSummary::default();

    for collection in ALL_COLLECTIONS {
        let Some(raw) = fetch_legacy_blob(collection).await? else { continue };

        // 1. Read with the old scheme.
        let plaintext = match decrypt_legacy(passphrase, &raw) {
            Ok(p) => p,
            Err(e) => { summary.failed.push(((*collection).into(), e.to_string())); continue; }
        };

        // 2. Apply locally first — data safety beats re-upload.
        apply_collection(pool, collection, &plaintext).await?;

        // 3. Re-encrypt under v2 and push.
        let vault = current_vault(app).await?;
        let key = derive_collection_key(&vault.root, collection);
        let compressed = zstd::encode_all(plaintext.as_slice(), 3)?;
        let aad = build_aad(collection, 1, &vault.device_id, 2);
        let blob = aead_encrypt(key.as_bytes(), &compressed, &aad)?;
        push_blob(collection, 1, blob).await?;

        summary.migrated.push((*collection).into());
    }

    // Legacy blobs are retained for 30 days, then deleted.
    schedule_legacy_cleanup(app, 30).await?;
    Ok(summary)
}
```

Migration is **read-legacy / write-v2**; v1 blobs are never written again. A device still on v1 will keep reading its own blobs until upgraded, so the retention window matters.

### 7.2 Version detection

```rust
pub fn detect_format(raw: &[u8]) -> Result<u8, CryptoError> {
    let v: serde_json::Value = serde_json::from_slice(raw).map_err(|_| CryptoError::Format)?;
    if let Some(f) = v.get("format").and_then(|x| x.as_u64()) { return Ok(f as u8); }   // v2
    if let Some(f) = v.get("v").and_then(|x| x.as_u64())      { return Ok(f as u8); }   // v1
    Err(CryptoError::Format)
}
```

---

## 8. Privacy Modes

```rust
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum PrivacyMode {
    Standard,
    Enhanced,
}
```

| Behaviour | `standard` | `enhanced` |
|---|---|---|
| Collections synced | All seven | `sessions` excluded; `history` opt-in |
| Envelope `createdAt` | Present | Omitted |
| History timestamps | Exact ms | Rounded to the hour |
| History `watchProgress` | Exact | Rounded to 10 s |
| Video titles/authors in history | Included | **Stripped** — ids only |
| Blob padding | None | Padded to the next 4 KiB boundary |
| Push scheduling | Immediate on change | Jittered 0–15 min |
| Device name | Hostname | `device-<8 hex>` |
| Settings synced | All non-device-local | Curated allow-list only |
| Legacy PBKDF2 vaults | Readable | Readable, forced re-key on unlock |
| Direct-connection fallback | Permitted if enabled | **Never** |

### 8.1 Implementation

```rust
impl PrivacyMode {
    pub fn allows(&self, collection: &str) -> bool {
        match self {
            PrivacyMode::Standard => true,
            PrivacyMode::Enhanced => collection != "sessions",
        }
    }

    pub fn include_timestamps(&self) -> bool {
        matches!(self, PrivacyMode::Standard)
    }

    pub fn scrub_history(&self, mut entries: Vec<HistoryEntry>) -> Vec<HistoryEntry> {
        if matches!(self, PrivacyMode::Standard) {
            return entries;
        }
        const HOUR_MS: i64 = 3_600_000;
        for e in &mut entries {
            e.time_watched   = (e.time_watched / HOUR_MS) * HOUR_MS;
            e.watch_progress = (e.watch_progress / 10.0).round() * 10.0;
            e.title.clear();
            e.author = None;
            // author_id is retained: it drives channel statistics and is
            // already derivable from the subscriptions collection.
        }
        entries
    }

    /// Length hiding: blob size otherwise leaks how much history a user has.
    pub fn pad(&self, mut data: Vec<u8>) -> Vec<u8> {
        if matches!(self, PrivacyMode::Standard) {
            return data;
        }
        const BLOCK: usize = 4096;
        let target = data.len().div_ceil(BLOCK) * BLOCK;
        let pad = target - data.len();
        data.extend(std::iter::repeat(0u8).take(pad));
        data.extend_from_slice(&(pad as u32).to_le_bytes());   // trailer
        data
    }
}
```

Padding is applied **after compression and before encryption**, and the 4-byte trailer records the pad length so the reader can trim exactly.

### 8.2 Device-local settings

Never synced in either mode:

```rust
pub const DEVICE_LOCAL_SETTINGS: &[&str] = &[
    "bounds", "downloadPath", "proxyConfig", "ytdlpOverridePath",
    "hardwareAcceleration", "displayScale", "audioDevice",
    "syncDeviceId", "syncPassphraseSalt", "lastSyncVersions",
];
```

---

## 9. Threat Model

| Adversary | Capability | Mitigation |
|---|---|---|
| Passive server operator | Reads all blobs | AES-256-GCM; server holds no keys |
| Active server operator | Reorders/replays/substitutes blobs | AAD binds collection + version + device; `If-Match` versioning |
| Network attacker | MITM | TLS + rustls; certificate validation never disabled |
| Pairing MITM | Intercepts pairing channel | Out-of-band six-word fingerprint confirmation |
| Offline passphrase cracking | Steals blobs, brute-forces | Argon2id 64 MiB (~250 ms/guess); 12+ char minimum enforced |
| Local malware | Reads process memory | `ZeroizeOnDrop`, 30-min auto-lock; not fully mitigable |
| Traffic analysis | Infers activity from blob size/timing | `enhanced`: padding + jittered scheduling |

**Explicit non-goals:** the server *does* learn which collections exist, roughly how often a device syncs, and each device's IP. Fully hiding those requires a private-information-retrieval design that is out of scope.

---

## 10. Error Handling

```rust
#[derive(Debug, thiserror::Error)]
pub enum CryptoError {
    #[error("key derivation failed: {0}")]     Kdf(String),
    #[error("encryption failed")]              Encrypt,
    #[error("decryption failed")]              Decrypt,   // never says why
    #[error("malformed envelope")]             Format,
    #[error("unsupported format version: {0}")] UnsupportedVersion(u8),
    #[error("vault is locked")]                Locked,
}
```

`Decrypt` is intentionally uninformative to the caller. Internally, `tracing` records whether the failure was a tag mismatch, an AAD mismatch, or a length error — never the key, nonce, or plaintext. The UI maps `Decrypt` to a single actionable message: *"Wrong passphrase, or this data was created by a newer version of SlyTube."*
