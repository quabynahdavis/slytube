# External Binaries (Sidecars)

This directory holds the platform-specific `yt-dlp` binaries that are bundled
with OpenTubeX as a Tauri **sidecar**.

The sidecar is declared in `src-tauri/tauri.conf.json`:

```json
"bundle": {
  "externalBin": ["binaries/yt-dlp"]
}
```

## Naming Convention

Tauri resolves sidecars by appending the **Rust target triple** to the base
name. Every binary you place here MUST follow this pattern:

```
yt-dlp-<target-triple>[.exe]
```

Run the following to discover the target triple of your current machine:

```bash
rustc -vV | grep host
```

### Expected filenames

| Platform | Architecture | Filename |
| --- | --- | --- |
| Linux | x86_64 | `yt-dlp-x86_64-unknown-linux-gnu` |
| Linux | aarch64 | `yt-dlp-aarch64-unknown-linux-gnu` |
| macOS | Intel | `yt-dlp-x86_64-apple-darwin` |
| macOS | Apple Silicon | `yt-dlp-aarch64-apple-darwin` |
| Windows | x86_64 | `yt-dlp-x86_64-pc-windows-msvc.exe` |
| Windows | aarch64 | `yt-dlp-aarch64-pc-windows-msvc.exe` |

> A build will fail if the binary for the target you are compiling for is
> missing from this directory.

## Downloading the binaries

Official releases: <https://github.com/yt-dlp/yt-dlp/releases/latest>

### Linux (x86_64)

```bash
curl -L -o yt-dlp-x86_64-unknown-linux-gnu \
  https://github.com/yt-dlp/yt-dlp/releases/latest/download/yt-dlp_linux
chmod +x yt-dlp-x86_64-unknown-linux-gnu
```

### Linux (aarch64)

```bash
curl -L -o yt-dlp-aarch64-unknown-linux-gnu \
  https://github.com/yt-dlp/yt-dlp/releases/latest/download/yt-dlp_linux_aarch64
chmod +x yt-dlp-aarch64-unknown-linux-gnu
```

### macOS (universal build works for both arches)

```bash
curl -L -o yt-dlp-aarch64-apple-darwin \
  https://github.com/yt-dlp/yt-dlp/releases/latest/download/yt-dlp_macos
chmod +x yt-dlp-aarch64-apple-darwin
cp yt-dlp-aarch64-apple-darwin yt-dlp-x86_64-apple-darwin
```

### Windows (x86_64)

```powershell
Invoke-WebRequest -Uri https://github.com/yt-dlp/yt-dlp/releases/latest/download/yt-dlp.exe `
  -OutFile yt-dlp-x86_64-pc-windows-msvc.exe
```

## Runtime Dependency: FFmpeg

`yt-dlp` requires **FFmpeg** to merge separate audio/video streams and to
transcode. FFmpeg is *not* bundled here — OpenTubeX resolves it from the
system `PATH`. Users should install it via their package manager
(`apt`, `brew`, `winget`, `pacman`, …).

If you prefer to bundle FFmpeg too, add `binaries/ffmpeg` to `externalBin`
and drop the target-triple-suffixed binaries in this directory following the
exact same naming convention.

## Version Control

The binaries themselves are intentionally **not** committed to git — they are
large and platform-specific. Only this `README.md` is tracked. See the
`.gitignore` in this directory.
