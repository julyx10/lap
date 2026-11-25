# SignPath

SignPath signs the Windows MSI and its embedded `Lap.exe`, then the workflow
regenerates Tauri's updater signature over the signed MSI. Third-party binaries
are excluded from signing with Lap's certificate.

## Configuration

In SignPath project `lap`, configure artifact slug `windows-msi` using
[`windows-msi.xml`](windows-msi.xml). The outer ZIP matches GitHub's artifact
archive. Link the GitHub.com trusted build system and authorize the CI user as a
submitter for `test-signing`, using the self-signed test certificate.

GitHub Actions settings:

| Type | Name | Value |
| --- | --- | --- |
| Variable | `SIGNPATH_ORGANIZATION_ID` | SignPath organization ID |
| Variable | `SIGNPATH_PROJECT_SLUG` | `lap` |
| Variable | `SIGNPATH_TEST_SIGNING_POLICY_SLUG` | `test-signing` |
| Secret | `SIGNPATH_API_TOKEN` | CI user's API token |

The existing `TAURI_SIGNING_PRIVATE_KEY` secret is also required.

## Usage

Run **Release Windows** with `build_target: x64`, `arm64`, or `all`:

| `signing_mode` | Behavior |
| --- | --- |
| `prepare` | Upload an unsigned MSI ZIP sample for SignPath configuration. |
| `test` | Test-sign and upload `signpath-test-windows-*` artifacts. |
| `disabled` | Use the existing release flow; requires `release_tag`. |

Leave `release_tag` empty for `prepare` and `test`; neither publishes a release.
For test builds, approve the SignPath request if required and verify signatures
on both the MSI and installed `Lap.exe`. Self-signed certificates are not publicly
trusted. Production signing will be configured separately after SignPath review.

See [SignPath's GitHub integration](https://docs.signpath.io/trusted-build-systems/github).
