# Arch packaging

Source-built `lap` recipe for x86_64. `pkgver` stays `0.3.2`; `pkgrel` is `2` for the Arch updater and shutdown fixes. The recipe source commit is `25f41f357e05df5c8123f78b4f5b111803c50dc7` (`v0.3.2-25-g25f41f35`), not the `v0.3.2` tag. Four native submodules are pinned tarballs, not git checkouts. Models and the x86_64 FFmpeg sidecars are checksummed `source=` entries staged into `src-tauri/resources/` before `cargo tauri build --bundles deb`. `package()` extracts only that DEB's `data.tar.*`.

This is not an AUR publication. `yay -S lap` will not work until the recipe is published. Install a locally built package with `pacman -U`.

## Local build

From this directory, as a non-root user with the listed `makedepends` installed:

```bash
makepkg -s
```

Inspect the new package before installation. A rebuild is required after changing `PKGBUILD` or `disable-updater.patch`; the older package file in this checkout does not contain the updater fix.

On 2026-10-12 a local `0.3.2-2` package was built from copied prepared sources using `makepkg --noextract --nodeps`. After installing it, the tester reported that Lap opened, closed with the main window's X, and reopened successfully. This confirms that user-reported close/reopen path only. A later clean, dependency-checking build of the final recipe in a disposable Arch container produced `lap-0.3.2-2-x86_64.pkg.tar.zst` (SHA-256 `551bbccbf54e54ef8d5887470bbbfcb9424238ebcfa9ddb3d1c7b379226130d8`). All 16 recipe source checksums passed, `.SRCINFO` matched, and namcap reported no errors. The clean artifact has not been installed or exercised in the GUI; AI inference, FFmpeg-backed video, updater UI, and package removal remain unverified for that artifact. Remote CI has not run.

`libappindicator` is the Arch package name; it provides `libappindicator-gtk3`. Do not depend on `libayatana-appindicator`.

## CI validation and release refresh

`.github/workflows/release-arch.yml` builds the recipe in an Arch container as a non-root user on pull requests that change packaging, on manual dispatch, and on `v*` tags. It compares generated `.SRCINFO` with the committed file, runs namcap, and inspects the package with pacman. It uploads source identity, SHA-256, and inspection logs, but withholds the binary package while the bundled model and FFmpeg distribution questions remain open. This validation does not publish to the AUR or attach an Arch package to a GitHub release.

The package source is the immutable `_commit` in `PKGBUILD`, not necessarily the workflow revision. On a release tag, the workflow requires the tag to equal `v$pkgver` and `_commit` to be an ancestor of the tagged revision. Checkout includes full history for that ancestry check; `source-identity.txt` records the source and workflow revisions separately. The recipe does not need to contain its own commit SHA.

To prepare a later release:

1. Finish and review the application source in commit A.
2. In a later packaging commit B, set `_commit` to A, update `pkgver`/`pkgrel`, review resource pins and patches, and update the source archive checksum. Regenerate `.SRCINFO` with `makepkg --printsrcinfo > .SRCINFO` and test the recipe.
3. Tag B as `v$pkgver`. The package builds A with the recipe's explicit patches; it does not claim to build the entire tree at B. Keep B packaging-only; if application code changes before tagging, select the new source revision and refresh its pin and checksum first.

A green workflow and installed-app check are still required before offering an AUR package.

Runtime `depends` were refined from `ldd` and namcap, not from the initial plan list alone. `openssl` is a direct `DT_NEEDED` entry and is not pulled in by `webkit2gtk-4.1`. `hicolor-icon-theme` is required for the installed hicolor icon. `glibc`, `glib2`, `libgcc`, and `libstdc++` are direct namcap dependencies. `libappindicator` stays even though it is not a `DT_NEEDED` entry: the tray icon dlopens it. `librsvg` stays because `gtk3` uses it for SVG icons and the parent plan requires it; namcap may still call it unused.

## Package-managed updates

The recipe applies `disable-updater.patch` only inside the source tree extracted by `makepkg`. Its frontend build flag hides the update button and skips automatic and manual update calls; its Tauri capability change removes the updater permission. The plugin remains compiled but the Arch frontend cannot invoke it. Upgrades must go through pacman. Other distribution builds retain their existing updater behavior. The previously built `lap-0.3.2-1-x86_64.pkg.tar.zst` predates the shutdown fix and must not be used to retest closing the main window.

The `fix-optional-aptabase-exit.patch` guards the exit-time event flush with the same `APTABASE_KEY` condition used to register Aptabase. Without a key, the plugin state is absent and calling its flush method panics when the app exits. This fix is also applied to the working-tree `src-tauri/src/main.rs` for upstream review; the recipe patch is needed because the pinned source archive predates that edit.

# Build inputs

The rest of this file is the resource contract consumed by the recipe. The initial Arch recipe remains **x86_64-only**; preserving the existing Linux aarch64 downloader is not Arch ARM support.

From the repository root:

```bash
bash scripts/download_models.sh
bash scripts/download_ffmpeg_sidecar.sh linux x86_64
# Existing upstream Linux target, not an Arch packaging target:
bash scripts/download_ffmpeg_sidecar.sh linux aarch64
```

Destinations remain script-relative, independent of the caller's working directory:

- Models: `src-tauri/resources/models/<destination>`.
- Sidecars: `src-tauri/resources/ffmpeg/<filename>`.

The Bash model downloader checks all five files; the Bash sidecar downloader checks both files for the selected Linux target. They verify SHA-256 on every invocation, including cached or externally pre-staged files. A corrupt cache fails without overwriting it: remove the rejected file explicitly, then rerun. New downloads use unique sibling `.part.*` files, fail on HTTP errors, and are verified before atomic rename. Temporary files are removed on normal failure/interrupt; SIGKILL cannot run cleanup, but abandoned `.part.*` files are never treated as cached inputs. Model downloads are installed mode 644 and sidecars mode 755; verified cached sidecars gain execute permission. SHA-256 requires `sha256sum` or macOS `shasum -a 256`, alongside Bash, curl, and standard file utilities. Optional `HF_TOKEN` remains supported; these public inputs were downloaded without credentials.

Phase 2 can stage the exact files below before invoking these scripts; a valid pre-staged build requires no model/sidecar network requests. The existing Bash release/PR jobs invoke the model downloader even on a CI cache hit. PowerShell downloaders and non-Linux sidecar checksums are **outside this contract** and retain their previous integrity limitations. This is acquisition verification, not a claim that the entire application build is hermetic or independently reproducible.

## Model provenance

Hugging Face revision URLs select immutable Git commits, not `main`. Metadata was resolved with `GET https://huggingface.co/api/models/<repo>/revision/<commit>?blobs=true` on 2026-10-06. The four ONNX digests match the revision's Git LFS `sha256` identifiers and independently downloaded bytes. The tokenizer is an ordinary Git blob: its downloaded Git blob ID is `564c0ebd5ce29c4ee4864004aee693deadd3128c`; SHA-256 was computed independently over those bytes. These hashes identify the publisher-selected content; they are not independent model-quality or publisher-authenticity attestations.

| Destination | Pinned source URL | SHA-256 |
|---|---|---|
| `tokenizer.json` | `https://huggingface.co/openai/clip-vit-base-patch32/resolve/3d74acf9a28c67741b2f4f2ea7635f0aaf6f0268/tokenizer.json` | `b556ac8c99757ffb677208af34bc8c6721572114111a6e0aaf5fa69ff0b8d842` |
| `text_model.onnx` | `https://huggingface.co/Xenova/clip-vit-base-patch32/resolve/d15189d7028b43f1d3e65039190477f6af591c2a/onnx/text_model_quantized.onnx` | `73baab855d406190da9faa498cfedf65f15cf309f4cc7385b7b032e6d08e5c3a` |
| `vision_model.onnx` | `https://huggingface.co/Xenova/clip-vit-base-patch32/resolve/d15189d7028b43f1d3e65039190477f6af591c2a/onnx/vision_model_quantized.onnx` | `583fd1110a514667812fee7d684952aaf82a99b959760c8d7dca7e0ab9839299` |
| `det_500m.onnx` | `https://huggingface.co/deepghs/insightface/resolve/4e1f33d3fe0e50a0945f3a53ab94ae8977ae7ddb/buffalo_s/det_500m.onnx?download=true` | `5e4447f50245bbd7966bd6c0fa52938c61474a04ec7def48753668a9d8b4ea3a` |
| `w600k_mbf.onnx` | `https://huggingface.co/deepghs/insightface/resolve/4e1f33d3fe0e50a0945f3a53ab94ae8977ae7ddb/buffalo_s/w600k_mbf.onnx?download=true` | `9cc6e4a75f0e2bf0b1aed94578f144d15175f357bdc05e815e5c4a02b319eb4f` |

OpenAI supplies the CLIP tokenizer; Xenova supplies quantized ONNX conversions of OpenAI CLIP ViT-B/32; deepghs distributes the InsightFace Buffalo-S models. The [pinned deepghs model card](https://huggingface.co/deepghs/insightface/blob/4e1f33d3fe0e50a0945f3a53ab94ae8977ae7ddb/README.md) declares `license: other`, a distribution disclaimer, and attributes the models to InsightFace. [InsightFace's upstream license policy](https://github.com/deepinsight/insightface#license) distinguishes MIT code from models available for non-commercial research only. Hash verification does **not** establish unrestricted model use or redistribution rights; resolve this licensing risk before publishing a package.

## Linux FFmpeg/FFprobe provenance

Origin: [`julyx10/lap-binaries`, release `ffmpeg-8.1`](https://github.com/julyx10/lap-binaries/releases/tag/ffmpeg-8.1), release ID `309197994`. Digests were cross-checked against both GitHub's release asset metadata and the release's `SHA256SUMS.txt`, then matched against downloaded bytes on 2026-10-06.

The release API reports `immutable: false`: the release tag/name is **not** an immutable binary identity. The Bash Linux downloader therefore uses the numeric asset endpoint with `Accept: application/octet-stream`, plus a committed digest. Deleting/re-uploading a same-named asset does not retarget the numeric ID; deletion can make it unavailable. A changed response cannot silently pass the pinned digest. These are content-bound asset identities, not a promise of permanent availability or a signed reproducible build.

| Filename / destination | Numeric asset URL | SHA-256 |
|---|---|---|
| `ffmpeg-x86_64-unknown-linux-gnu` | `https://api.github.com/repos/julyx10/lap-binaries/releases/assets/405019648` | `caf3206f8858cfb3e87dcc02f1bf5d514790ad7f64b6c49a6da511b3e3033628` |
| `ffprobe-x86_64-unknown-linux-gnu` | `https://api.github.com/repos/julyx10/lap-binaries/releases/assets/405019643` | `931f0eabf33462953978eea5d0cb92edc60ec4e33eaa1f0b5845537636ab190d` |
| `ffmpeg-aarch64-unknown-linux-gnu` | `https://api.github.com/repos/julyx10/lap-binaries/releases/assets/405019654` | `21ba8322e97370ecd081e6b128a99b1323f73cec7fa631829a7e2e7e67bcbe0f` |
| `ffprobe-aarch64-unknown-linux-gnu` | `https://api.github.com/repos/julyx10/lap-binaries/releases/assets/405019651` | `1856b1c9cedc0be91cca5c49148d2faa319afd09837c91e36ea163baab5cd24e` |

Asset creation times match the successful [upload workflow run 24919827459](https://github.com/julyx10/lap-binaries/actions/runs/24919827459), whose recorded source revision is `d5538364ae49c3873d7920998572098d239c8afb`. Its [acquisition script](https://github.com/julyx10/lap-binaries/blob/d5538364ae49c3873d7920998572098d239c8afb/scripts/download-ffmpeg.py) downloads Linux binaries from Martin Riedl's rolling `https://ffmpeg.martin-riedl.de/redirect/latest/linux/{amd64,arm64}/release/{ffmpeg,ffprobe}.zip` URLs, renames extracted files, and generates the checksums. The release tag points at an older source revision; the upload run, not that tag's script, explains the current assets. This is an observed provenance association, not cryptographic build attestation, and the upstream rolling downloads cannot reproduce the release just from the acquisition script.

Both downloaded x86_64 executables report FFmpeg 8.1 with `--enable-nonfree --enable-gpl` and Martin Riedl branding. This is a material redistribution/licensing risk, not resolved by SHA-256 or the release title. Review before packaging/publishing; this phase does not replace the existing sidecars or claim legal clearance. aarch64 bytes were downloaded and hashed, not executed on this x86_64 host. Numeric API acquisition also depends on GitHub availability and unauthenticated rate limits.

### Publication evidence still needed

An AUR submission would carry the recipe and metadata, not a prebuilt Lap package or the source assets. Running that recipe downloads the pinned assets and bundles them into the user's local package. Uploading the resulting binary through CI or a GitHub release would distribute those bundled bytes directly. Review both paths against the exact asset terms before enabling distribution.

The inspected 2026-10-07 package contains only Lap's GPL license and `package-source-license` under `/usr/share/licenses/lap`; it has no separate FFmpeg source/configuration or model terms. [FFmpeg's own legal checklist](https://ffmpeg.org/legal.html) calls for corresponding source and build details for redistributed binaries, and its LGPL example excludes both `--enable-gpl` and `--enable-nonfree`. The pinned binaries report both flags, so obtain an appropriate redistributable FFmpeg build and its matching source/build record before publishing a binary artifact. The AUR recipe's direct download of these assets also needs review.

The pinned face-model files are hosted by deepghs, while [InsightFace's policy](https://github.com/deepinsight/insightface#license) limits its pretrained models to non-commercial research and points to a licensing contact for open-sourced face-recognition packs. The pinned model card labels its terms `other`; no separate authorization for these two exact ONNX files is present in the checkout. Confirm their rights and provide the applicable terms before publication. This records missing evidence and does not decide whether a particular distribution is permitted.

## ONNX Runtime acquisition audit

`src-tauri/Cargo.toml` retains `ort = "=2.0.0-rc.10"`, default features disabled, with `std`, `ndarray`, `download-binaries`, and `copy-dylibs`. `src-tauri/Cargo.lock` locks both `ort` and `ort-sys` to `2.0.0-rc.10`. The locked `ort-sys` registry archive checksum is `e2aba9f5c7c479925205799216e7e5d07cc1d4fa76ea8058c60a9a30f6a4e890`; that identifies the **crate**, not the separately downloaded runtime archive.

Inspection of that crate's `build.rs` and embedded `dist.txt` establishes:

- Runtime version: 1.22.0. This application's features select the `none` distribution.
- `find_dist` reads URL **and SHA-256** from the embedded distribution table. `fetch_file` uses HTTPS, disallows redirects, and `verify_file` hashes fresh archive bytes before extraction. It is inaccurate to describe fresh default ONNX downloads as unchecked merely because makepkg cannot see them.
- `copy-dylibs` stages runtime libraries adjacent to Cargo outputs (including deps/examples directories); it is not an additional checksum check.

| Target | Embedded archive URL | Embedded SHA-256 |
|---|---|---|
| x86_64 Linux | `https://cdn.pyke.io/0/pyke:ort-rs/ms@1.22.0/x86_64-unknown-linux-gnu.tgz` | `ED1716DE95974BF47AB0223CA33734A0B5A5D09A181225D0E8ED62D070AEA893` |
| aarch64 Linux (existing upstream target) | `https://cdn.pyke.io/0/pyke:ort-rs/ms@1.22.0/aarch64-unknown-linux-gnu.tgz` | `24E4760207136FC50B854BB5012AB81DE6189039CF6D4FD3F5B8D3DB7E929F1E` |

Remaining limitations:

1. The CDN URL is versioned, not independently proven immutable. The crate's committed digest constrains accepted fresh bytes; `Cargo.lock` alone is not evidence of runtime binary integrity or an independently attested build.
2. An existing extracted `onnxruntime` directory is reused **without rehashing its contents**. On Linux the cache is `$XDG_CACHE_HOME/dfbin/<target>/<hash>/onnxruntime` when XDG_CACHE_HOME is absolute; otherwise it is `$HOME/.cache/ort.pyke.io/dfbin/<target>/<hash>/onnxruntime`. The hash in the directory name does not authenticate files placed there. Use an isolated empty cache for clean-build verification; no cache-hardening change is included here.
3. A suitable `libonnxruntime` found via pkg-config takes precedence; `ORT_LIB_LOCATION` can select supplied libraries, and `ORT_SKIP_DOWNLOAD`/`CARGO_NET_OFFLINE=true` skip the default fetch. Supplied/system files are not authenticated by the embedded archive hash. Packaging must control these inputs/environment rather than assume the lockfile forces bundled acquisition.
4. ONNX acquisition remains inside the crate build script, not makepkg's `source=()` checksum contract. This phase audits that behavior; it does not change features, vendored code, or library sourcing, and does not claim to have separately exercised ONNX corruption rejection.

## Refreshing checksums

Never replace a hash simply because verification failed. Confirm the intended publisher, model conversion/variant, release, supported target, and licensing first.

1. Models: query the intended repository revision via the Hugging Face API with `blobs=true`; record the full Git commit. Build `/resolve/<commit>/<path>` URLs, download to a separate directory with `curl -fL`, and compute `sha256sum` (or `shasum -a 256`). Compare ONNX hashes with revision LFS SHA-256 metadata. For ordinary Git files, compare `git hash-object <file>` with metadata `blobId` as well as recording SHA-256. Do not use the Git blob SHA-1 or an opaque HTTP ETag as the content SHA-256.
2. Sidecars: inspect `https://api.github.com/repos/julyx10/lap-binaries/releases/tags/<tag>` for numeric release/asset identities, digests, and mutability. Compare `SHA256SUMS.txt` and asset metadata; download each asset via its numeric URL with `curl -fL -H 'Accept: application/octet-stream'`, hash independently, and inspect the native executables' `-version` output. Record the upload workflow source and original provider when available. Do not silently follow a replaced release asset or an upstream `latest` URL.
3. Update the Bash URL/asset/digest literals and this table together. Update `contrib/aur/PKGBUILD` `source`/`sha256sums` and regenerate `.SRCINFO`. Do not refresh the unchanged PowerShell or non-Linux contract implicitly.
4. Run `python -m unittest discover -s scripts/tests -v` and `bash -n scripts/download_models.sh scripts/download_ffmpeg_sidecar.sh`. Copy the scripts into an isolated empty `scripts/` layout and run them against upstream, then rerun with valid pre-staged files and unavailable curl. Exercise corrupt cached and successful-response bytes, as well as HTTP failure, before accepting the new input contract. Keep real user resources untouched.
5. If upgrading `ort`, inspect the newly locked `ort-sys` archive's checksum, distribution table, fresh verification and cache/override branches; update this audit rather than assuming old behavior survives an upgrade. Explicit human review of provenance, licensing, and residual acquisition risk is required before the next packaging phase.
